//! Boomy Control desktop. One page per legacy route, in the legacy order.
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unimplemented)]
#![deny(clippy::todo)]

use eframe::egui::{self, Color32, RichText};
use serde_json::Value;
use std::sync::{mpsc, Arc};
use yahboom_core::{dispatch, Command};

const BG: Color32 = Color32::from_rgb(10, 10, 12);
const PANEL: Color32 = Color32::from_rgb(18, 18, 22);
const ACCENT: Color32 = Color32::from_rgb(99, 102, 241);
const DANGER: Color32 = Color32::from_rgb(220, 38, 38);
const MUTED: Color32 = Color32::from_rgb(148, 163, 184);

#[derive(Clone, Copy, PartialEq, Eq)]
enum LoadState {
    Unloaded,
    Loading,
    Loaded,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Dashboard,
    MissionControl,
    Status,
    Analytics,
    Visualization,
    Map,
    Slam,
    LidarMap,
    Movement,
    Sensors,
    Peripherals,
    Audio,
    Voice,
    VoiceUpgrade,
    LidarAddon,
    Companion,
    Missions,
    Llm,
    Skills,
    Workflows,
    Diagnostics,
    Inbox,
    Logs,
    Tools,
    Apps,
    Settings,
    Onboarding,
    Help,
    Crossconnect,
    Raw,
}

impl Page {
    fn label(self) -> &'static str {
        match self {
            Self::Dashboard => "Dashboard",
            Self::MissionControl => "Mission Control",
            Self::Status => "Status",
            Self::Analytics => "Analytics",
            Self::Visualization => "Visualization",
            Self::Map => "Map",
            Self::Slam => "SLAM Map",
            Self::LidarMap => "Lidar Map",
            Self::Movement => "Movement",
            Self::Sensors => "Sensors",
            Self::Peripherals => "Peripherals",
            Self::Audio => "Audio Soundboard",
            Self::Voice => "Voice & Audio",
            Self::VoiceUpgrade => "Voice Upgrade",
            Self::LidarAddon => "Lidar Addon",
            Self::Companion => "AI Companion",
            Self::Missions => "Missions",
            Self::Llm => "Local LLM",
            Self::Skills => "Skills",
            Self::Workflows => "Workflows",
            Self::Diagnostics => "Diagnostic Hub",
            Self::Inbox => "Event Inbox",
            Self::Logs => "Server logs",
            Self::Tools => "MCP Tools",
            Self::Apps => "Apps Hub",
            Self::Settings => "Settings",
            Self::Onboarding => "Onboarding",
            Self::Help => "Help",
            Self::Crossconnect => "Crossconnect",
            Self::Raw => "Raw JSON",
        }
    }
}

struct Group {
    title: &'static str,
    pages: &'static [Page],
}

const GROUPS: &[Group] = &[
    Group {
        title: "OPERATIONS",
        pages: &[
            Page::Dashboard,
            Page::MissionControl,
            Page::Status,
            Page::Analytics,
            Page::Visualization,
            Page::Map,
            Page::Slam,
            Page::LidarMap,
        ],
    },
    Group {
        title: "CHASSIS & PAYLOAD",
        pages: &[
            Page::Movement,
            Page::Sensors,
            Page::Peripherals,
            Page::Audio,
            Page::Voice,
            Page::VoiceUpgrade,
            Page::LidarAddon,
        ],
    },
    Group {
        title: "INTELLIGENCE",
        pages: &[
            Page::Companion,
            Page::Missions,
            Page::Llm,
            Page::Skills,
            Page::Workflows,
        ],
    },
    Group {
        title: "SYSTEM HUB",
        pages: &[
            Page::Diagnostics,
            Page::Inbox,
            Page::Logs,
            Page::Tools,
            Page::Apps,
            Page::Settings,
            Page::Onboarding,
            Page::Help,
        ],
    },
    Group {
        title: "KERNEL",
        pages: &[Page::Crossconnect, Page::Raw],
    },
];

pub fn legacy_page_count() -> usize {
    GROUPS.iter().map(|group| group.pages.len()).sum::<usize>() - 2
}

