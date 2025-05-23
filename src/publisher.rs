// Copyright (c) 2025, The Ruskit Authors
// MIT License
// All rights reserved.

//! # RabbitMQ Message Publisher
//!
//! This module provides functionality for publishing messages to RabbitMQ exchanges.
//! It implements the `Publisher` trait from the messaging abstraction library,
//! supporting OpenTelemetry tracing for distributed request tracking.

use crate::otel::RabbitMQTracePropagator;
use async_trait::async_trait;
use lapin::{
    options::BasicPublishOptions,
    types::{
        AMQPValue, FieldTable, LongInt, LongLongInt, LongString, LongUInt, ShortInt, ShortString,
    },
    BasicProperties, Channel,
};
use messaging::{
    errors::MessagingError,
    publisher::{HeaderValues, PublishMessage, Publisher},
};
use opentelemetry::{global, Context};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};
use tracing::error;
use uuid::Uuid;

/// Default content type for JSON messages
pub const JSON_CONTENT_TYPE: &str = "application/json";

/// RabbitMQ implementation of the Publisher trait.
///
/// This publisher sends messages to RabbitMQ exchanges, with support for
/// message headers, routing keys, and OpenTelemetry context propagation.
pub struct RabbitMQPublisher {
    channel: Arc<Channel>,
}

impl RabbitMQPublisher {
    /// Creates a new RabbitMQ publisher.
    ///
    /// # Parameters
    /// * `channel` - A channel to the RabbitMQ server
    ///
    /// # Returns
    /// An Arc-wrapped RabbitMQPublisher instance for thread-safe sharing
    pub fn new(channel: Arc<Channel>) -> Arc<RabbitMQPublisher> {
        Arc::new(RabbitMQPublisher { channel })
    }
}

#[async_trait]
impl Publisher for RabbitMQPublisher {
    /// Publishes a message to RabbitMQ.
    ///
    /// This method publishes a message to the specified exchange with the given
    /// routing key, message type, and payload. It also propagates OpenTelemetry
    /// trace context in the message headers for distributed tracing.
    ///
    /// # Parameters
    /// * `ctx` - OpenTelemetry context for tracing
    /// * `infos` - Message details including payload, exchange, routing key, etc.
    ///
    /// # Returns
    /// Ok(()) on success or MessagingError on failure
    async fn publish(&self, ctx: &Context, infos: &PublishMessage) -> Result<(), MessagingError> {
        let mut btree = BTreeMap::<ShortString, AMQPValue>::default();

        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(ctx, &mut RabbitMQTracePropagator::new(&mut btree))
        });

        if infos.headers.is_some() {
            self.btree_map(&infos.headers.clone().unwrap(), &mut btree);
        }

        match self
            .channel
            .basic_publish(
                &infos.to,
                &infos.key.clone().unwrap_or_default(),
                BasicPublishOptions {
                    immediate: false,
                    mandatory: false,
                },
                &infos.data,
                BasicProperties::default()
                    .with_content_type(ShortString::from(JSON_CONTENT_TYPE))
                    .with_type(ShortString::from(
                        infos.msg_type.clone().unwrap_or_default(),
                    ))
                    .with_message_id(ShortString::from(Uuid::new_v4().to_string()))
                    .with_headers(FieldTable::from(btree)),
            )
            .await
        {
            Err(err) => {
                error!(error = err.to_string(), "error publishing message");
                Err(MessagingError::PublisherError)
            }
            _ => Ok(()),
        }
    }
}

impl RabbitMQPublisher {
    /// Converts a HashMap of header values to a BTreeMap of AMQP values.
    ///
    /// This internal method handles the conversion between the generic HeaderValues
    /// from the messaging abstraction and the specific AMQPValue types needed by RabbitMQ.
    ///
    /// # Parameters
    /// * `hash_map` - HashMap of header values from PublishMessage
    /// * `btree` - BTreeMap to populate with AMQP values
    fn btree_map(
        &self,
        hash_map: &HashMap<String, HeaderValues>,
        btree: &mut BTreeMap<ShortString, AMQPValue>,
    ) {
        for (key, value) in hash_map.clone() {
            let amqp_value = match value {
                HeaderValues::ShortString(v) => AMQPValue::ShortString(ShortString::from(v)),
                HeaderValues::LongString(v) => AMQPValue::LongString(LongString::from(v)),
                HeaderValues::Int(v) => AMQPValue::ShortInt(ShortInt::from(v)),
                HeaderValues::LongInt(v) => AMQPValue::LongInt(LongInt::from(v)),
                HeaderValues::LongLongInt(v) => AMQPValue::LongLongInt(LongLongInt::from(v)),
                HeaderValues::Uint(v) => AMQPValue::LongUInt(LongUInt::from(v)),
                HeaderValues::LongUint(v) => AMQPValue::LongUInt(LongUInt::from(v)),
                HeaderValues::LongLongUint(v) => AMQPValue::LongUInt(LongUInt::from(v as u32)),
            };

            btree.insert(ShortString::from(key), amqp_value);
        }
    }
}
