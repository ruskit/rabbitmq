// Copyright (c) 2025, The Ruskit Authors
// MIT License
// All rights reserved.

//! # Exchange Management for RabbitMQ
//!
//! This module provides types and functions for defining and managing RabbitMQ exchanges.
//! Exchanges are the routing mechanism in RabbitMQ that determine how messages are
//! distributed to queues. This module defines different exchange types and provides
//! a builder pattern for creating exchange definitions.

use crate::errors::AmqpError;
use lapin::types::{AMQPValue, LongString, ShortString};
use std::collections::BTreeMap;

/// Constant for the header field used to specify the delayed exchange type
pub const AMQP_HEADERS_DELAYED_EXCHANGE_TYPE: &str = "x-delayed-type";

/// Represents the types of exchanges available in RabbitMQ.
///
/// Each exchange type has specific routing behavior:
/// - Direct: Routes messages to queues based on an exact match of routing keys
/// - Fanout: Broadcasts messages to all bound queues regardless of routing keys
/// - Topic: Routes messages based on wildcard pattern matching of routing keys
/// - Headers: Routes based on message header values instead of routing keys
/// - XMessageDelayed: Extension for delayed message delivery (plugin required)
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ExchangeKind {
    #[default]
    Direct,
    Fanout,
    Topic,
    Headers,
    XMessageDelayed,
}

impl TryInto<lapin::ExchangeKind> for ExchangeKind {
    type Error = AmqpError;

    /// Converts the internal ExchangeKind to lapin's ExchangeKind.
    ///
    /// This handles the special case of converting XMessageDelayed to a custom
    /// exchange type that requires the delayed message exchange plugin.
    fn try_into(self) -> Result<lapin::ExchangeKind, AmqpError> {
        match self {
            ExchangeKind::Direct => Ok(lapin::ExchangeKind::Direct),
            ExchangeKind::Fanout => Ok(lapin::ExchangeKind::Fanout),
            ExchangeKind::Headers => Ok(lapin::ExchangeKind::Headers),
            ExchangeKind::Topic => Ok(lapin::ExchangeKind::Topic),
            ExchangeKind::XMessageDelayed => {
                Ok(lapin::ExchangeKind::Custom("x-delayed-message".to_owned()))
            }
        }
    }
}

/// Definition of a RabbitMQ exchange with its configuration parameters.
///
/// This struct implements the builder pattern to create and configure exchange definitions.
/// It supports standard exchange types as well as special configurations like delayed messaging.
#[derive(Debug, Clone)]
pub struct ExchangeDefinition<'ex> {
    pub(crate) name: &'ex str,
    pub(crate) kind: &'ex ExchangeKind,
    pub(crate) delete: bool,
    pub(crate) durable: bool,
    pub(crate) passive: bool,
    pub(crate) internal: bool,
    pub(crate) no_wait: bool,
    pub(crate) params: BTreeMap<ShortString, AMQPValue>,
}

impl<'ex> ExchangeDefinition<'ex> {
    /// Creates a new exchange definition with the given name.
    ///
    /// By default, the exchange is created as a Direct exchange with default parameters.
    ///
    /// # Parameters
    /// * `name` - The name of the exchange
    ///
    /// # Returns
    /// A new exchange definition with default settings
    pub fn new(name: &'ex str) -> ExchangeDefinition<'ex> {
        ExchangeDefinition {
            name,
            kind: &ExchangeKind::Direct,
            delete: false,
            durable: false,
            passive: false,
            internal: false,
            no_wait: false,
            params: BTreeMap::default(),
        }
    }

    /// Sets the exchange type.
    ///
    /// # Parameters
    /// * `kind` - The exchange type
    ///
    /// # Returns
    /// Self for method chaining
    pub fn kind(mut self, kind: &'ex ExchangeKind) -> Self {
        self.kind = kind;
        self
    }

    /// Sets the exchange type to Direct.
    ///
    /// # Returns
    /// Self for method chaining
    pub fn direct(mut self) -> Self {
        self.kind = &ExchangeKind::Direct;
        self
    }

    /// Sets the exchange type to Fanout.
    ///
    /// # Returns
    /// Self for method chaining
    pub fn fanout(mut self) -> Self {
        self.kind = &ExchangeKind::Fanout;
        self
    }

    /// Creates a delayed direct exchange.
    ///
    /// This requires the x-delayed-message plugin to be installed on the RabbitMQ server.
    ///
    /// # Returns
    /// Self for method chaining
    pub fn direct_delead(mut self) -> Self {
        self.kind = &ExchangeKind::XMessageDelayed;
        self.params.insert(
            ShortString::from(AMQP_HEADERS_DELAYED_EXCHANGE_TYPE),
            AMQPValue::LongString(LongString::from("direct")),
        );
        self
    }

    /// Creates a delayed fanout exchange.
    ///
    /// This requires the x-delayed-message plugin to be installed on the RabbitMQ server.
    ///
    /// # Returns
    /// Self for method chaining
    pub fn fanout_delead(mut self) -> Self {
        self.kind = &ExchangeKind::XMessageDelayed;
        self.params.insert(
            ShortString::from(AMQP_HEADERS_DELAYED_EXCHANGE_TYPE),
            AMQPValue::LongString(LongString::from("fanout")),
        );
        self
    }

    /// Sets the exchange parameters.
    ///
    /// # Parameters
    /// * `params` - A map of exchange parameters
    ///
    /// # Returns
    /// Self for method chaining
    pub fn params(mut self, params: BTreeMap<ShortString, AMQPValue>) -> Self {
        self.params = params;
        self
    }

    /// Adds a single parameter to the exchange.
    ///
    /// # Parameters
    /// * `key` - The parameter name
    /// * `value` - The parameter value
    ///
    /// # Returns
    /// Self for method chaining
    pub fn param(mut self, key: ShortString, value: AMQPValue) -> Self {
        self.params.insert(key, value);
        self
    }

    /// Sets the exchange to auto-delete when no longer used.
    ///
    /// # Returns
    /// Self for method chaining
    pub fn delete(mut self) -> Self {
        self.delete = true;
        self
    }

    /// Makes the exchange durable, persisting across broker restarts.
    ///
    /// # Returns
    /// Self for method chaining
    pub fn durable(mut self) -> Self {
        self.durable = true;
        self
    }

    /// Makes the exchange passive, checking for existence without creating it.
    ///
    /// # Returns
    /// Self for method chaining
    pub fn passive(mut self) -> Self {
        self.passive = true;
        self
    }

    /// Makes the exchange internal, preventing direct publishing.
    ///
    /// # Returns
    /// Self for method chaining
    pub fn internal(mut self) -> Self {
        self.internal = true;
        self
    }

    /// Sets no_wait flag, making the operation non-blocking.
    ///
    /// # Returns
    /// Self for method chaining
    pub fn no_wait(mut self) -> Self {
        self.no_wait = true;
        self
    }
}

/// Placeholder struct for exchange binding operations.
///
/// This struct is intended for future implementation of exchange-to-exchange bindings.
pub struct ExchangeBinding {}