struct Card {
    title: &'static str,
    body: &'static str,
    operation: Option<&'static str>,
}

const MOVEMENT_CARDS: &[Card] = &[
    Card {
        title: "Forward",
        body: "Drive forward 1.5 s",
        operation: Some("forward"),
    },
    Card {
        title: "Backward",
        body: "Drive backward 1.5 s",
        operation: Some("backward"),
    },
    Card {
        title: "Turn left 90°",
        body: "Rotate left ~90°",
        operation: Some("turn_left"),
    },
    Card {
        title: "Turn right 90°",
        body: "Rotate right ~90°",
        operation: Some("turn_right"),
    },
    Card {
        title: "Spin left",
        body: "Full rotation left",
        operation: Some("turn_left"),
    },
    Card {
        title: "Spin right",
        body: "Full rotation right",
        operation: Some("turn_right"),
    },
    Card {
        title: "Small square",
        body: "Forward + turn x4",
        operation: Some("forward"),
    },
    Card {
        title: "Square patrol",
        body: "One full square (4 sides)",
        operation: Some("forward"),
    },
    Card {
        title: "Figure-8",
        body: "Approx figure-8 with arcs",
        operation: Some("forward"),
    },
    Card {
        title: "Back and forth",
        body: "Forward, turn 180, repeat",
        operation: Some("forward"),
    },
    Card {
        title: "Wiggle",
        body: "Quick left-right sway",
        operation: Some("turn_left"),
    },
    Card {
        title: "Box step",
        body: "Forward, strafe, back, strafe",
        operation: Some("strafe_left"),
    },
    Card {
        title: "Spin dance",
        body: "Two full rotations",
        operation: Some("turn_left"),
    },
    Card {
        title: "Shuffle",
        body: "Small forward-back pulse",
        operation: Some("forward"),
    },
];
const PERIPHERAL_CARDS: &[Card] = &[
    Card {
        title: "Camera PTZ",
        body: "Pan ID 1, tilt ID 2",
        operation: Some("camera_reset"),
    },
    Card {
        title: "GPIO",
        body: "Headlight LED",
        operation: None,
    },
    Card {
        title: "Tapo Audio",
        body: "Type to speak",
        operation: Some("say"),
    },
    Card {
        title: "Light Control",
        body: "Patrol car, danger red, safe green",
        operation: Some("patrol_car"),
    },
];
const AUDIO_CARDS: &[Card] = &[
    Card {
        title: "Sound Check",
        body: "Stored clip",
        operation: Some("audio_list_stored"),
    },
    Card {
        title: "Voice Beep",
        body: "Short beep",
        operation: Some("play_beep"),
    },
];
const VOICE_CARDS: &[Card] = &[
    Card {
        title: "Say",
        body: "Speak the text field",
        operation: Some("say"),
    },
    Card {
        title: "Listen",
        body: "Tapo listen",
        operation: None,
    },
];
const COMPANION_CARDS: &[Card] = &[
    Card {
        title: "Move forward 0.5 meters",
        body: "Chat shortcut",
        operation: Some("forward"),
    },
    Card {
        title: "Rotate 90 degrees left",
        body: "Chat shortcut",
        operation: Some("turn_left"),
    },
    Card {
        title: "Emergency stop",
        body: "Halt",
        operation: Some("stop_all"),
    },
];
const VIZ_CARDS: &[Card] = &[Card {
    title: "3D Visualization",
    body: "v0.1 pre-alpha. Left drag orbit. Scroll zoom. Right drag pan.",
    operation: None,
}];
const ONBOARD_CARDS: &[Card] = &[Card {
    title: "Hardware bring-up",
    body: "Power the Raspbot, join its Wi-Fi or use Ethernet, then reconnect.",
    operation: None,
}];
const HELP_CARDS: &[Card] = &[Card {
    title: "Help",
    body: "Gateway on 10892. Webapp on 10893. Actuators need a confirm click.",
    operation: None,
}];

