//! HTTP client for the running `yahboom-mcp` gateway. Read-only opens are
//! GETs. Chassis and payload writes go through `POST /api/v1/control/tool`
//! and only after the caller has set `confirm`.
use crate::creds::with_timeout_secs;
use crate::error::DeviceError;
use crate::import::Settings;
use serde_json::{json, Value};
use std::time::Duration;

const MAX_BODY: usize = 32 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct Gateway {
    base: String,
    client: reqwest::Client,
}

impl Gateway {
    pub fn new(settings: &Settings) -> Result<Self, DeviceError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(8))
            .build()
            .map_err(|err| DeviceError::Protocol(format!("gateway client: {err}")))?;
        Ok(Self {
            base: settings.gateway_url.trim_end_matches('/').to_string(),
            client,
        })
    }

    pub async fn get_json(&self, path: &str) -> Result<Value, DeviceError> {
        self.request("GET", path, None).await
    }

    pub async fn post_tool(
        &self,
        operation: &str,
        param1: Option<&str>,
        param2: Option<&str>,
        param3: Option<&str>,
    ) -> Result<Value, DeviceError> {
        let body = json!({
            "operation": operation,
            "param1": param1,
            "param2": param2,
            "param3": param3,
        });
        self.request("POST", "/api/v1/control/tool", Some(body))
            .await
    }

    async fn request(
        &self,
        method: &str,
        path: &str,
        body: Option<Value>,
    ) -> Result<Value, DeviceError> {
        if !path.starts_with("/api/") && path != "/stream" {
            return Err(DeviceError::InvalidInput(format!(
                "refusing non-api path '{path}'"
            )));
        }
        let url = format!("{}{path}", self.base);
        let builder = match method {
            "GET" => self.client.get(&url),
            "POST" => self
                .client
                .post(&url)
                .json(&body.unwrap_or_else(|| json!({}))),
            _ => {
                return Err(DeviceError::InvalidInput(format!(
                    "unsupported method {method}"
                )))
            }
        };
        let response = with_timeout_secs(path, 8, builder.send())
            .await?
            .map_err(|err| map_req(path, err))?;
        let status = response.status();
        let len = response.content_length().unwrap_or(0);
        if len > MAX_BODY as u64 {
            return Err(DeviceError::Protocol(format!(
                "{path} payload exceeds 32 MiB"
            )));
        }
        let bytes = with_timeout_secs(path, 8, response.bytes())
            .await?
            .map_err(|err| map_req(path, err))?;
        if bytes.len() > MAX_BODY {
            return Err(DeviceError::Protocol(format!(
                "{path} payload exceeds 32 MiB"
            )));
        }
        if !status.is_success() {
            let snippet = String::from_utf8_lossy(&bytes);
            let snippet = snippet.chars().take(180).collect::<String>();
            return Err(DeviceError::Protocol(format!(
                "{path} HTTP {status}: {snippet}"
            )));
        }
        serde_json::from_slice(&bytes)
            .map_err(|err| DeviceError::Protocol(format!("{path} decode: {err}")))
    }
}

fn map_req(op: &str, err: reqwest::Error) -> DeviceError {
    if err.is_timeout() {
        DeviceError::Timeout(op.to_string())
    } else if err.is_connect() {
        DeviceError::Offline(format!("{op}: {err}"))
    } else {
        DeviceError::Protocol(format!("{op}: {err}"))
    }
}
