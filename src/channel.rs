// Copyright (c) 2025, The Ruskit Authors
// MIT License
// All rights reserved.

//! # AMQP Channel Management
//!
//! This module handles the creation and management of AMQP connections and channels.
//! It provides functionality to establish connections to RabbitMQ server and create
//! communication channels for message publishing and consuming.

use crate::errors::AmqpError;
use configs::{configs::Configs, dynamic::DynamicConfigs};
use lapin::{types::LongString, Channel, Connection, ConnectionProperties};
use std::sync::Arc;
use tracing::{debug, error};

/// Creates a new AMQP channel for communication with RabbitMQ.
///
/// This function establishes a connection to RabbitMQ using configuration parameters
/// provided in the `cfg` argument, then creates a channel on that connection.
/// Both the connection and channel are wrapped in Arc for thread-safe sharing.
///
/// # Parameters
/// * `cfg` - Configuration containing RabbitMQ connection details like host, port, credentials, etc.
///
/// # Returns
/// * `Result<(Arc<Connection>, Arc<Channel>), AmqpError>` -
///   A tuple containing the connection and channel on success, or an error on failure.
///
/// # Example
/// ```
/// let (conn, channel) = new_amqp_channel(&config).await?;
/// ```
pub async fn new_amqp_channel<T>(
    cfg: &Configs<T>,
) -> Result<(Arc<Connection>, Arc<Channel>), AmqpError>
where
    T: DynamicConfigs,
{
    debug!("creating amqp connection...");
    let options = ConnectionProperties::default()
        .with_connection_name(LongString::from(cfg.app.name.clone()));

    let uri = format!(
        "amqp://{}:{}@{}:{}/{}",
        cfg.rabbitmq.user,
        cfg.rabbitmq.password,
        cfg.rabbitmq.host,
        cfg.rabbitmq.port,
        cfg.rabbitmq.vhost
    );

    let conn = match Connection::connect(&uri, options).await {
        Ok(c) => Ok(c),
        Err(err) => {
            error!(error = err.to_string(), "failure to connect");
            Err(AmqpError::ConnectionError {})
        }
    }?;
    debug!("amqp connected");

    debug!("creating amqp channel...");
    match conn.create_channel().await {
        Ok(c) => {
            debug!("channel created");
            Ok((Arc::new(conn), Arc::new(c)))
        }
        Err(err) => {
            error!(error = err.to_string(), "error to create the channel");
            Err(AmqpError::ChannelError {})
        }
    }
}
