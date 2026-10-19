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

const ACCENT: Color32 = Color32::from_rgb(79, 70, 229);
const DANGER: Color32 = Color32::from_rgb(185, 28, 28);
const OK: Color32 = Color32::from_rgb(21, 128, 61);

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

    #[cfg(test)]
    fn slug(self) -> &'static str {
        match self {
            Self::Dashboard => "dashboard",
            Self::MissionControl => "mission-control",
            Self::Status => "status",
            Self::Analytics => "analytics",
            Self::Visualization => "viz",
            Self::Map => "map",
            Self::Slam => "slam",
            Self::LidarMap => "lidar-map",
            Self::Movement => "movement",
            Self::Sensors => "sensors",
            Self::Peripherals => "peripherals",
            Self::Audio => "audio",
            Self::Voice => "voice",
            Self::VoiceUpgrade => "voice-upgrade",
            Self::LidarAddon => "lidar-addon",
            Self::Companion => "chat",
            Self::Missions => "missions",
            Self::Llm => "llm",
            Self::Skills => "skills",
            Self::Workflows => "workflows",
            Self::Diagnostics => "diagnostics",
            Self::Inbox => "inbox",
            Self::Logs => "logs",
            Self::Tools => "tools",
            Self::Apps => "apps",
            Self::Settings => "settings",
            Self::Onboarding => "onboarding",
            Self::Help => "help",
            Self::Crossconnect => "crossconnect",
            Self::Raw => "raw",
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

#[derive(Clone)]
struct Pending {
    label: &'static str,
    operation: &'static str,
    p1: Option<String>,
    p2: Option<String>,
    p3: Option<String>,
}

impl Pending {
    fn bare(label: &'static str, operation: &'static str) -> Self {
        Self {
            label,
            operation,
            p1: None,
            p2: None,
            p3: None,
        }
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
    pending: Option<Pending>,
    headlight: bool,
    wasd: bool,
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
        Self::offline(rt)
    }

    fn offline(rt: Arc<tokio::runtime::Runtime>) -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            page: Page::Dashboard,
            load: LoadState::Unloaded,
            health: None,
            last: None,
            note: String::new(),
            pending: None,
            headlight: false,
            wasd: false,
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
        self.draw(ui);
    }
}

