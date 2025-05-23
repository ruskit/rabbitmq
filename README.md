# Ruskit RabbitMQ

A robust RabbitMQ implementation that follows the messaging abstraction defined in the [Ruskit Messaging](https://github.com/ruskit/messaging) project. This implementation provides a comprehensive set of tools for working with RabbitMQ in Rust applications.

## Features

- **Complete RabbitMQ Topology Management**
  - Exchange declaration (Direct, Fanout, Topic, Headers, and Delayed messaging)
  - Queue declaration with configurable properties
  - Binding management between exchanges and queues
  
- **Robust Message Processing**
  - Dead Letter Queue (DLQ) support for handling failed messages
  - Retry mechanism with configurable attempts and delay
  - Message Time-To-Live (TTL) and queue length limits

- **Flexible Message Publishing**
  - Support for routing keys
  - Custom message headers
  - Content type specification

- **Consumer Management**
  - Easy handler registration for message types
  - Single and multi-queue consumption patterns
  - Automatic acknowledgment/negative-acknowledgment handling

- **Observability**
  - OpenTelemetry integration for distributed tracing
  - Comprehensive error reporting and logging

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rabbitmq = { git = "ssh://git@github.com/ruskit/rabbitmq.git", rev = "<latest-revision>" }
```

## Usage

### Creating a RabbitMQ Connection and Channel

```rust
use rabbitmq::channel::new_amqp_channel;
use configs::configs::Configs;

async fn setup_connection(config: &Configs<MyConfig>) -> Result<(Arc<Connection>, Arc<Channel>), AmqpError> {
    new_amqp_channel(config).await
}
```

### Defining and Creating Topology

```rust
use rabbitmq::topology::{AmqpTopology, Topology};
use rabbitmq::exchange::{ExchangeDefinition, ExchangeKind};
use rabbitmq::queue::{QueueDefinition, QueueBinding};

async fn setup_topology(channel: Arc<Channel>) -> Result<(), AmqpError> {
    // Define exchanges
    let direct_exchange = ExchangeDefinition::new("direct-exchange").direct().durable();
    let fanout_exchange = ExchangeDefinition::new("fanout-exchange").fanout().durable();
    
    // Define queues with DLQ and retry mechanism
    let queue = QueueDefinition::new("my-queue")
        .durable()
        .with_dlq()
        .with_retry(5000, 3); // 5000ms delay, 3 retries
    
    // Define bindings
    let binding = QueueBinding::new("my-queue")
        .exchange("direct-exchange")
        .routing_key("my-routing-key");
    
    // Create and install topology
    let topology = AmqpTopology::new(channel)
        .exchange(&direct_exchange)
        .exchange(&fanout_exchange)
        .queue(&queue)
        .queue_binding(&binding);
    
    topology.install().await
}
```

### Publishing Messages

```rust
use rabbitmq::publisher::RabbitMQPublisher;
use messaging::publisher::{PublishMessage, HeaderValues};
use opentelemetry::Context;
use std::collections::HashMap;

async fn publish_message(channel: Arc<Channel>) -> Result<(), MessagingError> {
    let publisher = RabbitMQPublisher::new(channel);
    
    // Create message payload (as bytes)
    let payload = serde_json::to_vec(&my_data).unwrap();
    
    // Optional: Add custom headers
    let mut headers = HashMap::new();
    headers.insert("custom-header".to_string(), HeaderValues::LongString("value".to_string()));
    
    // Create the message
    let message = PublishMessage {
        to: "direct-exchange".to_string(),
        key: Some("my-routing-key".to_string()),
        msg_type: Some("my-message-type".to_string()),
        data: payload,
        headers: Some(headers),
    };
    
    // Get OpenTelemetry context (or create a new one)
    let context = Context::current();
    
    // Publish the message
    publisher.publish(&context, &message).await
}
```

### Consuming Messages

```rust
use rabbitmq::dispatcher::RabbitMQDispatcher;
use messaging::dispatcher::DispatcherDefinition;
use messaging::handler::ConsumerHandler;

struct MyMessageHandler;

#[async_trait]
impl ConsumerHandler for MyMessageHandler {
    async fn exec(&self, ctx: &Context, msg: &ConsumerMessage) -> Result<(), MessagingError> {
        // Process the message
        println!("Received message: {:?}", msg);
        
        // Return Ok if processing succeeded, or Err if it failed
        Ok(())
    }
}

async fn consume_messages(channel: Arc<Channel>, queues: Vec<QueueDefinition>) -> Result<(), MessagingError> {
    // Create the dispatcher
    let dispatcher = RabbitMQDispatcher::new(channel, queues)
        .register(
            &DispatcherDefinition::new("my-queue", Some("my-message-type")),
            Arc::new(MyMessageHandler)
        );
    
    // Start consuming messages (this will block)
    dispatcher.consume_blocking().await
}
```

### Using Multiple Consumers

```rust
async fn consume_multiple_queues(channel: Arc<Channel>, queues: Vec<QueueDefinition>) -> Result<(), MessagingError> {
    let dispatcher = RabbitMQDispatcher::new(channel, queues)
        .register(
            &DispatcherDefinition::new("queue1", Some("message-type-1")),
            Arc::new(Handler1)
        )
        .register(
            &DispatcherDefinition::new("queue2", Some("message-type-2")),
            Arc::new(Handler2)
        );
    
    // Use multi-consumer mode
    dispatcher.consume_blocking_multi().await
}
```

## Error Handling

This library provides comprehensive error types through the `AmqpError` enum:

```rust
use rabbitmq::errors::AmqpError;

fn handle_error(error: AmqpError) {
    match error {
        AmqpError::ConnectionError => println!("Failed to connect to RabbitMQ"),
        AmqpError::ChannelError => println!("Failed to create channel"),
        AmqpError::DeclareExchangeError(name) => println!("Failed to declare exchange: {}", name),
        AmqpError::DeclareQueueError(name) => println!("Failed to declare queue: {}", name),
        // Handle other error variants...
        _ => println!("Other error: {:?}", error),
    }
}
```

## Distributed Tracing

This library integrates with OpenTelemetry for distributed tracing:

```rust
use opentelemetry::global;
use opentelemetry::sdk::propagation::TraceContextPropagator;

fn init_tracer() {
    global::set_text_map_propagator(TraceContextPropagator::new());
    
    // Configure your OpenTelemetry tracer here
    // ...
}
```

## License

MIT License

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.