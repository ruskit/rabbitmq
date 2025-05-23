// Copyright (c) 2025, The Ruskit Authors
// MIT License
// All rights reserved.

//! # RabbitMQ Topology Management
//!
//! This module provides functionality for defining and creating RabbitMQ topology components.
//! The topology includes exchanges, queues, and the bindings between them. It supports advanced
//! RabbitMQ features such as Dead Letter Queues (DLQs) and retry mechanisms.
//!
//! The main components are:
//! - `Topology` trait: Interface for topology management
//! - `AmqpTopology`: Implementation of the Topology trait for RabbitMQ
//! - Header constants: Constants for RabbitMQ header fields

use crate::{
    errors::AmqpError,
    exchange::{ExchangeBinding, ExchangeDefinition},
    queue::{QueueBinding, QueueDefinition},
};
use async_trait::async_trait;
use lapin::{
    options::{QueueBindOptions, QueueDeclareOptions},
    types::{AMQPValue, FieldTable, LongInt, LongString, ShortString},
    Channel,
};
use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};
use tracing::{debug, error};

/// Constant for the header field used to specify a dead letter exchange
pub const AMQP_HEADERS_DEAD_LETTER_EXCHANGE: &str = "x-dead-letter-exchange";
/// Constant for the header field used to specify a dead letter routing key
pub const AMQP_HEADERS_DEAD_LETTER_ROUTING_KEY: &str = "x-dead-letter-routing-key";
/// Constant for the header field used to specify message TTL
pub const AMQP_HEADERS_MESSAGE_TTL: &str = "x-message-ttl";
/// Constant for the header field used to specify maximum queue length
pub const AMQP_HEADERS_MAX_LENGTH: &str = "x-max-length";
/// Constant for the header field used to specify maximum queue size in bytes
pub const AMQP_HEADERS_MAX_LENGTH_BYTES: &str = "x-max-length-bytes";

/// Trait defining the interface for topology management.
///
/// This trait provides methods for registering exchanges, queues, and bindings,
/// as well as installing the topology to the RabbitMQ server.
#[async_trait]
pub trait Topology<'tp> {
    /// Adds an exchange definition to the topology.
    fn exchange(self, def: &'tp ExchangeDefinition) -> Self;

    /// Adds a queue definition to the topology.
    fn queue(self, def: &'tp QueueDefinition) -> Self;

    /// Adds an exchange-to-exchange binding to the topology.
    fn exchange_binding(self, binding: &'tp ExchangeBinding) -> Self;

    /// Adds a queue-to-exchange binding to the topology.
    fn queue_binding(self, binding: &'tp QueueBinding) -> Self;

    /// Installs the topology to the RabbitMQ server.
    ///
    /// This creates all the exchanges and queues, and sets up all the bindings.
    async fn install(&self) -> Result<(), AmqpError>;
}

/// RabbitMQ implementation of the Topology trait.
///
/// This struct maintains collections of exchanges, queues, and bindings,
/// and provides methods to install them to a RabbitMQ server.
pub struct AmqpTopology<'tp> {
    channel: Arc<Channel>,
    pub(crate) queues: HashMap<&'tp str, &'tp QueueDefinition>,
    pub(crate) queues_binding: Vec<&'tp QueueBinding<'tp>>,
    pub(crate) exchanges: Vec<&'tp ExchangeDefinition<'tp>>,
    pub(crate) exchanges_binding: Vec<&'tp ExchangeBinding>,
}

impl<'tp> AmqpTopology<'tp> {
    /// Creates a new AmqpTopology instance.
    ///
    /// # Parameters
    /// * `channel` - A channel to the RabbitMQ server
    ///
    /// # Returns
    /// A new AmqpTopology instance
    pub fn new(channel: Arc<Channel>) -> AmqpTopology<'tp> {
        AmqpTopology {
            channel,
            queues: HashMap::default(),
            queues_binding: vec![],
            exchanges: vec![],
            exchanges_binding: vec![],
        }
    }
}

#[async_trait]
impl<'tp> Topology<'tp> for AmqpTopology<'tp> {
    /// Adds an exchange definition to the topology.
    ///
    /// # Parameters
    /// * `def` - An exchange definition
    ///
    /// # Returns
    /// Self for method chaining
    fn exchange(mut self, def: &'tp ExchangeDefinition) -> Self {
        self.exchanges.push(def);
        self
    }

    /// Adds a queue definition to the topology.
    ///
    /// # Parameters
    /// * `def` - A queue definition
    ///
    /// # Returns
    /// Self for method chaining
    fn queue(mut self, def: &'tp QueueDefinition) -> Self {
        self.queues.insert(&def.name, def);
        self
    }

    /// Adds an exchange-to-exchange binding to the topology.
    ///
    /// # Parameters
    /// * `binding` - An exchange binding
    ///
    /// # Returns
    /// Self for method chaining
    fn exchange_binding(mut self, binding: &'tp ExchangeBinding) -> Self {
        self.exchanges_binding.push(binding);
        self
    }

    /// Adds a queue-to-exchange binding to the topology.
    ///
    /// # Parameters
    /// * `binding` - A queue binding
    ///
    /// # Returns
    /// Self for method chaining
    fn queue_binding(mut self, binding: &'tp QueueBinding) -> Self {
        self.queues_binding.push(binding);
        self
    }

    /// Installs the topology to the RabbitMQ server.
    ///
    /// This method performs the following operations in order:
    /// 1. Creates all exchanges
    /// 2. Creates all queues (including DLQs and retry queues if configured)
    /// 3. Sets up exchange-to-exchange bindings
    /// 4. Sets up queue-to-exchange bindings
    ///
    /// # Returns
    /// Ok(()) on success or AmqpError on failure
    async fn install(&self) -> Result<(), AmqpError> {
        self.install_exchange().await?;
        self.install_queue().await?;
        self.binding_exchanges().await?;
        self.binding_queues().await
    }
}

