// Copyright (c) 2025, The Ruskit Authors
// MIT License
// All rights reserved.

use crate::errors::AmqpError;
use configs::{configs::Configs, dynamic::DynamicConfigs};
use lapin::{types::LongString, Channel, Connection, ConnectionProperties};
use std::sync::Arc;
use tracing::{debug, error};

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
