use mv64e_mtb_dto::Mtb;
use std::fmt::{Debug, Display, Formatter};
use std::time::Duration;

pub struct HttpResponse {
    pub status_code: u16,
    pub status_body: String,
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
    pub fn new(base_url: &str, username: Option<String>, password: Option<String>) -> Self {
        let user_agent_string = format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));

        Self {
            base_url: if base_url.ends_with("/") {
                base_url[0..base_url.len() - 2].to_string()
            } else {
                base_url.to_string()
            },
            username,
            password,
            client: reqwest::Client::builder()
                .user_agent(user_agent_string)
                .build()
                .expect("Failed to build HTTP client"),
        }
    }

    pub async fn send_mtb_request(&self, mtb: &Mtb) -> Result<HttpResponse, HttpClientError> {
        let response = self
            .client
            .post(format!("{}/mtb/etl/patient-record", &self.base_url))
            .basic_auth(
                &self.username.clone().unwrap_or_default(),
                Some(self.password.clone().unwrap_or_default()),
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
                &self.username.clone().unwrap_or_default(),
                Some(self.password.clone().unwrap_or_default()),
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
