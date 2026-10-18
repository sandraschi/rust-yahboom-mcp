# Conversion Audit Log: rust-yahboom-mcp

## 2026-10-18 Phase 0
- Legacy `D:\Dev\repos\yahboom-mcp` stayed at `58378e3`.
- Captured 28 routes plus scroll frames into `docs/parity/legacy/`.
- Robot link was down (`192.168.1.11`, ROS disconnected). No drive command was sent.

## 2026-10-18 Phase 1-4
- Kernel copied from `rust-devices-mcp`: `bridge.rs`, `llm.rs` (`gemma4:12b`), `error.rs`, calibre `creds.rs`, `vault.rs`, `registry.rs`.
- L1 talks to the running gateway at `http://127.0.0.1:10892`. Reads fall back to a demo payload on timeout or connection failure. Actuators need `confirm=true`. `stop` and `stop_all` do not.
- L2 CLI plus MCP `2024-11-05`, including legacy `yahboom_tool`.
- L3 egui window has the 28 legacy pages in sidebar order, plus Crossconnect and Raw JSON.

## 2026-10-18 Phase 5 local gates
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed.
- `cargo test --workspace`: yahboom-core 17, rust-yahboom-mcp 5, yahboom-app 4.
- `cargo fmt --all --check` passed.
- `cargo run -q -p yahboom-app -- --smoke` printed `smoke: status ok, 28 legacy pages`.
- `cargo run -q -p rust-yahboom-mcp -- status --json` returned `ok: true`, `mode: live`, ROS disconnected, gateway version `2.5.0b1`.
- MCP stdio `initialize`, `tools/list`, and `tools/call yahboom_tool health_check` answered. The tool call returned `isError: true` because `POST /api/v1/control/tool` answered HTTP 500 while the robot link was down. The process did not crash.

## Not done
- Screenshot diff of the egui window against `docs/parity/legacy/`.
- `egui_kittest` goldens. They wait until that diff matches.
- NSIS installer and GitHub release. Not published.
