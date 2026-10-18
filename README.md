# rust-yahboom-mcp

Pure-Rust client for the Yahboom gateway that `yahboom-mcp` already runs on port 10892. The legacy repo is not modified.

```
cargo run -p rust-yahboom-mcp -- status --json
cargo run -p rust-yahboom-mcp -- tool health_check --json
cargo run -p rust-yahboom-mcp -- mcp-stdio
cargo run -p yahboom-app -- --smoke
```

Actuator operations refuse to run unless `--confirm` is set. `stop` and `stop_all` are the exceptions. If the gateway is down, read commands return `mode: demo` instead of crashing.