fn cards_for(page: Page) -> &'static [Card] {
    match page {
        Page::Movement => MOVEMENT_CARDS,
        Page::Peripherals => PERIPHERAL_CARDS,
        Page::Audio => AUDIO_CARDS,
        Page::Voice => VOICE_CARDS,
        Page::Companion => COMPANION_CARDS,
        Page::Visualization => VIZ_CARDS,
        Page::Onboarding => ONBOARD_CARDS,
        Page::Help => HELP_CARDS,
        _ => &[],
    }
}

struct Job {
    page_hint: String,
    value: Value,
}

pub struct YahboomApp {
    page: Page,
    load: LoadState,
    health: Option<Value>,
    last: Option<Value>,
    note: String,
    pending: Option<(&'static str, &'static str)>,
    provider: String,
    base_url: String,
    model: String,
    api_key: String,
    speak: String,
    light: bool,
    rt: Arc<tokio::runtime::Runtime>,
    tx: mpsc::Sender<Job>,
    rx: mpsc::Receiver<Job>,
}

impl YahboomApp {
    pub fn new(cc: &eframe::CreationContext<'_>, rt: Arc<tokio::runtime::Runtime>) -> Self {
        apply_theme(&cc.egui_ctx, false);
        let (tx, rx) = mpsc::channel();
        Self {
            page: Page::Dashboard,
            load: LoadState::Unloaded,
            health: None,
            last: None,
            note: String::new(),
            pending: None,
            provider: "ollama".to_string(),
            base_url: "http://127.0.0.1:11434/v1".to_string(),
            model: "gemma4:12b".to_string(),
            api_key: String::new(),
            speak: String::new(),
            light: false,
            rt,
            tx,
            rx,
        }
    }

    fn spawn(&self, hint: &str, cmd: Command) {
        let tx = self.tx.clone();
        let hint = hint.to_string();
        self.rt.spawn(async move {
            let value = match dispatch(cmd).await {
                Ok(value) => value,
                Err(err) => serde_json::json!({"ok": false, "error": err.to_string()}),
            };
            let _ = tx.send(Job {
                page_hint: hint,
                value,
            });
        });
    }

    fn link_up(&self) -> bool {
        self.health
            .as_ref()
            .and_then(|value| value.pointer("/health/robot_connection/ros"))
            .and_then(|value| value.as_str())
            == Some("connected")
    }

    fn mode(&self) -> &str {
        self.health
            .as_ref()
            .and_then(|value| value.get("mode"))
            .and_then(|value| value.as_str())
            .unwrap_or("offline")
    }
}

impl eframe::App for YahboomApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let zoom_delta = ctx.input(|input| input.zoom_delta());
        if (zoom_delta - 1.0).abs() > 0.001 {
            let next = (ctx.zoom_factor() * zoom_delta).clamp(0.7, 2.0);
            ctx.set_zoom_factor(next);
        }
        while let Ok(job) = self.rx.try_recv() {
            if job.page_hint == "health" {
                self.health = Some(job.value);
                self.load = LoadState::Loaded;
            } else {
                self.note = job.page_hint;
                self.last = Some(job.value);
            }
        }
        if self.load == LoadState::Unloaded {
            self.load = LoadState::Loading;
            self.spawn("health", Command::Status);
        }
        egui::Panel::top("top").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("BOOMY SYSTEM CORE").color(MUTED));
                let link = if self.link_up() {
                    "LINK ACTIVE"
                } else {
                    "LINK LOST"
                };
                let color = if self.link_up() {
                    Color32::from_rgb(34, 197, 94)
                } else {
                    DANGER
                };
                ui.label(RichText::new(link).color(color));
                if ui
                    .button(if self.light { "Dark" } else { "Light" })
                    .clicked()
                {
                    self.light = !self.light;
                    apply_theme(&ctx, self.light);
                }
                if ui
                    .button(RichText::new("EMERGENCY STOP").color(Color32::WHITE))
                    .clicked()
                {
                    self.spawn("stop_all", tool("stop_all"));
                }
            });
        });
        egui::Panel::left("nav").default_size(230.0).show(ui, |ui| {
            ui.label(
                RichText::new("BOOMY CONTROL")
                    .strong()
                    .color(Color32::WHITE),
            );
            ui.label(RichText::new("INDUSTRIAL CONTROL").size(10.0).color(MUTED));
            egui::ScrollArea::vertical().show(ui, |ui| {
                for group in GROUPS {
                    ui.add_space(8.0);
                    ui.label(RichText::new(group.title).size(11.0).color(MUTED));
                    for page in group.pages {
                        let selected = *page == self.page;
                        let text = if selected {
                            RichText::new(page.label()).color(Color32::WHITE)
                        } else {
                            RichText::new(page.label()).color(MUTED)
                        };
                        if ui.selectable_label(selected, text).clicked() {
                            self.page = *page;
                            self.pending = None;
                        }
                    }
                }
            });
        });
        egui::CentralPanel::default().show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.view_page(ui);
            });
        });
    }
}

