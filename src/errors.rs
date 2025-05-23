// Copyright (c) 2025, The Ruskit Authors
// MIT License
// All rights reserved.

//! # Error Types for RabbitMQ Implementation
//!
//! This module provides a comprehensive set of error types for RabbitMQ operations.
//! The `AmqpError` enum represents all possible error scenarios that can occur during
//! connection, channel, exchange, queue, and message handling operations.

use thiserror::Error;

/// Represents errors that can occur during AMQP/RabbitMQ operations.
///
/// This enum covers all error scenarios for RabbitMQ interactions, including connection
/// issues, channel creation, exchange and queue declarations, message publishing,
/// and consumer-related errors. Each variant provides specific context about
/// what operation failed.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum AmqpError {
    /// Internal errors that don't fit into other categories
    #[error("internal error")]
    InternalError,

    /// Error establishing a connection to the RabbitMQ server
    #[error("failure to connect")]
    ConnectionError,

    /// Error creating a channel from an established connection
    #[error("failure to create a channel")]
    ChannelError,

    /// Error declaring an exchange with the given name
    #[error("failure to declare an exchange `{0}`")]
    DeclareExchangeError(String),

    /// Error declaring a queue with the given name
    #[error("failure to declare a queue `{0}`")]
    DeclareQueueError(String),

    /// Error binding an exchange to a queue
    #[error("failure to binding exchange `{0}` to queue `{0}`")]
    BindingExchangeToQueueError(String, String),

    /// Error binding a consumer to a queue
    #[error("failure to declare consumer `{0}`")]
    BindingConsumerError(String),

    /// Error publishing a message
    #[error("failure to publish")]
    PublishingError,

    /// Error parsing a message payload
    #[error("failure to parse payload")]
    ParsePayloadError,

    /// Error acknowledging a message
    #[error("failure to ack message")]
    AckMessageError,

    /// Error negative-acknowledging a message
    #[error("failure to nack message")]
    NackMessageError,

    /// Error requeuing a message
    #[error("failure to requeuing message")]
    RequeuingMessageError,

    /// Error publishing a message to the Dead Letter Queue (DLQ)
    #[error("failure to publish to dlq")]
    PublishingToDQLError,

    /// Error configuring Quality of Service parameters
    #[error("failure to configure qos `{0}`")]
    QoSDeclarationError(String),

    /// Error declaring a consumer
    #[error("consumer declaration error")]
    ConsumerDeclarationError,

    /// Error consuming a message
    #[error("failure to consume message `{0}`")]
    ConsumerError(String),
}