impl YahboomApp {
    fn draw(&mut self, ui: &mut egui::Ui) {
        apply_theme(ui.ctx(), self.light);
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
                ui.label(RichText::new("BOOMY SYSTEM CORE").color(secondary(ui)));
                let link = if self.link_up() {
                    "LINK ACTIVE"
                } else {
                    "LINK LOST"
                };
                let color = if self.link_up() {
                    status_ok(ui)
                } else {
                    status_bad(ui)
                };
                ui.label(RichText::new(link).strong().color(color));
                theme_toggle_button(ui, &mut self.light);
                if ui
                    .add(
                        egui::Button::new(RichText::new("EMERGENCY STOP").color(Color32::WHITE))
                            .fill(DANGER),
                    )
                    .clicked()
                {
                    self.spawn("stop_all", tool("stop_all"));
                }
            });
        });
        egui::Panel::left("nav").default_size(230.0).show(ui, |ui| {
            ui.label(RichText::new("BOOMY CONTROL").strong().color(primary(ui)));
            ui.label(
                RichText::new("INDUSTRIAL CONTROL")
                    .size(13.0)
                    .color(secondary(ui)),
            );
            egui::ScrollArea::vertical().show(ui, |ui| {
                for group in GROUPS {
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(group.title)
                            .size(13.0)
                            .strong()
                            .color(secondary(ui)),
                    );
                    for page in group.pages {
                        let selected = *page == self.page;
                        let text = if selected {
                            RichText::new(page.label()).strong().color(primary(ui))
                        } else {
                            RichText::new(page.label()).color(secondary(ui))
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
        if self.page != Page::Dashboard {
            ui.heading(self.page.label());
            ui.label(RichText::new(format!("mode {}", self.mode())).color(secondary(ui)));
        }
        if !self.note.is_empty() {
            ui.label(RichText::new(&self.note).color(primary(ui)));
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
        if let Some(pending) = self.pending.clone() {
            ui.separator();
            ui.label(format!("Confirm {} ({})", pending.label, pending.operation));
            if ui.button("Confirm drive").clicked() {
                let cmd = yahboom_core::tool_command(
                    pending.operation,
                    pending.p1,
                    pending.p2,
                    pending.p3,
                    true,
                )
                .unwrap_or(Command::Status);
                self.spawn(pending.label, cmd);
                self.pending = None;
            }
        }
    }

    fn arm(&mut self, label: &'static str, operation: &'static str) {
        self.pending = Some(Pending::bare(label, operation));
    }

    fn arm_params(
        &mut self,
        label: &'static str,
        operation: &'static str,
        p1: Option<String>,
        p2: Option<String>,
        p3: Option<String>,
    ) {
        self.pending = Some(Pending {
            label,
            operation,
            p1,
            p2,
            p3,
        });
    }

    fn view_dashboard(&mut self, ui: &mut egui::Ui) {
        let snap = dash_snapshot(self.health.as_ref(), self.link_up());
        let dark = ui.visuals().dark_mode;
        let (fill, stroke, ink) = if snap.connected {
            (
                Color32::from_rgb(6, 48, 32),
                Color32::from_rgb(52, 211, 153),
                Color32::from_rgb(209, 250, 229),
            )
        } else if dark {
            (
                Color32::from_rgb(69, 10, 10),
                Color32::from_rgb(248, 113, 113),
                Color32::from_rgb(254, 226, 226),
            )
        } else {
            (
                Color32::from_rgb(254, 226, 226),
                Color32::from_rgb(185, 28, 28),
                Color32::from_rgb(127, 29, 29),
            )
        };
        let mut banner = egui::Frame::group(ui.style());
        banner.fill = fill;
        banner.stroke = egui::Stroke::new(1.0, stroke);
        banner.inner_margin = egui::Margin::same(12);
        banner.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(if snap.connected {
                            "Robot connected"
                        } else {
                            "No connection to robot"
                        })
                        .strong()
                        .size(18.0)
                        .color(ink),
                    );
                    ui.label(
                        RichText::new(format!(
                            "Target {}  |  ROS {}  |  SSH {}  |  Video {}  |  cmd_vel {}",
                            snap.ip, snap.ros, snap.ssh, snap.video, snap.cmd_vel
                        ))
                        .size(13.0)
                        .color(ink),
                    );
                });
                ui.with_layout(egui::Layout::top_down(egui::Align::Max), |ui| {
                    if ui.button("Server logs").clicked() {
                        self.page = Page::Logs;
                    }
                    if ui.button("Diagnostics").clicked() {
                        self.page = Page::Diagnostics;
                    }
                    if ui.button("Help").clicked() {
                        self.page = Page::Help;
                    }
                });
            });
            if !snap.connected && ui.button("New robot? Start hardware bring-up").clicked() {
                self.page = Page::Onboarding;
            }
            ui.add_space(6.0);
            ui.label(
                RichText::new("STACK HEALTH  |  see Status for details")
                    .size(13.0)
                    .color(ink),
            );
            ui.horizontal_wrapped(|ui| {
                for layer in &snap.layers {
                    let mark = if layer.ok { "OK" } else { "X" };
                    let color = if layer.ok { status_ok(ui) } else { status_bad(ui) };
                    ui.label(RichText::new(format!("{mark} {}", layer.title)).color(color));
                }
            });
            if !snap.connected {
                ui.add_space(6.0);
                ui.label(RichText::new("Hey - start with the robot and the link. Turn the Raspbot on and wait for it to boot. Join the Raspbot Wi-Fi access point on this PC, or connect an Ethernet cable between Goliath and the Pi. Until that path exists, Docker, systemd, and Yahboom bringup on the Pi are not in play for this dashboard. After power and network are good, use Reconnect or Diagnostics Hard Reset if ROS still does not attach.").color(ink));
                ui.label(RichText::new("This PC can reach the configured robot address. Both ROS (rosbridge) and SSH are down. The gateway on Goliath is running. The Pi is not participating.").color(ink));
                ui.label(RichText::new("Robot off or booting. Raspbot power / SD / wait for AP or Ethernet to come up.").color(ink));
                ui.label(RichText::new("Goliath not on the robot network. Join Wi-Fi to the Raspbot access point and no Ethernet cable is plugged between Goliath and the Raspberry Pi.").color(ink));
                ui.label(RichText::new("Wrong YAHBOOM_IP in webapp/start.ps1 (or env). It must match the Pi on the link you use (AP vs home LAN vs Ethernet).").color(ink));
            }
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("CONTROL CENTER").strong().size(22.0));
            ui.label(
                RichText::new("SOTA v1.20  |  ROS 2 HUMBLE  |  VIENNA_ALSERGRUND")
                    .color(secondary(ui)),
            );
            let bridge = if snap.connected {
                "ROS BRIDGE LIVE"
            } else {
                "ROS BRIDGE OFFLINE"
            };
            ui.label(RichText::new(bridge).strong().color(if snap.connected {
                status_ok(ui)
            } else {
                status_bad(ui)
            }));
            if ui.button("Reconnect").clicked() {
                self.load = LoadState::Unloaded;
            }
        });
        ui.add_space(8.0);
        ui.columns(3, |cols| {
            kpi(
                cols,
                0,
                "SERVER",
                "yahboom-mcp",
                &format!("v{}", snap.version),
            );
            kpi(cols, 1, "ROBOT", &snap.ip, &format!("ROS {}", snap.ros));
            kpi(
                cols,
                2,
                "TOOLS",
                &snap.layers.len().to_string(),
                "Stack layers",
            );
        });
        ui.add_space(8.0);
        let width = ui.available_width();
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 200.0), egui::Sense::hover());
        ui.painter().rect_filled(
            rect,
            egui::CornerRadius::same(16),
            Color32::from_rgb(8, 10, 16),
        );
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "Visual stream offline. Connect the ROS bridge first.",
            egui::FontId::proportional(16.0),
            secondary(ui),
        );
        ui.painter().text(
            rect.left_top() + egui::vec2(16.0, 16.0),
            egui::Align2::LEFT_TOP,
            "LIVE FEED",
            egui::FontId::proportional(13.0),
            primary(ui),
        );
        ui.painter().text(
            rect.left_bottom() + egui::vec2(16.0, -28.0),
            egui::Align2::LEFT_BOTTOM,
            format!("Heading {:.1} deg", snap.heading),
            egui::FontId::proportional(16.0),
            primary(ui),
        );
        ui.add_space(8.0);
        ui.columns(4, |cols| {
            kpi(
                &mut cols[0..1],
                0,
                "CORE ENERGY",
                &snap.battery,
                &snap.voltage,
            );
            kpi(
                &mut cols[1..2],
                0,
                "TILT / TRIM",
                &format!("{:.1} deg", snap.pitch),
                &format!("P: {:.1} deg", snap.roll),
            );
            kpi(
                &mut cols[2..3],
                0,
                "SPEED",
                &format!("{:.2} m/s", snap.speed),
                &format!("Rot: {:.2}", snap.spin),
            );
            kpi(&mut cols[3..4], 0, "OBSTACLES", &snap.nearest, "LIDAR");
        });
        ui.add_space(8.0);
        ui.label(
            RichText::new("DEPLOYMENT SYSTEM")
                .strong()
                .color(secondary(ui)),
        );
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                section_title(ui, "CAMERA PTZ");
                ui.horizontal(|ui| {
                    if ui.button("Center").clicked() {
                        self.arm("Center", "camera_reset");
                    }
                    if ui.button("Demo").clicked() {
                        self.arm("PTZ demo", "camera_set_pos");
                    }
                });
                ui.horizontal(|ui| {
                    ui.add_space(28.0);
                    if ui.button("Up").clicked() {
                        self.arm("Tilt up", "camera_up");
                    }
                });
                ui.horizontal(|ui| {
                    if ui.button("Left").clicked() {
                        self.arm("Pan left", "camera_left");
                    }
                    if ui.button("Right").clicked() {
                        self.arm("Pan right", "camera_right");
                    }
                });
                ui.horizontal(|ui| {
                    ui.add_space(22.0);
                    if ui.button("Down").clicked() {
                        self.arm("Tilt down", "camera_down");
                    }
                });
                ui.label(RichText::new("Pan: ID 1  |  Tilt: ID 2").color(secondary(ui)));
            });
            ui.vertical(|ui| {
                section_title(ui, "GPIO");
                ui.horizontal(|ui| {
                    ui.label("Headlight LED");
                    if ui
                        .selectable_label(self.headlight, if self.headlight { "On" } else { "Off" })
                        .clicked()
                    {
                        self.headlight = !self.headlight;
                        self.note = "Headlight is GPIO 17. The tool bus has no gpio_set, so this switch is local.".to_string();
                    }
                });
                section_title(ui, "TAPO AUDIO");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.speak);
                    if ui.button("Speak").clicked() {
                        self.arm_params("Speak", "say", Some(self.speak.clone()), None, None);
                    }
                    if ui.button("Listen").clicked() {
                        self.note = "Tapo listen stays on the webapp microphone path.".to_string();
                    }
                });
                section_title(ui, "KEYBOARD");
                if ui
                    .selectable_label(self.wasd, if self.wasd { "Active" } else { "Enable" })
                    .clicked()
                {
                    self.wasd = !self.wasd;
                }
                ui.label(RichText::new("Use WASD keys to drive").color(secondary(ui)));
                section_title(ui, "LIGHT CONTROL");
                if ui.button("Stop").clicked() {
                    self.arm("Lights off", "led_off");
                }
                if ui.button("Patrol car pattern").clicked() {
                    self.arm("Patrol car", "patrol_car");
                }
                ui.horizontal(|ui| {
                    if ui.button("Danger red").clicked() {
                        self.arm_params(
                            "Danger red",
                            "led",
                            Some("255".into()),
                            Some("0".into()),
                            Some("0".into()),
                        );
                    }
                    if ui.button("Safe green").clicked() {
                        self.arm_params(
                            "Safe green",
                            "led",
                            Some("0".into()),
                            Some("255".into()),
                            Some("0".into()),
                        );
                    }
                });
                section_title(ui, "VOICE & MEDIA HUB");
                ui.label(RichText::new("Say Bumi Bumi to wake").color(secondary(ui)));
            });
        });
    }

    fn view_cards(&mut self, ui: &mut egui::Ui, heading: &str) {
        ui.label(RichText::new(heading).strong().color(secondary(ui)));
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
                ui.label(RichText::new(card.body).color(secondary(ui)));
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
                            self.arm(card.title, operation);
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

struct LayerRow {
    title: String,
    ok: bool,
}

struct DashSnap {
    connected: bool,
    ip: String,
    ros: String,
    ssh: String,
    video: String,
    cmd_vel: String,
    version: String,
    layers: Vec<LayerRow>,
    heading: f32,
    pitch: f32,
    roll: f32,
    speed: f32,
    spin: f32,
    battery: String,
    voltage: String,
    nearest: String,
}

const STACK_TITLES: &[&str] = &[
    "This PC to robot (TCP)",
    "SSH control session",
    "Pi: hostname, IP, Wi-Fi",
    "Pi: Docker engine",
    "Pi: ROS container (yahboom_ros2_final)",
    "Inside container: ROS 2 graph",
    "Inside container: rosbridge node",
    "This PC to rosbridge (WebSocket)",
    "Bridge: /cmd_vel advertised",
    "Video pipeline (WebBridge)",
];

fn dash_snapshot(root: Option<&serde_json::Value>, connected: bool) -> DashSnap {
    let body = root.and_then(|value| value.get("health"));
    let link = body.and_then(|value| value.get("robot_connection"));
    let text = |node: Option<&serde_json::Value>, key: &str, fallback: &str| -> String {
        node.and_then(|value| value.get(key))
            .and_then(|value| value.as_str())
            .unwrap_or(fallback)
            .to_string()
    };
    let ip = text(link, "ip", "");
    let ip = if ip.is_empty() {
        text(root, "robot_ip", "192.168.1.11")
    } else {
        ip
    };
    let cmd_vel = match link
        .and_then(|value| value.get("cmd_vel_ready"))
        .and_then(|value| value.as_bool())
    {
        Some(true) => "ready",
        _ => "not ready",
    };
    let layers = body
        .and_then(|value| value.get("stack"))
        .and_then(|value| value.get("layers"))
        .and_then(|value| value.as_array())
        .map(|rows| {
            rows.iter()
                .filter_map(|row| {
                    let title = row.get("title")?.as_str()?;
                    let ok = row
                        .get("ok")
                        .and_then(|value| value.as_bool())
                        .unwrap_or(false);
                    Some(LayerRow {
                        title: title.to_string(),
                        ok,
                    })
                })
                .collect::<Vec<_>>()
        })
        .filter(|rows| !rows.is_empty())
        .unwrap_or_else(|| {
            STACK_TITLES
                .iter()
                .map(|title| LayerRow {
                    title: (*title).to_string(),
                    ok: false,
                })
                .collect()
        });
    DashSnap {
        connected,
        ip,
        ros: text(link, "ros", "disconnected"),
        ssh: text(link, "ssh", "disconnected"),
        video: text(link, "video", "inactive"),
        cmd_vel: cmd_vel.to_string(),
        version: text(body.and_then(|value| value.get("system")), "version", "?"),
        layers,
        heading: 0.0,
        pitch: 0.0,
        roll: 0.0,
        speed: 0.0,
        spin: 0.0,
        battery: "--".to_string(),
        voltage: "--".to_string(),
        nearest: "Clear".to_string(),
    }
}

fn section_title(ui: &mut egui::Ui, title: &str) {
    ui.add_space(8.0);
    ui.label(RichText::new(title).strong().color(secondary(ui)));
}

fn kpi(cols: &mut [egui::Ui], index: usize, kicker: &str, value: &str, sub: &str) {
    let ui = &mut cols[index];
    let mut frame = egui::Frame::group(ui.style());
    frame.inner_margin = egui::Margin::same(10);
    frame.show(ui, |ui| {
        ui.label(RichText::new(kicker).size(13.0).color(secondary(ui)));
        ui.label(RichText::new(value).strong());
        ui.label(RichText::new(sub).color(secondary(ui)));
    });
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

fn status_ok(ui: &egui::Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(74, 222, 128)
    } else {
        OK
    }
}

fn status_bad(ui: &egui::Ui) -> Color32 {
    if ui.visuals().dark_mode {
        Color32::from_rgb(252, 165, 165)
    } else {
        DANGER
    }
}

fn primary(ui: &egui::Ui) -> Color32 {
    ui.visuals().text_color()
}

fn secondary(ui: &egui::Ui) -> Color32 {
    ui.visuals().weak_text_color()
}

fn harsh_visuals() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(Color32::from_rgb(245, 247, 250));
    visuals.weak_text_color = Some(Color32::from_rgb(186, 194, 206));
    visuals.window_fill = Color32::from_rgb(10, 13, 18);
    visuals.panel_fill = Color32::from_rgb(16, 26, 46);
    visuals.extreme_bg_color = Color32::from_rgb(10, 13, 18);
    visuals.code_bg_color = Color32::from_rgb(8, 12, 22);
    visuals.faint_bg_color = Color32::from_rgb(22, 34, 58);
    visuals.warn_fg_color = Color32::from_rgb(255, 200, 120);
    visuals.error_fg_color = Color32::from_rgb(252, 165, 165);
    visuals.widgets.noninteractive.fg_stroke =
        egui::Stroke::new(1.0, Color32::from_rgb(186, 194, 206));
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, Color32::from_rgb(245, 247, 250));
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, Color32::WHITE);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.5, Color32::WHITE);
    visuals.selection.bg_fill = Color32::from_rgb(49, 46, 129);
    visuals.selection.stroke = egui::Stroke::new(1.0, Color32::WHITE);
    visuals.hyperlink_color = Color32::from_rgb(165, 180, 252);
    visuals.window_corner_radius = egui::CornerRadius::same(12);
    visuals.menu_corner_radius = egui::CornerRadius::same(8);
    visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, Color32::from_rgb(38, 52, 82));
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, Color32::from_rgb(51, 65, 85));
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.5, ACCENT);
    visuals
}

