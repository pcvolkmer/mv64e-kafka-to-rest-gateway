use crate::cli::Cli;
use crate::http_client::HttpClient;
use crate::message::Message;
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::error::Error;
use std::string::ToString;
use std::sync::LazyLock;
use std::time::Duration;
use tracing::{error, info, warn};

#[cfg(not(test))]
use clap::Parser;

mod cli;
mod http_client;
mod message;

#[derive(Serialize, Deserialize)]
struct ResponsePayload {
    request_id: String,
    status_code: u16,
    status_body: Value,
}

#[cfg(not(test))]
static CONFIG: LazyLock<Cli> = LazyLock::new(Cli::parse);

fn client_config() -> ClientConfig {
    let mut client_config = ClientConfig::new();
    client_config.set("bootstrap.servers", &CONFIG.bootstrap_servers);

    if CONFIG.ssl_cert_file.is_some() || CONFIG.ssl_key_file.is_some() {
        client_config
            .set("security.protocol", "ssl")
            .set(
                "ssl.ca.location",
                CONFIG.ssl_ca_file.clone().unwrap_or_default(),
            )
            .set(
                "ssl.certificate.location",
                CONFIG.ssl_cert_file.clone().unwrap_or_default(),
            )
            .set(
                "ssl.key.location",
                CONFIG.ssl_key_file.clone().unwrap_or_default(),
            );
        if let Some(ssl_key_password) = &CONFIG.ssl_key_password {
            client_config.set("ssl.key.password", ssl_key_password);
        }
        client_config
    } else {
        client_config
    }
}

async fn start_service(
    consumer: StreamConsumer,
    producer: &FutureProducer,
    http_client: HttpClient,
) -> Result<(), Box<dyn Error>> {
    let topic: &str = &CONFIG.topic.clone();
    consumer.subscribe(&[topic])?;
    info!("Kafka topic '{}' subscribed", CONFIG.topic);

    while let Ok(msg) = consumer.recv().await {
        let msg = match Message::try_from(msg) {
            Ok(msg) => msg,
            Err(e) => {
                error!(e);
                continue;
            }
        };

        match http_client.send_to_dip(&msg.payload()).await {
            Err(err) => error!("{}", err),
            Ok(response) => {
                let response_payload = ResponsePayload {
                    request_id: msg.request_id(),
                    status_code: response.status_code,
                    status_body: serde_json::from_str::<Value>(&response.status_body)
                        .unwrap_or(json!({})),
                };
                let Ok(response_payload) = serde_json::to_string(&response_payload) else {
                    error!("Error serializing response");
                    continue;
                };
                let key = msg.key();
                let response_record = FutureRecord::to(&CONFIG.response_topic)
                    .key(&key)
                    .payload(&response_payload);

                match if let Some(headers) = msg.headers() {
                    producer
                        .send(
                            response_record.headers(headers.detach()),
                            Duration::from_secs(1),
                        )
                        .await
                } else {
                    producer.send(response_record, Duration::from_secs(1)).await
                } {
                    Ok(_) => {
                        info!("Response for '{}' sent successfully", msg.request_id());
                    }
                    Err((err, _)) => {
                        error!("Could not send response for '{}': {err}", msg.request_id());
                    }
                }

                if response.has_valid_response_code() {
                    let _ = msg.commit(&consumer);
                } else {
                    warn!(
                        "Unexpected Status Code for Request '{}': HTTP {}",
                        &msg.request_id(),
                        response.status_code
                    );
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    #[cfg(debug_assertions)]
    {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    }

    #[cfg(not(debug_assertions))]
    {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    let mut consumer_client_config = client_config();

    let consumer: StreamConsumer = consumer_client_config
        .set("group.id", &CONFIG.group_id)
        .set("enable.partition.eof", "false")
        .set("auto.offset.reset", "earliest")
        .set("enable.auto.commit", "false")
        .set_log_level(RDKafkaLogLevel::Debug)
        .create()?;

    let mut producer_client_config = client_config();

    let producer: &FutureProducer = &producer_client_config
        .set("bootstrap.servers", &CONFIG.bootstrap_servers)
        .set("message.timeout.ms", "5000")
        .create()?;

    let http_client = HttpClient::new(
        &CONFIG.dnpm_dip_uri,
        CONFIG.dnpm_dip_username.clone(),
        CONFIG.dnpm_dip_password.clone(),
        CONFIG.dnpm_dip_ca_file.clone(),
    )
    .map_err(|err| err.to_string())?;

    start_service(consumer, producer, http_client).await
}

// Test Configuration
#[cfg(test)]
static CONFIG: LazyLock<Cli> = LazyLock::new(|| Cli {
    bootstrap_servers: "localhost:9094".to_string(),
    topic: "test-topic".to_string(),
    response_topic: "test-response-topic".to_string(),
    group_id: "test-group-id".to_string(),
    dnpm_dip_uri: "http://localhost:8000/api".to_string(),
    dnpm_dip_username: None,
    dnpm_dip_password: None,
    dnpm_dip_ca_file: None,
    ssl_ca_file: None,
    ssl_cert_file: None,
    ssl_key_file: None,
    ssl_key_password: None,
});
