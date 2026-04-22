use std::time::Duration;

use futures_lite::StreamExt;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions, BasicNackOptions, QueueDeclareOptions},
    types::FieldTable,
    Connection, ConnectionProperties,
};
use sqlx::MySqlPool;
use tracing::{error, info, warn};

use super::import_processor::process_import_message;

/// Spawn a background RabbitMQ consumer task.
///
/// The task runs for the entire lifetime of the process; it does not need to be
/// awaited by the caller.
pub async fn start_consumer(pool: MySqlPool) {
    let url = std::env::var("RABBITMQ_URL")
        .unwrap_or_else(|_| "amqp://guest:guest@rabbitmq:5672/%2F".to_string());
    let queue =
        std::env::var("RABBITMQ_QUEUE").unwrap_or_else(|_| "transactions-import".to_string());

    info!("Starting RabbitMQ consumer — url={} queue={}", url, queue);

    // Retry connection on startup (RabbitMQ may not be ready immediately)
    let connection = loop {
        match Connection::connect(&url, ConnectionProperties::default()).await {
            Ok(conn) => break conn,
            Err(e) => {
                warn!("Failed to connect to RabbitMQ: {}. Retrying in 3s...", e);
                tokio::time::sleep(Duration::from_secs(3)).await;
            }
        }
    };

    let channel = connection
        .create_channel()
        .await
        .expect("Failed to create RabbitMQ channel");

    channel
        .queue_declare(
            &queue,
            QueueDeclareOptions {
                durable: true,
                ..Default::default()
            },
            FieldTable::default(),
        )
        .await
        .expect("Failed to declare queue");

    let mut consumer = channel
        .basic_consume(
            &queue,
            "account-service",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to start consuming from queue");

    info!("RabbitMQ consumer listening on queue '{}'", queue);

    while let Some(delivery) = consumer.next().await {
        match delivery {
            Err(e) => {
                warn!("RabbitMQ delivery error: {}", e);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Ok(delivery) => {
                let payload = match std::str::from_utf8(&delivery.data) {
                    Ok(s) => s.to_owned(),
                    Err(e) => {
                        error!("Failed to decode message payload: {}", e);
                        delivery
                            .nack(BasicNackOptions {
                                requeue: false,
                                ..Default::default()
                            })
                            .await
                            .ok();
                        continue;
                    }
                };

                match process_import_message(&pool, &payload).await {
                    Ok(count) => {
                        info!("Imported {} transaction(s) from RabbitMQ message", count);
                        delivery.ack(BasicAckOptions::default()).await.ok();
                    }
                    Err(e) => {
                        error!("Failed to process import message: {}", e);
                        // Nack with requeue — message will be redelivered
                        delivery
                            .nack(BasicNackOptions {
                                requeue: true,
                                ..Default::default()
                            })
                            .await
                            .ok();
                    }
                }
            }
        }
    }
}
