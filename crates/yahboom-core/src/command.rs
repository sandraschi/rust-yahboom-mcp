//! Command bus. Reads degrade to a demo payload when the gateway is down.
//! Anything that can move the chassis or fire a payload requires `confirm`.
use crate::error::DeviceError;
use crate::gateway::Gateway;
use crate::import::{load_settings, Settings};
use serde_json::{json, Value};

const READ_PATHS: &[&str] = &[
    "/api/v1/health",
    "/api/v1/sensors",
    "/api/v1/telemetry",
    "/api/v1/diagnostics/logs",
    "/api/v1/diagnostics/stack",
    "/api/v1/diagnostics/ros/topics",
    "/api/v1/slam/map",
    "/api/v1/lidar/dreame-map",
    "/api/v1/missions/status",
    "/api/v1/gpio",
];

const SAFE_OPS: &[&str] = &[
    "health_check",
    "config_show",
    "inspect_stack",
    "read_imu",
    "read_encoders",
    "read_battery",
    "read_all",
    "read_lidar",
    "gripper_status",
    "list_trajectories",
    "stop",
    "stop_all",
];

#[derive(Debug, Clone)]
pub enum Command {
    Status,
    Get {
        path: String,
    },
    Tool {
        operation: String,
        param1: Option<String>,
        param2: Option<String>,
        param3: Option<String>,
        confirm: bool,
    },
    BridgeEvent {
        title: String,
        summary: Option<String>,
    },
    BridgeSpeak {
        text: String,
    },
    BridgeWebhook {
        url: String,
        title: String,
        summary: Option<String>,
    },
    LlmModels {
        provider: Option<String>,
        base_url: Option<String>,
    },
    LlmChat {
        provider: Option<String>,
        base_url: Option<String>,
        model: Option<String>,
        api_key: Option<String>,
        text: String,
    },
}

impl Command {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::Get { .. } => "get",
            Self::Tool { .. } => "tool",
            Self::BridgeEvent { .. } => "bridge.event",
            Self::BridgeSpeak { .. } => "bridge.speak",
            Self::BridgeWebhook { .. } => "bridge.webhook",
            Self::LlmModels { .. } => "llm.models",
            Self::LlmChat { .. } => "llm.chat",
        }
    }

    fn is_read(&self) -> bool {
        match self {
            Self::Status | Self::Get { .. } | Self::LlmModels { .. } => true,
            Self::Tool { operation, .. } => !requires_confirm(operation),
            _ => false,
        }
    }
}

pub fn requires_confirm(operation: &str) -> bool {
    let op = operation.trim().to_ascii_lowercase();
    !SAFE_OPS.contains(&op.as_str())
}

pub fn tool_command(
    operation: &str,
    param1: Option<String>,
    param2: Option<String>,
    param3: Option<String>,
    confirm: bool,
) -> Result<Command, DeviceError> {
    let operation = operation.trim().to_ascii_lowercase();
    if operation.is_empty() {
        return Err(DeviceError::InvalidInput(
            "operation is required".to_string(),
        ));
    }
    if requires_confirm(&operation) && !confirm {
        return Err(DeviceError::InvalidInput(format!(
            "operation '{operation}' can move the robot or fire a payload; pass confirm=true"
        )));
    }
    Ok(Command::Tool {
        operation,
        param1,
        param2,
        param3,
        confirm,
    })
}

pub async fn dispatch(cmd: Command) -> Result<Value, DeviceError> {
    dispatch_with(&load_settings(), cmd).await
}

pub async fn dispatch_with(settings: &Settings, cmd: Command) -> Result<Value, DeviceError> {
    match run_live(settings, cmd.clone()).await {
        Ok(value) => Ok(value),
        Err(err @ (DeviceError::Offline(_) | DeviceError::Timeout(_))) if cmd.is_read() => {
            Ok(demo_payload(&cmd, &err.to_string(), settings))
        }
        Err(err) => Err(err),
    }
}

async fn run_live(settings: &Settings, cmd: Command) -> Result<Value, DeviceError> {
    match cmd {
        Command::Status => {
            let body = Gateway::new(settings)?.get_json("/api/v1/health").await?;
            Ok(json!({
                "ok": true,
                "mode": "live",
                "gateway": settings.gateway_url,
                "robot_ip": settings.robot_ip,
                "health": body,
            }))
        }
        Command::Get { path } => {
            if !READ_PATHS.contains(&path.as_str()) {
                return Err(DeviceError::InvalidInput(format!(
                    "path '{path}' is not on the read allowlist"
                )));
            }
            let body = Gateway::new(settings)?.get_json(&path).await?;
            Ok(json!({"ok": true, "mode": "live", "path": path, "body": body}))
        }
        Command::Tool {
            operation,
            param1,
            param2,
            param3,
            confirm: _,
        } => {
            let body = Gateway::new(settings)?
                .post_tool(
                    &operation,
                    param1.as_deref(),
                    param2.as_deref(),
                    param3.as_deref(),
                )
                .await?;
            Ok(json!({"ok": true, "mode": "live", "operation": operation, "body": body}))
        }
        Command::BridgeEvent { title, summary } => {
            crate::bridge::aiwatcher_event(None, &title, summary, None).await
        }
        Command::BridgeSpeak { text } => crate::bridge::speak(None, &text, None).await,
        Command::BridgeWebhook {
            url,
            title,
            summary,
        } => crate::bridge::webhook(&url, &title, summary).await,
        Command::LlmModels { provider, base_url } => {
            let names = crate::llm::models(provider, base_url, None).await?;
            Ok(json!({"ok": true, "models": names}))
        }
        Command::LlmChat {
            provider,
            base_url,
            model,
            api_key,
            text,
        } => {
            let reply = crate::llm::chat(
                provider,
                base_url,
                model,
                api_key,
                vec![crate::llm::ChatMessage {
                    role: "user".to_string(),
                    content: text,
                }],
            )
            .await?;
            Ok(json!({"ok": true, "reply": reply}))
        }
    }
}

fn demo_payload(cmd: &Command, reason: &str, settings: &Settings) -> Value {
    json!({
        "ok": true,
        "mode": "demo",
        "reason": reason,
        "command": cmd.name(),
        "gateway": settings.gateway_url,
        "robot_ip": settings.robot_ip,
        "health": {
            "status": "offline",
            "robot_connection": {
                "ros": "disconnected",
                "video": "inactive",
                "ssh": "disconnected",
                "ip": settings.robot_ip,
                "cmd_vel_ready": false
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forward_requires_confirm_and_stop_does_not() {
        assert!(requires_confirm("forward"));
        assert!(requires_confirm("Forward"));
        assert!(!requires_confirm("health_check"));
        assert!(!requires_confirm("stop_all"));
        assert!(tool_command("forward", None, None, None, false).is_err());
        assert!(tool_command("health_check", None, None, None, false).is_ok());
    }

    #[test]
    fn dispatch_future_is_send() {
        fn assert_send<T: Send>(_: T) {}
        assert_send(dispatch(Command::Status));
    }

    #[tokio::test]
    async fn offline_status_is_demo_not_a_crash() {
        let settings = Settings {
            gateway_url: "http://127.0.0.1:1".to_string(),
            robot_ip: "192.168.1.11".to_string(),
            origin: "test".to_string(),
        };
        let value = match dispatch_with(&settings, Command::Status).await {
            Ok(value) => value,
            Err(err) => json!({"ok": false, "error": err.to_string()}),
        };
        assert_eq!(value.get("mode").and_then(|v| v.as_str()), Some("demo"));
        assert_eq!(value.get("ok").and_then(|v| v.as_bool()), Some(true));
    }
}
