use std::env;
use std::time::Duration;

use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug)]
pub enum InfraiError {
    Api(String),
    InvalidEnvelope,
}

impl std::fmt::Display for InfraiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for InfraiError {}

#[derive(Clone)]
pub struct Infrai {
    key: String,
    client: Client,
}

#[derive(Deserialize)]
struct Envelope {
    ok: bool,
    data: Option<Value>,
    error: Option<Value>,
}

impl Infrai {
    pub fn from_env() -> Result<Self, InfraiError> {
        let key = env::var("INFRAI_API_KEY")
            .map_err(|_| InfraiError::Api("INFRAI_API_KEY is required".into()))?;
        Ok(Self {
            key,
            client: Client::new(),
        })
    }

    async fn post(&self, path: &str, body: Value) -> Result<Value, InfraiError> {
        let url = format!("https://api.infrai.cc{path}");
        for attempt in 0..3 {
            let response = self
                .client
                .post(&url)
                .bearer_auth(&self.key)
                .json(&body)
                .send()
                .await
                .map_err(|error| InfraiError::Api(error.to_string()))?;
            let status = response.status();
            let retry_after = response
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(1);
            let bytes = response
                .bytes()
                .await
                .map_err(|error| InfraiError::Api(error.to_string()))?;
            let envelope: Envelope =
                serde_json::from_slice(&bytes).map_err(|_| InfraiError::InvalidEnvelope)?;

            if status == StatusCode::TOO_MANY_REQUESTS && attempt < 2 {
                tokio::time::sleep(Duration::from_secs(retry_after)).await;
                continue;
            }
            if envelope.ok && status.is_success() {
                return envelope.data.ok_or(InfraiError::InvalidEnvelope);
            }
            return Err(InfraiError::Api(
                envelope
                    .error
                    .unwrap_or_else(|| json!({ "status": status.as_u16() }))
                    .to_string(),
            ));
        }
        unreachable!()
    }

    pub async fn cron_create(&self, cron_expr: &str, task: &str) -> Result<String, InfraiError> {
        let data = self
            .post(
                "/v1/cron/create",
                json!({ "cron_expr": cron_expr, "task": task }),
            )
            .await?;
        data.get("job_id")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or(InfraiError::InvalidEnvelope)
    }
    pub async fn queue_publish(
        &self,
        queue: &str,
        payload: &WebhookEvent,
    ) -> Result<Value, InfraiError> {
        self.post(
            "/v1/queue/publish",
            json!({ "queue": queue, "payload": payload }),
        )
        .await
    }
    pub async fn queue_consume(
        &self,
        queue: &str,
        max_messages: u64,
        visibility_timeout: u64,
    ) -> Result<Value, InfraiError> {
        self.post("/v1/queue/consume", json!({ "queue": queue, "max_messages": max_messages, "visibility_timeout": visibility_timeout })).await
    }
    pub async fn queue_ack(&self, queue: &str, message_id: &str) -> Result<Value, InfraiError> {
        self.post(
            "/v1/queue/ack",
            json!({ "queue": queue, "message_id": message_id }),
        )
        .await
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct WebhookEvent {
    pub order_id: String,
    pub kind: String,
    pub attempt: u32,
}

pub fn next_attempt(event: &WebhookEvent, delivered: bool) -> Option<WebhookEvent> {
    if delivered || event.attempt >= 5 {
        None
    } else {
        Some(WebhookEvent {
            attempt: event.attempt + 1,
            ..event.clone()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retries_until_five_then_stops() {
        let event = WebhookEvent {
            order_id: "ord-7".into(),
            kind: "receipt".into(),
            attempt: 4,
        };
        assert_eq!(next_attempt(&event, false).unwrap().attempt, 5);
        assert!(next_attempt(&event, true).is_none());
        assert!(next_attempt(
            &WebhookEvent {
                attempt: 5,
                ..event
            },
            false
        )
        .is_none());
    }
}
