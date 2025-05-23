// Copyright (c) 2025, The Ruskit Authors
// MIT License
// All rights reserved.

//! # RabbitMQ Message Dispatcher
//!
//! This module provides functionality for consuming messages from RabbitMQ queues.
//! It implements the `Dispatcher` trait from the messaging abstraction library,
//! supporting message handler registration and dispatch for different message types.
//!
//! The dispatcher supports both single-queue and multi-queue consumption patterns,
//! and integrates with OpenTelemetry for distributed tracing.

use crate::{consumer::consume, queue::QueueDefinition};
use async_trait::async_trait;
use futures_util::{future::join_all, StreamExt};
use lapin::{options::BasicConsumeOptions, types::FieldTable, Channel};
use messaging::{
    dispatcher::{Dispatcher, DispatcherDefinition},
    errors::MessagingError,
    handler::ConsumerHandler,
};
use opentelemetry::global;
use std::{collections::HashMap, sync::Arc};
use tracing::error;

/// RabbitMQ-specific dispatcher definition that associates a queue with a message handler.
///
/// This structure links a specific queue with its configuration to a message handler,
/// forming the core of the dispatcher registration system.
#[derive(Clone)]
pub struct RabbitMQDispatcherDefinition {
    pub(crate) queue_def: QueueDefinition,
    pub(crate) handler: Arc<dyn ConsumerHandler>,
}

/// RabbitMQ implementation of the Dispatcher trait.
///
/// This dispatcher manages consumers for RabbitMQ queues, routing received messages
/// to the appropriate handler based on message type.
pub struct RabbitMQDispatcher {
    channel: Arc<Channel>,
    queues_def: Vec<QueueDefinition>,
    pub(crate) dispatchers_def: HashMap<String, RabbitMQDispatcherDefinition>,
}

impl RabbitMQDispatcher {
    /// Creates a new RabbitMQ dispatcher.
    ///
    /// # Parameters
    /// * `channel` - A channel to the RabbitMQ server
    /// * `queues_def` - Queue definitions for all queues this dispatcher will manage
    ///
    /// # Returns
    /// A new RabbitMQDispatcher instance
    pub fn new(channel: Arc<Channel>, queues_def: Vec<QueueDefinition>) -> Self {
        RabbitMQDispatcher {
            channel,
            queues_def,
            dispatchers_def: HashMap::default(),
        }
    }
}

#[async_trait]
impl Dispatcher for RabbitMQDispatcher {
    /// Registers a message handler for a specific message type on a queue.
    ///
    /// This maps a message type to a handler and associates it with a queue.
    /// When messages of this type are received, they will be processed by the handler.
    ///
    /// # Parameters
    /// * `def` - Dispatcher definition containing queue name and message type
    /// * `handler` - Handler to process messages of the specified type
    ///
    /// # Returns
    /// Self for method chaining
    fn register(mut self, def: &DispatcherDefinition, handler: Arc<dyn ConsumerHandler>) -> Self {
        let mut queue_def = QueueDefinition::default();
        for queue in &self.queues_def {
            if def.name == queue.name {
                queue_def = queue.clone();
            }
        }

        self.dispatchers_def.insert(
            def.msg_type.clone().unwrap_or_default(),
            RabbitMQDispatcherDefinition { queue_def, handler },
        );

        self
    }

    /// Starts consuming messages in a blocking manner.
    ///
    /// This method starts consuming messages from the registered queues and
    /// dispatches them to the appropriate handlers. It blocks until completed.
    ///
    /// # Returns
    /// Ok(()) on success or MessagingError on failure
    async fn consume_blocking(&self) -> Result<(), MessagingError> {
        self.consume_blocking_single().await
    }
}

impl RabbitMQDispatcher {
    /// Consumes messages from a single queue in a blocking manner.
    ///
    /// This method is suitable when there's only one queue to consume from.
    /// It creates a consumer on the first registered queue and processes
    /// messages as they arrive.
    ///
    /// # Returns
    /// Ok(()) on success or MessagingError on failure
    pub async fn consume_blocking_single(&self) -> Result<(), MessagingError> {
        let key = self.dispatchers_def.keys().next().unwrap();
        let def = self.dispatchers_def.get(key).unwrap();

        let mut consumer = match self
            .channel
            .basic_consume(
                &def.queue_def.name,
                key,
                BasicConsumeOptions {
                    no_local: false,
                    no_ack: false,
                    exclusive: false,
                    nowait: false,
                },
                FieldTable::default(),
            )
            .await
        {
            Err(err) => {
                error!(error = err.to_string(), "error to create the consumer");
                Err(MessagingError::CreatingConsumerError)
            }
            Ok(c) => Ok(c),
        }?;

        let defs = self.dispatchers_def.clone();
        let channel = self.channel.clone();

        let spawned = tokio::spawn({
            async move {
                while let Some(result) = consumer.next().await {
                    match result {
                        Ok(delivery) => {
                            if let Err(err) = consume(
                                &global::tracer("amqp consumer"),
                                &delivery,
                                &defs,
                                channel.clone(),
                            )
                            .await
                            {
                                error!(error = err.to_string(), "error consume msg");
                            }
                        }

                        Err(err) => error!(error = err.to_string(), "errors consume msg"),
                    }
                }
            }
        })
        .await;

        if spawned.is_err() {
            return Err(MessagingError::ConsumerError("some error occur".to_owned()));
        }

        Ok(())
    }

    /// Consumes messages from multiple queues in a blocking manner.
    ///
    /// This method creates a separate consumer for each registered queue,
    /// processing messages in parallel as they arrive.
    ///
    /// # Returns
    /// Ok(()) on success or MessagingError on failure
    pub async fn consume_blocking_multi(&self) -> Result<(), MessagingError> {
        let mut spawns = vec![];

        for (msg_type, def) in &self.dispatchers_def {
            let mut consumer = match self
                .channel
                .basic_consume(
                    &def.queue_def.name,
                    msg_type,
                    BasicConsumeOptions {
                        no_local: false,
                        no_ack: false,
                        exclusive: false,
                        nowait: false,
                    },
                    FieldTable::default(),
                )
                .await
            {
                Err(err) => {
                    error!(error = err.to_string(), "failure to create the consumer");
                    Err(MessagingError::CreatingConsumerError)
                }
                Ok(c) => Ok(c),
            }?;

            let defs = self.dispatchers_def.clone();
            let channel = self.channel.clone();

            spawns.push(tokio::spawn({
                async move {
                    while let Some(result) = consumer.next().await {
                        match result {
                            Ok(delivery) => {
                                if let Err(err) = consume(
                                    &global::tracer("amqp consumer"),
                                    &delivery,
                                    &defs,
                                    channel.clone(),
                                )
                                .await
                                {
                                    error!(error = err.to_string(), "error consume msg")
                                }
                            }

                            Err(err) => error!(error = err.to_string(), "errors consume msg"),
                        }
                    }
                }
            }));
        }

        let spawned = join_all(spawns).await;
        for res in spawned {
            if res.is_err() {
                error!("tokio process error");
                return Err(MessagingError::InternalError);
            }
        }

        Ok(())
    }
}
