// Copyright (c) 2025, The Ruskit Authors
// MIT License
// All rights reserved.

//! # Queue Management for RabbitMQ
//!
//! This module provides types and functions for defining and managing RabbitMQ queues.
//! It includes support for Dead Letter Queues (DLQ) and retry mechanisms, which are
//! essential for robust message handling in distributed systems.

/// Definition of a RabbitMQ queue with its configuration parameters.
///
/// This struct implements the builder pattern to create and configure queue definitions.
/// It supports standard queue options as well as advanced features like message TTL,
/// max length, Dead Letter Queues (DLQ), and retry mechanisms.
#[derive(Debug, Clone, Default)]
pub struct QueueDefinition {
    pub(crate) name: String,
    pub(crate) durable: bool,
    pub(crate) delete: bool,
    pub(crate) exclusive: bool,
    pub(crate) passive: bool,
    pub(crate) no_wait: bool,
    pub(crate) ttl: Option<i32>,
    pub(crate) max_length: Option<i32>,
    pub(crate) max_length_bytes: Option<i32>,
    pub(crate) dlq_name: Option<String>,
    pub(crate) retry_name: Option<String>,
    pub(crate) retry_ttl: Option<i32>,
    pub(crate) retries: Option<i32>,
}

impl QueueDefinition {
    /// Creates a new queue definition with the given name.
    ///
    /// By default, the queue is created with standard settings (non-durable, non-exclusive, etc.)
    ///
    /// # Parameters
    /// * `name` - The name of the queue
    ///
    /// # Returns
    /// A new queue definition with default settings
    pub fn new(name: &str) -> QueueDefinition {
        QueueDefinition {
            name: name.to_owned(),
            durable: false,
            delete: false,
            exclusive: false,
            passive: false,
            no_wait: false,
            ttl: None,
            max_length: None,
            max_length_bytes: None,
            dlq_name: None,
            retry_name: None,
            retry_ttl: None,
            retries: None,
        }
    }

    /// Makes the queue durable, persisting across broker restarts.
    ///
    /// Durable queues will survive broker restart, preserving messages.
    ///
    /// # Returns
    /// Self for method chaining
    pub fn durable(mut self) -> Self {
        self.durable = true;
        self
    }

    /// Sets the queue to auto-delete when no longer used.
    ///
    /// # Returns
    /// Self for method chaining
    pub fn delete(mut self) -> Self {
        self.delete = true;
        self
    }

    /// Makes the queue exclusive to the connection.
    ///
    /// Exclusive queues are deleted when the connection closes.
    ///
    /// # Returns
    /// Self for method chaining
    pub fn exclusive(mut self) -> Self {
        self.exclusive = true;
        self
    }

    /// Sets the message Time-To-Live (TTL) for the queue.
    ///
    /// Messages that exceed this TTL will be automatically removed from the queue.
    ///
    /// # Parameters
    /// * `ttl` - TTL in milliseconds
    ///
    /// # Returns
    /// Self for method chaining
    pub fn ttl(mut self, ttl: i32) -> Self {
        self.ttl = Some(ttl);
        self
    }

    /// Sets the maximum number of messages the queue can hold.
    ///
    /// When this limit is reached, the oldest messages will be discarded,
    /// or sent to a Dead Letter Queue if configured.
    ///
    /// # Parameters
    /// * `max` - Maximum number of messages
    ///
    /// # Returns
    /// Self for method chaining
    pub fn max_length(mut self, max: i32) -> Self {
        self.max_length = Some(max);
        self
    }

    /// Sets the maximum size in bytes the queue can hold.
    ///
    /// When this limit is reached, the oldest messages will be discarded,
    /// or sent to a Dead Letter Queue if configured.
    ///
    /// # Parameters
    /// * `max_bytes` - Maximum size in bytes
    ///
    /// # Returns
    /// Self for method chaining
    pub fn max_length_bytes(mut self, max_bytes: i32) -> Self {
        self.max_length_bytes = Some(max_bytes);
        self
    }

    /// Adds a Dead Letter Queue (DLQ) to the queue.
    ///
    /// The DLQ will receive messages that are rejected, expired, or overflow
    /// from the main queue. The DLQ name will be the main queue name with "-dlq" suffix.
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_dlq(mut self) -> Self {
        self.dlq_name = Some(format!("{}-dlq", self.name));
        self
    }

    /// Adds a retry mechanism to the queue.
    ///
    /// This creates a retry queue that temporarily holds failed messages before
    /// redelivering them to the main queue. The retry queue name will be the main
    /// queue name with "-retry" suffix.
    ///
    /// # Parameters
    /// * `ttl` - Time in milliseconds to wait before retrying
    /// * `retries` - Maximum number of retry attempts
    ///
    /// # Returns
    /// Self for method chaining
    pub fn with_retry(mut self, ttl: i32, retries: i32) -> Self {
        self.retry_name = Some(format!("{}-retry", self.name));
        self.retries = Some(retries);
        self.retry_ttl = Some(ttl);
        self
    }
}

/// Configuration for binding a queue to an exchange.
///
/// Queue bindings define how messages flow from exchanges to queues based on
/// routing keys and exchange types.
pub struct QueueBinding<'qeb> {
    pub(crate) queue_name: &'qeb str,
    pub(crate) exchange_name: &'qeb str,
    pub(crate) routing_key: &'qeb str,
}

impl<'qeb> QueueBinding<'qeb> {
    /// Creates a new queue binding for the given queue.
    ///
    /// By default, the exchange name and routing key are empty strings.
    /// These should be set using the `exchange` and `routing_key` methods.
    ///
    /// # Parameters
    /// * `queue` - The name of the queue to bind
    ///
    /// # Returns
    /// A new queue binding with default settings
    pub fn new(queue: &'qeb str) -> QueueBinding<'qeb> {
        QueueBinding {
            queue_name: queue,
            exchange_name: "",
            routing_key: "",
        }
    }

    /// Sets the exchange to bind the queue to.
    ///
    /// # Parameters
    /// * `exchange` - The name of the exchange
    ///
    /// # Returns
    /// Self for method chaining
    pub fn exchange(mut self, exchange: &'qeb str) -> Self {
        self.exchange_name = exchange;
        self
    }

    /// Sets the routing key for the binding.
    ///
    /// # Parameters
    /// * `key` - The routing key
    ///
    /// # Returns
    /// Self for method chaining
    pub fn routing_key(mut self, key: &'qeb str) -> Self {
        self.routing_key = key;
        self
    }
}