impl YahboomApp {
    fn view_page(&mut self, ui: &mut egui::Ui) {
        ui.heading(self.page.label());
        ui.label(RichText::new(format!("mode {}", self.mode())).color(MUTED));
        if !self.note.is_empty() {
            ui.label(RichText::new(&self.note).color(ACCENT));
        }
        match self.page {
            Page::Dashboard => self.view_dashboard(ui),
            Page::Settings => self.view_settings(ui),
            Page::Movement => self.view_cards(ui, "PRESET MOVEMENTS"),
            Page::Raw => {
                ui.monospace(
                    self.last
                        .as_ref()
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "{}".into()),
                );
            }
            Page::Crossconnect => {
                if ui.button("Send fleet event").clicked() {
                    self.spawn(
                        "bridge.event",
                        Command::BridgeEvent {
                            title: "yahboom".to_string(),
                            summary: Some("desktop ping".to_string()),
                        },
                    );
                }
            }
            _ => self.view_cards(ui, self.page.label()),
        }
        if let Some(pending) = self.pending {
            ui.separator();
            ui.label(format!("Confirm {} ({})", pending.0, pending.1));
            if ui.button("Confirm drive").clicked() {
                self.spawn(pending.0, tool(pending.1));
                self.pending = None;
            }
        }
    }

    fn view_dashboard(&mut self, ui: &mut egui::Ui) {
        let ip = self
            .health
            .as_ref()
            .and_then(|v| v.get("robot_ip"))
            .and_then(|v| v.as_str())
            .unwrap_or("192.168.1.11");
        if self.link_up() {
            ui.label("Robot link is up.");
        } else {
            ui.label(RichText::new("No connection to robot").color(DANGER));
            ui.label(format!(
                "Target {ip}. ROS disconnected until the Pi answers."
            ));
        }
        ui.add_space(8.0);
        ui.label(RichText::new("CONTROL CENTER").strong());
        ui.label("SOTA | ROS 2 HUMBLE | VIENNA_ALSERGRUND");
        if ui.button("Reconnect").clicked() {
            self.load = LoadState::Unloaded;
        }
        ui.separator();
        ui.label("Camera PTZ");
        if ui.button("Center").clicked() {
            self.pending = Some(("Center", "camera_reset"));
        }
        ui.label("GPIO headlight");
        ui.label("Tapo Audio");
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.speak);
            if ui.button("Speak").clicked() {
                self.pending = Some(("Speak", "say"));
            }
        });
        ui.label("Keyboard: WASD drives when the link is up");
        ui.label("Light Control: Patrol car pattern, Danger red, Safe green");
        ui.label("Voice & Media Hub");
    }

    fn view_cards(&mut self, ui: &mut egui::Ui, heading: &str) {
        ui.label(RichText::new(heading).color(MUTED));
        let cards = cards_for(self.page);
        if cards.is_empty() {
            ui.label(
                "This page reads the gateway. Use Reconnect on Dashboard, then open Raw JSON.",
            );
            if ui.button("Refresh this page").clicked() {
                let path = path_for(self.page);
                self.spawn(
                    self.page.label(),
                    Command::Get {
                        path: path.to_string(),
                    },
                );
            }
            return;
        }
        let linked = self.link_up();
        for card in cards {
            ui.group(|ui| {
                ui.label(RichText::new(card.title).strong());
                ui.label(RichText::new(card.body).color(MUTED));
                if let Some(operation) = card.operation {
                    let safe = !yahboom_core::requires_confirm(operation);
                    let enabled = linked || safe;
                    if ui
                        .add_enabled(enabled, egui::Button::new(card.title))
                        .clicked()
                    {
                        if safe {
                            self.spawn(card.title, tool(operation));
                        } else {
                            self.pending = Some((card.title, operation));
                        }
                    }
                }
            });
        }
    }

    fn view_settings(&mut self, ui: &mut egui::Ui) {
        ui.label("LLM");
        ui.label("Provider");
        egui::ComboBox::from_id_salt("provider")
            .selected_text(&self.provider)
            .show_ui(ui, |ui| {
                for name in ["ollama", "lmstudio", "openai", "anthropic", "custom"] {
                    ui.selectable_value(&mut self.provider, name.to_string(), name);
                }
            });
        ui.label("Base URL");
        ui.text_edit_singleline(&mut self.base_url);
        ui.label("Model");
        ui.text_edit_singleline(&mut self.model);
        ui.label("API key (OS vault, not a file)");
        ui.add(egui::TextEdit::singleline(&mut self.api_key).password(true));
        if ui.button("Save key").clicked() {
            self.note = match crate::vault::set(crate::vault::LLM_API_KEY, &self.api_key) {
                Ok(()) => format!("stored in {}", crate::vault::SERVICE),
                Err(err) => err,
            };
        }
        if ui.button("Test connection").clicked() {
            self.spawn(
                "llm.models",
                Command::LlmModels {
                    provider: Some(self.provider.clone()),
                    base_url: Some(self.base_url.clone()),
                },
            );
        }
        let registry = crate::registry::load();
        ui.label(format!("Registry entries: {}", registry.devices.len()));
        ui.label(format!("Vault service: {}", crate::vault::SERVICE));
    }
}