fn crisp_light_visuals() -> egui::Visuals {
    let ink = Color32::from_rgb(15, 23, 42);
    let secondary = Color32::from_rgb(51, 65, 85);
    let mut visuals = egui::Visuals::light();
    visuals.override_text_color = Some(ink);
    visuals.weak_text_color = Some(secondary);
    visuals.window_fill = Color32::WHITE;
    visuals.panel_fill = Color32::from_rgb(248, 250, 252);
    visuals.extreme_bg_color = Color32::WHITE;
    visuals.code_bg_color = Color32::from_rgb(241, 245, 249);
    visuals.faint_bg_color = Color32::from_rgb(241, 245, 249);
    visuals.warn_fg_color = Color32::from_rgb(146, 64, 14);
    visuals.error_fg_color = DANGER;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, secondary);
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, ink);
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.5, ink);
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.5, ink);
    visuals.widgets.inactive.bg_fill = Color32::WHITE;
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(238, 242, 255);
    visuals.widgets.active.bg_fill = Color32::from_rgb(224, 231, 255);
    visuals.selection.bg_fill = Color32::from_rgb(224, 231, 255);
    visuals.selection.stroke = egui::Stroke::new(1.0, ink);
    visuals.hyperlink_color = ACCENT;
    visuals.window_corner_radius = egui::CornerRadius::same(12);
    visuals.menu_corner_radius = egui::CornerRadius::same(8);
    visuals.widgets.noninteractive.bg_stroke =
        egui::Stroke::new(1.0, Color32::from_rgb(203, 213, 225));
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, Color32::from_rgb(148, 163, 184));
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.5, ACCENT);
    visuals
}