impl<'tp> AmqpTopology<'tp> {
    /// Creates all exchanges defined in the topology.
    ///
    /// # Returns
    /// Ok(()) on success or AmqpError on failure
    async fn install_exchange(&self) -> Result<(), AmqpError> {
        for exch in self.exchanges.clone() {
            debug!("creating exchange: {}", exch.name);

            match self
                .channel
                .exchange_declare(
                    exch.name,
                    exch.kind.clone().try_into().unwrap(),
                    lapin::options::ExchangeDeclareOptions {
                        passive: exch.passive,
                        durable: exch.durable,
                        auto_delete: exch.delete,
                        internal: exch.internal,
                        nowait: exch.no_wait,
                    },
                    FieldTable::from(exch.params.clone()),
                )
                .await
            {
                Err(err) => {
                    error!(
                        error = err.to_string(),
                        name = exch.name,
                        "error to declare the exchange"
                    );
                    Err(AmqpError::DeclareExchangeError(err.to_string()))
                }
                _ => Ok(()),
            }?;

            debug!("exchange: {} was created", exch.name);
        }

        Ok(())
    }

    /// Creates all queues defined in the topology.
    ///
    /// This includes creating any associated DLQs and retry queues.
    ///
    /// # Returns
    /// Ok(()) on success or AmqpError on failure
    async fn install_queue(&self) -> Result<(), AmqpError> {
        for (name, def) in self.queues.clone() {
            debug!("creating queue: {}", name);

            let mut queue_args = BTreeMap::new();

            if def.retry_name.is_some() {
                self.declare_retry(def, &mut queue_args).await?;
            }

            if def.dlq_name.is_some() {
                self.declare_dql(def, &mut queue_args).await?;
            }

            if def.ttl.is_some() {
                queue_args.insert(
                    ShortString::from(AMQP_HEADERS_MESSAGE_TTL),
                    AMQPValue::LongInt(LongInt::from(def.ttl.unwrap())),
                );
            }

            if def.max_length.is_some() {
                queue_args.insert(
                    ShortString::from(AMQP_HEADERS_MAX_LENGTH),
                    AMQPValue::LongInt(LongInt::from(def.max_length.unwrap())),
                );
            }

            if def.max_length_bytes.is_some() {
                queue_args.insert(
                    ShortString::from(AMQP_HEADERS_MAX_LENGTH_BYTES),
                    AMQPValue::LongInt(LongInt::from(def.max_length_bytes.unwrap())),
                );
            }

            match self
                .channel
                .queue_declare(
                    name,
                    QueueDeclareOptions {
                        passive: def.passive,
                        durable: def.durable,
                        exclusive: def.exclusive,
                        auto_delete: def.delete,
                        nowait: def.no_wait,
                    },
                    FieldTable::from(queue_args),
                )
                .await
            {
                Err(err) => {
                    error!(error = err.to_string(), "");
                    Err(AmqpError::DeclareQueueError(name.to_owned()))
                }
                _ => {
                    debug!("queue: {} was created", name);
                    Ok(())
                }
            }?;
        }

        Ok(())
    }

