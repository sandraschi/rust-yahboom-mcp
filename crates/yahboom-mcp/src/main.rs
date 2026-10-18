//! L2 CLI and MCP 2024-11-05 stdio server.
//! stdout carries JSON-RPC frames only while `mcp-stdio` is running.
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unimplemented)]
#![deny(clippy::todo)]

use clap::{Parser, Subcommand};
use serde_json::{json, Value};
use std::io::Write;
use std::path::PathBuf;
use tokio::io::AsyncBufReadExt;
use yahboom_core::{dispatch, tool_command, Command, DeviceError};

#[derive(Parser)]
#[command(
    name = "rust-yahboom-mcp",
    version,
    about = "Yahboom gateway, CLI and MCP"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Status {
        #[arg(long)]
        json: bool,
    },
    Health {
        #[arg(long)]
        json: bool,
    },
    Sensors {
        #[arg(long)]
        json: bool,
    },
    Tool {
        operation: String,
        #[arg(long)]
        param1: Option<String>,
        #[arg(long)]
        param2: Option<String>,
        #[arg(long)]
        param3: Option<String>,
        #[arg(long)]
        confirm: bool,
        #[arg(long)]
        json: bool,
    },
    McpStdio,
}

fn install_forensic_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("[PANIC in rust-yahboom-mcp]: {info}\n");
        let _ = std::io::stderr().write_all(msg.as_bytes());
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let dir = PathBuf::from(local).join("rust-yahboom-mcp");
            let _ = std::fs::create_dir_all(&dir);
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(dir.join("crash.log"))
            {
                let _ = file.write_all(msg.as_bytes());
            }
        }
    }));
}

fn print_value(value: &Value, as_json: bool) {
    if as_json {
        println!("{value}");
        return;
    }
    let mode = value.get("mode").and_then(|v| v.as_str()).unwrap_or("live");
    let command = value
        .get("command")
        .and_then(|v| v.as_str())
        .unwrap_or("status");
    println!("{command} mode={mode}");
}

fn path_command(path: &str) -> Command {
    Command::Get {
        path: path.to_string(),
    }
}

async fn run_cli(command: Commands) -> Result<(), DeviceError> {
    match command {
        Commands::Status { json } => {
            print_value(&dispatch(Command::Status).await?, json);
        }
        Commands::Health { json } => {
            print_value(&dispatch(path_command("/api/v1/health")).await?, json);
        }
        Commands::Sensors { json } => {
            print_value(&dispatch(path_command("/api/v1/sensors")).await?, json);
        }
        Commands::Tool {
            operation,
            param1,
            param2,
            param3,
            confirm,
            json,
        } => {
            let cmd = tool_command(&operation, param1, param2, param3, confirm)?;
            print_value(&dispatch(cmd).await?, json);
        }
        Commands::McpStdio => run_mcp_stdio().await?,
    }
    Ok(())
}

pub fn tool_catalog() -> Vec<Value> {
    let mut tools = vec![
        tool_schema("yahboom.status", "Gateway and robot health.", None),
        tool_schema("yahboom.health", "GET /api/v1/health", None),
        tool_schema("yahboom.sensors", "GET /api/v1/sensors", None),
        tool_schema("yahboom.telemetry", "GET /api/v1/telemetry", None),
        tool_schema("yahboom.logs", "Recent server logs", None),
        tool_schema("yahboom.stack", "Diagnostic stack", None),
        tool_schema("yahboom.slam_map", "SLAM map JSON", None),
        tool_schema("yahboom.lidar_map", "Dreame lidar map JSON", None),
        tool_schema("yahboom.missions", "Mission status", None),
        tool_schema("yahboom.gpio", "GPIO status", None),
        tool_schema(
            "yahboom.tool",
            "Legacy yahboom_tool. Actuators need confirm=true.",
            Some(tool_params()),
        ),
        tool_schema("yahboom.llm_models", "List models for a provider.", None),
        tool_schema(
            "yahboom_tool",
            "Legacy portmanteau alias of yahboom.tool",
            Some(tool_params()),
        ),
        tool_schema(
            "yahboom_demo",
            "Legacy demo alias. Pass operation.",
            Some(tool_params()),
        ),
        tool_schema(
            "lidar",
            "Legacy lidar alias. Read unless confirm=true.",
            Some(tool_params()),
        ),
        tool_schema("ros_topic_list", "Legacy ROS topic list.", None),
        tool_schema(
            "ros_node_info",
            "Legacy ROS node info via stack inspect.",
            None,
        ),
        tool_schema(
            "ros_resync",
            "Legacy resync. Requires confirm=true.",
            Some(tool_params()),
        ),
        tool_schema(
            "ros_restart_bringup",
            "Legacy bringup restart. Requires confirm=true.",
            Some(tool_params()),
        ),
        tool_schema("yahboom_help_tool", "List this catalog.", None),
    ];
    tools.extend([
        tool_schema("yahboom.bridge_event", "Fleet event ingest.", None),
        tool_schema("yahboom.bridge_speak", "Speak text via speech-mcp.", None),
        tool_schema("yahboom.bridge_webhook", "POST a webhook.", None),
        tool_schema(
            "yahboom.llm_chat",
            "One chat turn. Default model gemma4:12b.",
            None,
        ),
    ]);
    tools
}

