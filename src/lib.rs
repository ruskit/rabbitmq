// Copyright (c) 2025, The Ruskit Authors
// MIT License
// All rights reserved.

//! # RabbitMQ Implementation for Ruskit Messaging
//!
//! This crate provides a RabbitMQ implementation of the messaging abstraction defined
//! in the `messaging` crate. It includes functionality for creating and managing
//! RabbitMQ topology (exchanges, queues, and bindings), publishing messages,
//! and consuming messages with robust error handling and retry mechanisms.
//!
//! ## Main Components
//!
//! - **Topology**: Define and create RabbitMQ exchanges, queues, and bindings
//! - **Publisher**: Publish messages to RabbitMQ exchanges
//! - **Dispatcher**: Consume messages from RabbitMQ queues and route them to handlers
//! - **Error Handling**: Comprehensive error types and handling
//! - **OpenTelemetry Integration**: Distributed tracing support
//!
//! ## Features
//!
//! - **Dead Letter Queues (DLQ)**: Support for handling failed messages
//! - **Retry Mechanism**: Automatic retry for failed message processing
//! - **Message TTL**: Time-to-live settings for messages
//! - **Queue Length Limits**: Constraints on queue size
//! - **Delayed Messaging**: Support for delayed message delivery (requires RabbitMQ plugin)
//! - **Distributed Tracing**: Integration with OpenTelemetry
//!
//! ## Usage Example
//!
//! ```rust
//! use rabbitmq::channel::new_amqp_channel;
//! use rabbitmq::topology::{AmqpTopology, Topology};
//! use rabbitmq::exchange::ExchangeDefinition;
//! use rabbitmq::queue::QueueDefinition;
//! use rabbitmq::publisher::RabbitMQPublisher;
//! use rabbitmq::dispatcher::RabbitMQDispatcher;
//!
//! async fn setup(config: &Config) -> Result<(), Error> {
//!     // Create connection and channel
//!     let (conn, channel) = new_amqp_channel(config).await?;
//!
//!     // Define exchange
//!     let exchange = ExchangeDefinition::new("my-exchange").durable().fanout();
//!
//!     // Define queue with DLQ and retry mechanism
//!     let queue = QueueDefinition::new("my-queue")
//!         .durable()
//!         .with_dlq()
//!         .with_retry(5000, 3);
//!
//!     // Create topology
//!     let topology = AmqpTopology::new(channel.clone())
//!         .exchange(&exchange)
//!         .queue(&queue);
//!
//!     // Install topology
//!     topology.install().await?;
//!
//!     // Create publisher
//!     let publisher = RabbitMQPublisher::new(channel.clone());
//!
//!     // Create dispatcher
//!     let dispatcher = RabbitMQDispatcher::new(channel, vec![queue])
//!         .register(&DispatcherDefinition::new("my-queue", Some("my-message-type")), handler);
//!
//!     // Start consuming messages
//!     dispatcher.consume_blocking().await?;
//!
//!     Ok(())
//! }
//! ```

mod consumer;
mod otel;

pub mod channel;
pub mod dispatcher;
pub mod errors;
pub mod exchange;
pub mod publisher;
pub mod queue;
pub mod topology;
