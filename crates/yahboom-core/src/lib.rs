//! L1 Yahboom core: local gateway client, command bus, fleet bridge and LLM.
//! UI and MCP stay out of this crate. Hardware writes require `confirm`.
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unimplemented)]
#![deny(clippy::todo)]

pub mod bridge;
pub mod command;
pub mod creds;
pub mod error;
pub mod gateway;
pub mod import;
pub mod llm;

pub use command::{dispatch, dispatch_with, requires_confirm, tool_command, Command};
pub use error::DeviceError;
pub use import::Settings;
