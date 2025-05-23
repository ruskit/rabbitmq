// Copyright (c) 2025, The Ruskit Authors
// MIT License
// All rights reserved.

//! # OpenTelemetry Integration for RabbitMQ
//!
//! This module provides integration with OpenTelemetry for distributed tracing.
//! It includes utilities for propagating trace context through RabbitMQ message headers,
//! extracting context from incoming messages, and creating trace spans for message processing.

use lapin::{
    protocol::basic::AMQPProperties,
    types::{AMQPValue, ShortString},
};
use opentelemetry::{
    global::{BoxedSpan, BoxedTracer},
    propagation::{Extractor, Injector},
    trace::{SpanKind, Tracer},
    Context,
};
use std::{borrow::Cow, collections::BTreeMap};
use tracing::error;

/// An adapter for injecting and extracting OpenTelemetry context from RabbitMQ headers.
///
/// This struct implements the OpenTelemetry `Injector` and `Extractor` traits,
/// allowing trace context to be propagated through RabbitMQ message headers.
pub(crate) struct RabbitMQTracePropagator<'a> {
    headers: &'a mut BTreeMap<ShortString, AMQPValue>,
}

impl<'a> RabbitMQTracePropagator<'a> {
    /// Creates a new RabbitMQTracePropagator.
    ///
    /// # Parameters
    /// * `headers` - A mutable reference to the BTreeMap containing RabbitMQ headers
    ///
    /// # Returns
    /// A new RabbitMQTracePropagator instance
    pub(crate) fn new(headers: &'a mut BTreeMap<ShortString, AMQPValue>) -> Self {
        Self { headers }
    }
}

impl Injector for RabbitMQTracePropagator<'_> {
    /// Sets a trace context key-value pair in RabbitMQ message headers.
    ///
    /// This method is called by OpenTelemetry when injecting trace context
    /// into outgoing messages.
    ///
    /// # Parameters
    /// * `key` - The header key
    /// * `value` - The header value
    fn set(&mut self, key: &str, value: String) {
        self.headers.insert(
            key.to_lowercase().into(),
            AMQPValue::LongString(value.into()),
        );
    }
}

impl Extractor for RabbitMQTracePropagator<'_> {
    /// Gets a trace context value from RabbitMQ message headers.
    ///
    /// This method is called by OpenTelemetry when extracting trace context
    /// from incoming messages.
    ///
    /// # Parameters
    /// * `key` - The header key to retrieve
    ///
    /// # Returns
    /// The header value as a string slice, or None if not found
    fn get(&self, key: &str) -> Option<&str> {
        self.headers.get(key).and_then(|header_value| {
            if let AMQPValue::LongString(header_value) = header_value {
                std::str::from_utf8(header_value.as_bytes())
                    .map_err(|e| error!("Error decoding header value {:?}", e))
                    .ok()
            } else {
                None
            }
        })
    }

    /// Gets all keys in the RabbitMQ message headers.
    ///
    /// # Returns
    /// A vector of header keys as string slices
    fn keys(&self) -> Vec<&str> {
        self.headers.keys().map(|header| header.as_str()).collect()
    }
}

/// Creates a new OpenTelemetry span for message processing.
///
/// This function extracts trace context from message properties and
/// creates a new span for processing the message.
///
/// # Parameters
/// * `props` - RabbitMQ message properties containing headers
/// * `tracer` - OpenTelemetry tracer
/// * `name` - Name for the new span (typically the message type)
///
/// # Returns
/// A tuple containing the extracted context and the new span
pub fn new_span(props: &AMQPProperties, tracer: &BoxedTracer, name: &str) -> (Context, BoxedSpan) {
    let ctx = opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&RabbitMQTracePropagator::new(
            &mut props.headers().clone().unwrap_or_default().inner().clone(),
        ))
    });

    let span = tracer
        .span_builder(Cow::from(name.to_owned()))
        .with_kind(SpanKind::Consumer)
        .start_with_context(tracer, &ctx);

    (ctx, span)
}
