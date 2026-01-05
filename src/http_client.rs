use mv64e_mtb_dto::{ModelProjectConsentPurpose, Mtb};
use reqwest::{Certificate, Error};
use std::fmt::{Debug, Display, Formatter};
use std::fs;
use std::time::Duration;
use tracing::info;

pub(crate) struct HttpResponse {
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
pub(crate) struct HttpClientError(String);

impl Display for HttpClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Error> for HttpClientError {
    fn from(err: Error) -> Self {
        if err.is_status() {
            match err.status() {
                Some(status_code) => HttpClientError(format!(
                    "Failed to send MTB request: HTTP {status_code} - {err}"
                )),
                _ => HttpClientError(format!("Failed to send MTB request: {err}")),
            }
        } else if err.is_timeout() {
            HttpClientError("Failed to send MTB request: Timeout".to_string())
        } else if err.is_redirect() {
            HttpClientError("Failed to send MTB request: Redirect".to_string())
        } else if err.is_connect() {
            HttpClientError("Failed to send MTB request: Cannot connect".to_string())
        } else {
            HttpClientError(format!("Failed to send MTB request: {err}"))
        }
    }
}

pub(crate) struct HttpClient {
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
            base_url: Self::base_url(base_url),
            username,
            password,
            client: client_builder
                .build()
                .map_err(|_| HttpClientError("Failed to build HTTP client".to_string()))?,
        })
    }

    pub async fn send_to_dip(&self, payload: &Mtb) -> Result<HttpResponse, HttpClientError> {
        if let Some(metadata) = &payload.metadata {
            if metadata
                .model_project_consent
                .provisions
                .iter()
                .any(|provision| provision.purpose == ModelProjectConsentPurpose::Sequencing)
                || metadata.research_consents.is_some()
            {
                self.send_mtb_request(payload).await
            } else {
                self.send_delete_request(&payload.patient.id).await
            }
        } else {
            self.send_mtb_request(payload).await
        }
    }

    fn base_url(base_url: &str) -> String {
        if base_url.ends_with('/') {
            Self::base_url(base_url[0..base_url.len() - 1].to_string().as_str())
        } else {
            base_url.to_string()
        }
    }

    async fn send_mtb_request(&self, mtb: &Mtb) -> Result<HttpResponse, HttpClientError> {
        let url = format!("{}/mtb/etl/patient-record", &self.base_url);
        info!("Sending POST request to {}", url);
        let response = self
            .client
            .post(url)
            .basic_auth(
                self.username.clone().unwrap_or("anonymous".to_string()),
                self.password.clone(),
            )
            .timeout(Duration::from_secs(5))
            .json(&mtb)
            .send()
            .await
            .map_err(HttpClientError::from)?;

        Ok(HttpResponse {
            status_code: response.status().as_u16(),
            status_body: response.text().await.unwrap_or_default(),
        })
    }

    async fn send_delete_request(&self, patient_id: &str) -> Result<HttpResponse, HttpClientError> {
        let url = format!("{}/mtb/etl/patient/{}", &self.base_url, patient_id);
        info!("Sending DELETE request to {}", url);
        let response = self
            .client
            .delete(url)
            .basic_auth(
                self.username.clone().unwrap_or("anonymous".to_string()),
                self.password.clone(),
            )
            .timeout(Duration::from_secs(5))
            .send()
            .await
            .map_err(HttpClientError::from)?;

        Ok(HttpResponse {
            status_code: response.status().as_u16(),
            status_body: response.text().await.unwrap_or_default(),
        })
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use crate::http_client::HttpClient;
    use httpmock::Method::{DELETE, POST};
    use httpmock::MockServer;
    use mv64e_mtb_dto::{
        ConsentProvision, ModelProjectConsent, ModelProjectConsentPurpose, Mtb, MvhMetadata,
        MvhSubmissionType, Provision,
    };
    use rstest::rstest;

    #[rstest]
    #[case("http://localhost:8080", "http://localhost:8080")]
    #[case("http://localhost:8080/", "http://localhost:8080")]
    #[case("http://localhost:8080/api", "http://localhost:8080/api")]
    #[case("http://localhost:8080/api/", "http://localhost:8080/api")]
    #[case("http://localhost:8080/api//", "http://localhost:8080/api")]
    fn should_return_clean_base_url(#[case] in_base_url: &str, #[case] expected_base_url: &str) {
        assert_eq!(expected_base_url, HttpClient::base_url(in_base_url));
    }

    #[tokio::test]
    async fn should_send_delete_to_dip_on_sequencing_deny() {
        let mock_server = MockServer::start();
        let mock = mock_server.mock(|when, then| {
            when.method(POST).path("/mtb/etl/patient-record");
            then.status(200);
        });

        let mut mtb = Mtb::new_with_consent_rejected("12345678");
        mtb.metadata = Some(MvhMetadata {
            model_project_consent: ModelProjectConsent {
                date: None,
                provisions: vec![Provision {
                    date: "2025-10-17".to_string(),
                    provision_type: ConsentProvision::Deny,
                    purpose: ModelProjectConsentPurpose::Sequencing,
                }],
                version: "1".to_string(),
            },
            mvh_metadata_type: MvhSubmissionType::Test,
            research_consents: None,
            reason_research_consent_missing: None,
            transfer_tan: "42".to_string(),
        });

        let http_client = HttpClient::new(&mock_server.base_url(), None, None, None)
            .expect("Could not create client");
        let result = http_client.send_to_dip(&mtb).await;

        mock.assert();

        assert!(result.is_ok());
        assert!(result.expect("ok").has_valid_response_code());
    }

    #[tokio::test]
    async fn should_send_delete_to_dip_if_no_consent() {
        let mock_server = MockServer::start();
        let mock = mock_server.mock(|when, then| {
            when.method(DELETE).path("/mtb/etl/patient/12345678");
            then.status(200);
        });

        let mut mtb = Mtb::new_with_consent_rejected("12345678");
        mtb.metadata = Some(MvhMetadata {
            model_project_consent: ModelProjectConsent {
                date: None,
                provisions: vec![],
                version: "1".to_string(),
            },
            mvh_metadata_type: MvhSubmissionType::Test,
            research_consents: None,
            reason_research_consent_missing: None,
            transfer_tan: "42".to_string(),
        });

        let http_client = HttpClient::new(&mock_server.base_url(), None, None, None)
            .expect("Could not create client");
        let result = http_client.send_to_dip(&mtb).await;

        mock.assert();

        assert!(result.is_ok());
        assert!(result.expect("ok").has_valid_response_code());
    }

    #[tokio::test]
    async fn should_send_update_to_dip_on_sequencing_permit() {
        let mock_server = MockServer::start();
        let mock = mock_server.mock(|when, then| {
            when.method(POST).path("/mtb/etl/patient-record");
            then.status(201);
        });

        let mut mtb = Mtb::new_with_consent_rejected("12345678");
        mtb.metadata = Some(MvhMetadata {
            model_project_consent: ModelProjectConsent {
                date: None,
                provisions: vec![Provision {
                    date: "2025-10-17".to_string(),
                    provision_type: ConsentProvision::Permit,
                    purpose: ModelProjectConsentPurpose::Sequencing,
                }],
                version: "1".to_string(),
            },
            mvh_metadata_type: MvhSubmissionType::Test,
            research_consents: None,
            reason_research_consent_missing: None,
            transfer_tan: "42".to_string(),
        });

        let http_client = HttpClient::new(&mock_server.base_url(), None, None, None)
            .expect("Could not create client");
        let result = http_client.send_to_dip(&mtb).await;

        mock.assert();

        assert!(result.is_ok());
        assert!(result.expect("ok").has_valid_response_code());
    }

    #[tokio::test]
    #[rstest]
    #[case(200, true)]
    #[case(201, true)]
    #[case(400, true)]
    #[case(422, true)]
    #[case(500, false)]
    async fn should_indicate_status_code(#[case] status_code: u16, #[case] expected: bool) {
        let mock_server = MockServer::start();
        let mock = mock_server.mock(|when, then| {
            when.method(POST).path("/mtb/etl/patient-record");
            then.status(status_code);
        });

        let mut mtb = Mtb::new_with_consent_rejected("12345678");
        mtb.metadata = Some(MvhMetadata {
            model_project_consent: ModelProjectConsent {
                date: None,
                provisions: vec![Provision {
                    date: "2025-10-17".to_string(),
                    provision_type: ConsentProvision::Permit,
                    purpose: ModelProjectConsentPurpose::Sequencing,
                }],
                version: "1".to_string(),
            },
            mvh_metadata_type: MvhSubmissionType::Test,
            research_consents: None,
            reason_research_consent_missing: None,
            transfer_tan: "42".to_string(),
        });

        let http_client = HttpClient::new(&mock_server.base_url(), None, None, None)
            .expect("Could not create client");
        let result = http_client.send_to_dip(&mtb).await;

        mock.assert();

        assert!(result.is_ok());
        assert_eq!(result.expect("ok").has_valid_response_code(), expected);
    }
}