fn tool_params() -> Value {
    json!({
        "type": "object",
        "properties": {
            "operation": {"type": "string"},
            "param1": {"type": "string"},
            "param2": {"type": "string"},
            "param3": {"type": "string"},
            "confirm": {"type": "boolean"}
        }
    })
}

fn tool_schema(name: &str, description: &str, schema: Option<Value>) -> Value {
    json!({
        "name": name,
        "description": description,
        "inputSchema": schema.unwrap_or_else(|| json!({"type": "object", "properties": {}}))
    })
}

pub fn command_from_tool(name: &str, args: &Value) -> Result<Command, DeviceError> {
    let op = arg_str(args, "operation");
    let p1 = arg_str(args, "param1");
    let p2 = arg_str(args, "param2");
    let p3 = arg_str(args, "param3");
    let confirm = args
        .get("confirm")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    match name {
        "yahboom.status" | "yahboom_status" => Ok(Command::Status),
        "yahboom.health" | "yahboom_health" => Ok(path_command("/api/v1/health")),
        "yahboom.sensors" | "yahboom_sensors" => Ok(path_command("/api/v1/sensors")),
        "yahboom.telemetry" | "yahboom_telemetry" => Ok(path_command("/api/v1/telemetry")),
        "yahboom.logs" | "yahboom_logs" => Ok(path_command("/api/v1/diagnostics/logs")),
        "yahboom.stack" | "yahboom_stack" => Ok(path_command("/api/v1/diagnostics/stack")),
        "yahboom.slam_map" | "yahboom_slam_map" => Ok(path_command("/api/v1/slam/map")),
        "yahboom.lidar_map" | "yahboom_lidar_map" => Ok(path_command("/api/v1/lidar/dreame-map")),
        "yahboom.missions" | "yahboom_missions" => Ok(path_command("/api/v1/missions/status")),
        "yahboom.gpio" | "yahboom_gpio" => Ok(path_command("/api/v1/gpio")),
        "yahboom.tool" | "yahboom_tool" => {
            tool_command(op.as_deref().unwrap_or("health_check"), p1, p2, p3, confirm)
        }
        "yahboom_demo" => {
            tool_command(op.as_deref().unwrap_or("health_check"), p1, p2, p3, confirm)
        }
        "lidar" => tool_command("read_lidar", p1, p2, p3, false),
        "ros_topic_list" => Ok(path_command("/api/v1/diagnostics/ros/topics")),
        "ros_node_info" | "ros_resync" => tool_command(
            "inspect_stack",
            p1,
            p2,
            p3,
            confirm || name == "ros_node_info",
        ),
        "ros_restart_bringup" => tool_command("execute_command", p1, p2, p3, confirm),
        "yahboom_help_tool" => Ok(Command::Status),
        "yahboom.llm_models" | "yahboom_llm_models" => Ok(Command::LlmModels {
            provider: op,
            base_url: p1,
        }),
        "yahboom.llm_chat" | "yahboom_llm_chat" => Ok(Command::LlmChat {
            provider: op,
            base_url: p1,
            model: p2,
            api_key: None,
            text: p3.unwrap_or_else(|| "status".to_string()),
        }),
        "yahboom.bridge_event" | "yahboom_bridge_event" => Ok(Command::BridgeEvent {
            title: op.unwrap_or_else(|| "yahboom".to_string()),
            summary: p1,
        }),
        "yahboom.bridge_speak" | "yahboom_bridge_speak" => Ok(Command::BridgeSpeak {
            text: op.unwrap_or_default(),
        }),
        "yahboom.bridge_webhook" | "yahboom_bridge_webhook" => Ok(Command::BridgeWebhook {
            url: op.unwrap_or_default(),
            title: p1.unwrap_or_else(|| "yahboom".to_string()),
            summary: p2,
        }),
        _ => Err(DeviceError::InvalidInput(format!("unknown tool '{name}'"))),
    }
}

