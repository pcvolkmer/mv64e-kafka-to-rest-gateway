use mv64e_mtb_dto::Mtb;
use reqwest::Certificate;
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::time::Duration;
use tracing::info;

pub struct HttpResponse {
    pub status_code: u16,
    pub status_body: String,
}

impl HttpResponse {
    pub fn has_valid_response_code(&self) -> bool {
        self.status_code == 200
            || self.status_code == 201
            || self.status_code == 400
            || self.status_code == 422
    }
}

#[derive(Debug, Clone)]
pub struct HttpClientError(String);

impl Display for HttpClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct HttpClient {
    base_url: String,
    username: Option<String>,
    password: Option<String>,
    client: reqwest::Client,
}

impl HttpClient {
    pub fn new(
        base_url: &str,
        username: Option<String>,
        password: Option<String>,
        ssl_ca_file: Option<String>,
    ) -> Result<Self, HttpClientError> {
        let user_agent_string = format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

        // Allow invalid (self signed) certificates in dev/debug env only
        #[cfg(not(debug_assertions))]
        let accept_invalid_certificates = false;
        #[cfg(debug_assertions)]
        let accept_invalid_certificates = true;

        let client_builder = reqwest::Client::builder()
            .user_agent(user_agent_string)
            .danger_accept_invalid_certs(accept_invalid_certificates);

        let client_builder = if ssl_ca_file.is_some() {
            info!("Use HTTPS to connect to DNPM:DIP");
            let cert = match ssl_ca_file {
                Some(path) => match fs::read(path.as_str()) {
                    Ok(file) => Certificate::from_pem(&file)
                        .map_err(|_| HttpClientError("Failed to build HTTP client".to_string()))?,
                    _ => {
                        return Err(HttpClientError(
                            "Failed to load certificate from a ca-file".to_string(),
                        ))?;
                    }
                },
                _ => {
                    return Err(HttpClientError(
                        "Failed to load certificate from a ca-file".to_string(),
                    ))?;
                }
            };
            client_builder.add_root_certificate(cert)
        } else {
            client_builder
        };

        Ok(Self {
            base_url: if base_url.ends_with('/') {
                base_url[0..base_url.len() - 2].to_string()
            } else {
                base_url.to_string()
            },
            username,
            password,
            client: client_builder
                .build()
                .map_err(|_| HttpClientError("Failed to build HTTP client".to_string()))?,
        })
    }

    pub async fn send_mtb_request(&self, mtb: &Mtb) -> Result<HttpResponse, HttpClientError> {
        let response = self
            .client
            .post(format!("{}/mtb/etl/patient-record", &self.base_url))
            .basic_auth(
                self.username.clone().unwrap_or("anonymous".to_string()),
                self.password.clone(),
            )
            .timeout(Duration::from_secs(5))
            .json(&mtb)
            .send()
            .await
            .map_err(|err| HttpClientError(format!("Failed to send MTB request: {err}")))?;

        Ok(HttpResponse {
            status_code: response.status().as_u16(),
            status_body: response.text().await.unwrap_or_default(),
        })
    }

    pub async fn send_delete_request(
        &self,
        patient_id: &str,
    ) -> Result<HttpResponse, HttpClientError> {
        let response = self
            .client
            .delete(format!("{}/mtb/etl/patient/{}", &self.base_url, patient_id))
            .basic_auth(
                self.username.clone().unwrap_or("anonymous".to_string()),
                self.password.clone(),
            )
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(|err| HttpClientError(format!("Failed to send delete request: {err}")))?;

        Ok(HttpResponse {
            status_code: response.status().as_u16(),
            status_body: response.text().await.unwrap_or_default(),
        })
    }
}