    /// Creates a retry queue for the specified queue.
    ///
    /// A retry queue holds failed messages temporarily before redelivering
    /// them to the original queue after a delay.
    ///
    /// # Parameters
    /// * `def` - The queue definition for which to create a retry queue
    /// * `queue_args` - Arguments for the main queue, which will be modified to include
    ///                  dead letter configuration pointing to the retry queue
    ///
    /// # Returns
    /// Ok(()) on success or AmqpError on failure
    async fn declare_retry(
        &self,
        def: &QueueDefinition,
        queue_args: &mut BTreeMap<ShortString, AMQPValue>,
    ) -> Result<(), AmqpError> {
        let mut args = BTreeMap::new();

        args.insert(
            ShortString::from(AMQP_HEADERS_DEAD_LETTER_EXCHANGE),
            AMQPValue::LongString(LongString::from("")),
        );
        args.insert(
            ShortString::from(AMQP_HEADERS_DEAD_LETTER_ROUTING_KEY),
            AMQPValue::LongString(LongString::from(def.name.clone())),
        );
        args.insert(
            ShortString::from(AMQP_HEADERS_MESSAGE_TTL),
            AMQPValue::LongInt(LongInt::from(def.retry_ttl.unwrap())),
        );

        let retry_name = def.retry_name.clone().unwrap();

        match self
            .channel
            .queue_declare(
                &retry_name,
                QueueDeclareOptions {
                    passive: def.passive,
                    durable: def.durable,
                    exclusive: def.exclusive,
                    auto_delete: def.delete,
                    nowait: def.no_wait,
                },
                FieldTable::from(args),
            )
            .await
        {
            Err(err) => {
                error!(error = err.to_string(), "failure to declare retry queue");
                Err(AmqpError::DeclareQueueError(retry_name))
            }
            _ => {
                queue_args.insert(
                    ShortString::from(AMQP_HEADERS_DEAD_LETTER_EXCHANGE),
                    AMQPValue::LongString(LongString::from("")),
                );

                queue_args.insert(
                    ShortString::from(AMQP_HEADERS_DEAD_LETTER_ROUTING_KEY),
                    AMQPValue::LongString(LongString::from(retry_name)),
                );
                Ok(())
            }
        }
    }

    /// Creates a Dead Letter Queue (DLQ) for the specified queue.
    ///
    /// A DLQ receives messages that fail processing after all retry attempts
    /// or that expire before being processed.
    ///
    /// # Parameters
    /// * `def` - The queue definition for which to create a DLQ
    /// * `queue_args` - Arguments for the main queue, which will be modified to include
    ///                  dead letter configuration pointing to the DLQ if no retry queue exists
    ///
    /// # Returns
    /// Ok(()) on success or AmqpError on failure
    async fn declare_dql(
        &self,
        def: &QueueDefinition,
        queue_args: &mut BTreeMap<ShortString, AMQPValue>,
    ) -> Result<(), AmqpError> {
        let dlq_name = def.dlq_name.clone().unwrap();

        match self
            .channel
            .queue_declare(
                &dlq_name,
                QueueDeclareOptions {
                    passive: def.passive,
                    durable: def.durable,
                    exclusive: def.exclusive,
                    auto_delete: def.delete,
                    nowait: def.no_wait,
                },
                FieldTable::default(),
            )
            .await
        {
            Err(err) => {
                error!(error = err.to_string(), "failure to declare retry queue");
                Err(AmqpError::DeclareQueueError(dlq_name))
            }
            _ => {
                if def.retry_name.is_none() {
                    queue_args.insert(
                        ShortString::from(AMQP_HEADERS_DEAD_LETTER_EXCHANGE),
                        AMQPValue::LongString(LongString::from("")),
                    );

                    queue_args.insert(
                        ShortString::from(AMQP_HEADERS_DEAD_LETTER_ROUTING_KEY),
                        AMQPValue::LongString(LongString::from(def.name.clone())),
                    );
                }
                Ok(())
            }
        }
    }

    /// Sets up exchange-to-exchange bindings.
    ///
    /// # Returns
    /// Ok(()) on success or AmqpError on failure
    async fn binding_exchanges(&self) -> Result<(), AmqpError> {
        // Currently not implemented
        Ok(())
    }

    /// Sets up queue-to-exchange bindings.
    ///
    /// # Returns
    /// Ok(()) on success or AmqpError on failure
    async fn binding_queues(&self) -> Result<(), AmqpError> {
        for binding in self.queues_binding.clone() {
            debug!(
                "binding queue: {} to the exchange: {} with the key: {}",
                binding.queue_name, binding.exchange_name, binding.routing_key
            );

            match self
                .channel
                .queue_bind(
                    binding.queue_name,
                    binding.exchange_name,
                    binding.routing_key,
                    QueueBindOptions { nowait: false },
                    FieldTable::default(),
                )
                .await
            {
                Err(err) => {
                    error!(error = err.to_string(), "error to bind queue to exchange");

                    Err(AmqpError::BindingExchangeToQueueError(
                        binding.exchange_name.to_owned(),
                        binding.queue_name.to_owned(),
                    ))
                }
                _ => Ok(()),
            }?;
        }

        debug!("queue was bounded");

        Ok(())
    }
}