fn arg_str(args: &Value, key: &str) -> Option<String> {
    args.get(key).and_then(|v| match v {
        Value::String(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    })
}

async fn run_mcp_stdio() -> Result<(), DeviceError> {
    let stdin = tokio::io::stdin();
    let reader = tokio::io::BufReader::new(stdin);
    let mut lines = reader.lines();
    let mut stdout = std::io::stdout();
    loop {
        let line = match lines.next_line().await {
            Ok(Some(line)) => line,
            Ok(None) => break,
            Err(err) => return Err(DeviceError::Protocol(format!("stdin: {err}"))),
        };
        if line.trim().is_empty() {
            continue;
        }
        let message: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(_) => {
                write_frame(
                    &mut stdout,
                    &json!({"jsonrpc":"2.0","id": Value::Null, "error": {"code": -32700, "message": "Parse error"}}),
                )?;
                continue;
            }
        };
        if message.get("id").is_none() {
            continue;
        }
        let id = message.get("id").cloned().unwrap_or(Value::Null);
        let method = message.get("method").and_then(|v| v.as_str()).unwrap_or("");
        let response = match method {
            "initialize" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {"tools": {}},
                    "serverInfo": {"name": "rust-yahboom-mcp", "version": "0.1.0"}
                }
            }),
            "tools/list" => json!({"jsonrpc":"2.0","id": id, "result": {"tools": tool_catalog()}}),
            "tools/call" => {
                let params = message.get("params").cloned().unwrap_or_else(|| json!({}));
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let args = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                match command_from_tool(name, &args) {
                    Ok(cmd) => match dispatch(cmd).await {
                        Ok(value) => rpc_ok(id, &value),
                        Err(err) => rpc_tool_error(id, &err.to_string()),
                    },
                    Err(err) => rpc_tool_error(id, &err.to_string()),
                }
            }
            _ => json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {"code": -32601, "message": format!("unknown method {method}")}
            }),
        };
        write_frame(&mut stdout, &response)?;
    }
    Ok(())
}

fn rpc_ok(id: Value, value: &Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {"content": [{"type": "text", "text": value.to_string()}], "isError": false}
    })
}

fn rpc_tool_error(id: Value, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {"content": [{"type": "text", "text": message}], "isError": true}
    })
}

fn write_frame(stdout: &mut impl Write, value: &Value) -> Result<(), DeviceError> {
    writeln!(stdout, "{value}").map_err(|err| DeviceError::Protocol(format!("stdout: {err}")))?;
    stdout
        .flush()
        .map_err(|err| DeviceError::Protocol(format!("stdout flush: {err}")))
}

fn main() {
    install_forensic_panic_hook();
    let cli = Cli::parse();
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(err) => {
            eprintln!("failed to start async runtime: {err}");
            std::process::exit(1);
        }
    };
    if let Err(err) = runtime.block_on(run_cli(cli.command)) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_lists_the_legacy_aliases() {
        let catalog = tool_catalog();
        let names: Vec<&str> = catalog
            .iter()
            .filter_map(|tool| tool.get("name").and_then(|v| v.as_str()))
            .collect();
        assert!(names.contains(&"yahboom.status"));
        assert!(names.contains(&"yahboom_tool"));
        assert!(names.contains(&"lidar"));
        assert!(names.len() >= 20);
    }

    #[test]
    fn unknown_tool_becomes_error_content_not_panic() {
        let message = match command_from_tool("not_a_tool", &json!({})) {
            Err(err) => err.to_string(),
            Ok(_) => "unexpected ok".to_string(),
        };
        let frame = rpc_tool_error(json!(1), &message);
        assert_eq!(frame["result"]["isError"], true);
        assert!(message.contains("unknown tool"));
    }

    #[test]
    fn legacy_portmanteau_alias_routes_correctly() {
        let operation =
            match command_from_tool("yahboom_tool", &json!({"operation": "health_check"})) {
                Ok(Command::Tool { operation, .. }) => operation,
                Ok(_) => "other".to_string(),
                Err(err) => err.to_string(),
            };
        assert_eq!(operation, "health_check");
    }

    #[test]
    fn actuator_without_confirm_is_rejected() {
        let message = match command_from_tool("yahboom_tool", &json!({"operation": "forward"})) {
            Err(err) => err.to_string(),
            Ok(_) => "unexpected ok".to_string(),
        };
        assert!(message.contains("confirm=true"));
    }

    #[tokio::test]
    async fn demo_round_trip() {
        let value = match dispatch(Command::Status).await {
            Ok(value) => value,
            Err(err) => json!({"ok": false, "error": err.to_string()}),
        };
        assert_eq!(value.get("ok").and_then(|v| v.as_bool()), Some(true));
    }
}
