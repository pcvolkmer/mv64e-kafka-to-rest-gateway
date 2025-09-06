use crate::cli::Cli;
use crate::http_client::{HttpClient, HttpClientError, HttpResponse};
use clap::Parser;
use mv64e_mtb_dto::Mtb;
use rdkafka::config::RDKafkaLogLevel;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::message::{BorrowedMessage, Headers};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::{ClientConfig, Message};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::error::Error;
use std::string::ToString;
use std::sync::LazyLock;
use std::time::Duration;
use tracing::{error, info, warn};

mod cli;
mod http_client;

#[derive(Serialize, Deserialize)]
struct ResponsePayload {
    request_id: String,
    status_code: u16,
    status_body: Value,
}

#[cfg(not(test))]
static CONFIG: LazyLock<Cli> = LazyLock::new(Cli::parse);

async fn handle_record(payload: Mtb) -> Result<HttpResponse, HttpClientError> {
    let client = HttpClient::new(
        &CONFIG.dnpm_dip_uri,
        CONFIG.dnpm_dip_username.clone(),
        CONFIG.dnpm_dip_password.clone(),
    );

    if let Some(metadata) = &payload.metadata {
        if !metadata.model_project_consent.provisions.is_empty()
            || metadata.research_consents.is_some()
        {
            client.send_mtb_request(&payload).await
        } else {
            client.send_delete_request(&payload.patient.id).await
        }
    } else {
        client.send_mtb_request(&payload).await
    }
}

fn extract_request_id(msg: &BorrowedMessage) -> Option<String> {
    match msg.headers() {
        None => None,
        Some(headers) => {
            if let Some(value) = headers
                .iter()
                .find(|header| header.key == "requestId")?
                .value
            {
                match str::from_utf8(value) {
                    Ok(value) => Some(value.to_string()),
                    Err(_) => None,
                }
            } else {
                None
            }
        }
    }
}

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

    let topic: &str = &CONFIG.topic.clone();

    consumer.subscribe(&[topic])?;

    info!("Kafka topic '{}' subscribed", CONFIG.topic);

    let mut producer_client_config = client_config();

    let producer: &FutureProducer = &producer_client_config
        .set("bootstrap.servers", &CONFIG.bootstrap_servers)
        .set("message.timeout.ms", "5000")
        .create()?;

    while let Ok(msg) = consumer.recv().await {
        if let Some(Ok(payload)) = msg.payload_view::<str>() {
            if let Some(Ok(key)) = msg.key_view::<str>() {
                let payload = if let Ok(payload) = serde_json::from_str::<Mtb>(payload) {
                    payload
                } else {
                    error!("Error deserializing payload");
                    continue;
                };

                let request_id = extract_request_id(&msg).unwrap_or_default();

                match handle_record(payload).await {
                    Err(err) => error!("{}", err),
                    Ok(response) => {
                        let response_payload = ResponsePayload {
                            request_id: request_id.to_string(),
                            status_code: response.status_code,
                            status_body: serde_json::from_str::<Value>(&response.status_body)
                                .unwrap_or(json!({})),
                        };
                        let response_payload = serde_json::to_string(&response_payload)?;

                        let response_record = FutureRecord::to(&CONFIG.response_topic)
                            .key(key)
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
                                info!("Response for '{request_id}' sent successfully");
                            }
                            Err((err, _)) => {
                                error!("Could not send response for '{request_id}': {err}");
                            }
                        }

                        if response.status_code == 200
                            || response.status_code == 201
                            || response.status_code == 400
                            || response.status_code == 422
                        {
                            consumer.commit_message(&msg, CommitMode::Async)?;
                        } else {
                            warn!(
                                "Unexpected Status Code for Request '{}': HTTP {}",
                                &request_id, response.status_code
                            );
                        }
                    }
                }
            } else {
                error!("Error getting key");
            }
        } else {
            error!("Error getting payload");
        }
    }

    Ok(())
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
    ssl_ca_file: None,
    ssl_cert_file: None,
    ssl_key_file: None,
    ssl_key_password: None,
});