fn tool(operation: &str) -> Command {
    yahboom_core::tool_command(operation, None, None, None, true).unwrap_or(Command::Status)
}

fn path_for(page: Page) -> &'static str {
    match page {
        Page::Sensors => "/api/v1/sensors",
        Page::Logs => "/api/v1/diagnostics/logs",
        Page::Diagnostics | Page::Status => "/api/v1/diagnostics/stack",
        Page::Slam => "/api/v1/slam/map",
        Page::LidarMap | Page::Map => "/api/v1/lidar/dreame-map",
        Page::Missions => "/api/v1/missions/status",
        Page::Inbox => "/api/v1/diagnostics/logs",
        _ => "/api/v1/health",
    }
}

fn apply_theme(ctx: &egui::Context, light: bool) {
    let mut visuals = if light {
        egui::Visuals::light()
    } else {
        egui::Visuals::dark()
    };
    if !light {
        visuals.panel_fill = PANEL;
        visuals.window_fill = BG;
        visuals.extreme_bg_color = BG;
    }
    ctx.set_visuals(visuals);
}

pub fn smoke() -> eframe::Result<()> {
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(err) => {
            eprintln!("smoke: runtime init failed: {err}");
            std::process::exit(1);
        }
    };
    let value = runtime.block_on(dispatch(Command::Status));
    let ok = match value {
        Ok(value) => value.get("ok").and_then(|v| v.as_bool()) == Some(true),
        Err(err) => {
            eprintln!("smoke: {err}");
            false
        }
    };
    if ok && legacy_page_count() == 28 {
        println!("smoke: status ok, 28 legacy pages");
        Ok(())
    } else {
        eprintln!("smoke: parity check failed");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glyph_source_scan() {
        const ALLOWED: &[char] = &['°', '×', '·', '…'];
        let source = include_str!("app.rs");
        let mut offenders = Vec::new();
        for (idx, ch) in source.char_indices() {
            if !ch.is_ascii() && !ALLOWED.contains(&ch) {
                offenders.push((idx, ch));
                if offenders.len() >= 8 {
                    break;
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "non-allowlisted glyphs: {offenders:?}"
        );
    }

    #[test]
    fn every_legacy_route_is_a_page() {
        assert_eq!(legacy_page_count(), 28);
        assert_eq!(Page::Dashboard.label(), "Dashboard");
        assert_eq!(Page::Movement.label(), "Movement");
    }
}
