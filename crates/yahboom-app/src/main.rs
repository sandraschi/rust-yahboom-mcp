//! L3 desktop entry. `--smoke` checks status plus the 28-page inventory.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

mod app;
mod registry;
mod vault;

fn install_forensic_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let msg = format!("[PANIC in yahboom-app]: {info}\n");
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

fn main() -> eframe::Result<()> {
    install_forensic_panic_hook();
    if std::env::args().any(|arg| arg == "--smoke") {
        return app::smoke();
    }
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => Arc::new(runtime),
        Err(err) => {
            eprintln!("failed to start async runtime: {err}");
            std::process::exit(1);
        }
    };
    let viewport = eframe::egui::ViewportBuilder::default()
        .with_title("BOOMY CONTROL")
        .with_inner_size([1280.0, 860.0])
        .with_min_inner_size([960.0, 640.0]);
    eframe::run_native(
        "BOOMY CONTROL",
        eframe::NativeOptions {
            viewport,
            ..Default::default()
        },
        Box::new(move |ctx| Ok(Box::new(app::YahboomApp::new(ctx, runtime)))),
    )
}