fn apply_theme(ctx: &egui::Context, light: bool) {
    ctx.set_visuals(if light {
        crisp_light_visuals()
    } else {
        harsh_visuals()
    });
    let theme = if light {
        egui::Theme::Light
    } else {
        egui::Theme::Dark
    };
    ctx.style_mut_of(theme, |style| {
        use egui::TextStyle::{Body, Button, Heading, Monospace, Small};
        for (text_style, size) in [
            (Body, 16.0),
            (Button, 16.0),
            (Heading, 23.0),
            (Monospace, 14.0),
            (Small, 13.0),
        ] {
            if let Some(font) = style.text_styles.get_mut(&text_style) {
                font.size = size;
            }
        }
        style.spacing.item_spacing = egui::vec2(10.0, 8.0);
        style.spacing.button_padding = egui::vec2(10.0, 6.0);
    });
}

fn theme_toggle_button(ui: &mut egui::Ui, light: &mut bool) {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(30.0, 30.0), egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        ui.painter().rect(
            rect,
            egui::CornerRadius::same(6),
            visuals.bg_fill,
            visuals.bg_stroke,
            egui::StrokeKind::Inside,
        );
        let center = rect.center();
        let painter = ui.painter();
        if *light {
            let moon = Color32::from_rgb(30, 41, 59);
            painter.circle_filled(center, 6.5, moon);
            painter.circle_filled(center + egui::vec2(3.0, -2.2), 5.5, visuals.bg_fill);
        } else {
            let sun = Color32::from_rgb(217, 119, 6);
            painter.circle_filled(center, 4.5, sun);
            for i in 0..8 {
                let angle = (i as f32) * std::f32::consts::TAU / 8.0;
                let dir = egui::vec2(angle.cos(), angle.sin());
                painter.line_segment(
                    [center + dir * 6.5, center + dir * 9.2],
                    egui::Stroke::new(1.6, sun),
                );
            }
        }
    }
    if response
        .on_hover_text("Toggle light / dark theme")
        .clicked()
    {
        *light = !*light;
        apply_theme(ui.ctx(), *light);
    }
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
        assert_eq!(Page::Companion.slug(), "chat");
        assert_eq!(Page::Visualization.slug(), "viz");
    }

    #[test]
    #[ignore = "writes docs/parity/rust; cargo test -p yahboom-app -- --ignored write_page_stills"]
    fn write_page_stills() {
        let runtime = tokio::runtime::Runtime::new().unwrap_or_else(|err| {
            eprintln!("runtime init failed: {err}");
            std::process::exit(1);
        });
        let app = YahboomApp::offline(std::sync::Arc::new(runtime));
        let mut harness = egui_kittest::Harness::builder()
            .with_size(egui::vec2(1280.0, 860.0))
            .with_render_cursor(false)
            .with_accessibility_check(false)
            .wgpu()
            .build_ui_state(|ui, app: &mut YahboomApp| app.draw(ui), app);
        apply_theme(&harness.ctx, false);
        harness.run();
        let dir =
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/parity/rust");
        let created = std::fs::create_dir_all(&dir);
        assert!(created.is_ok(), "create stills dir");
        let pages: Vec<Page> = GROUPS
            .iter()
            .flat_map(|group| group.pages.iter().copied())
            .collect();
        assert_eq!(pages.len(), 30);
        for page in pages {
            harness.state_mut().page = page;
            harness.state_mut().pending = None;
            apply_theme(&harness.ctx, false);
            harness.run();
            let rendered = harness.render();
            assert!(rendered.is_ok(), "render {}", page.slug());
            let Ok(image) = rendered else {
                return;
            };
            let path = dir.join(format!("{}.png", page.slug()));
            let saved = image.save(&path);
            assert!(saved.is_ok(), "save {}", path.display());
        }
    }
}
