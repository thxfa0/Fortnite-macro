use crate::keys::key_to_vk;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

fn default_trigger() -> String {
    "G".into()
}
fn default_action() -> String {
    "H".into()
}
fn default_delay() -> u32 {
    8
}
fn default_repeat() -> u32 {
    50
}
fn default_toggle() -> String {
    "INSERT".into()
}
fn default_true() -> bool {
    true
}
fn default_volume() -> u32 {
    70
}

pub const DELAY_MIN: u32 = 5;
pub const DELAY_MAX: u32 = 20;
pub const REPEAT_MIN: u32 = 10;
pub const REPEAT_MAX: u32 = 300;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    #[default]
    #[serde(alias = "toggle")]
    Hold,
    Repeat,
}

impl Mode {
    pub const ALL: [Mode; 2] = [Mode::Hold, Mode::Repeat];

    pub fn as_u32(self) -> u32 {
        match self {
            Mode::Hold => 0,
            Mode::Repeat => 1,
        }
    }

    pub fn from_u32(v: u32) -> Mode {
        match v {
            1 => Mode::Repeat,
            _ => Mode::Hold,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Mode::Hold => "Hold",
            Mode::Repeat => "Repeat",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct RuleSet {
    #[serde(default = "default_trigger")]
    pub trigger: String,
    #[serde(default = "default_action")]
    pub action: String,
    #[serde(default = "default_delay")]
    pub delay_ms: u32,
    #[serde(default)]
    pub mode: Mode,
    #[serde(default = "default_repeat")]
    pub repeat_ms: u32,
}

impl Default for RuleSet {
    fn default() -> Self {
        Self {
            trigger: default_trigger(),
            action: default_action(),
            delay_ms: default_delay(),
            mode: Mode::Hold,
            repeat_ms: default_repeat(),
        }
    }
}

impl RuleSet {
    pub fn sanitize(&mut self) {
        self.delay_ms = self.delay_ms.clamp(DELAY_MIN, DELAY_MAX);
        self.repeat_ms = self.repeat_ms.clamp(REPEAT_MIN, REPEAT_MAX);
        if key_to_vk(&self.trigger) == 0 {
            self.trigger = default_trigger();
        }
        if key_to_vk(&self.action) == 0 {
            self.action = default_action();
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Profile {
    pub app: String,
    #[serde(flatten)]
    pub rules: RuleSet,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    #[serde(flatten)]
    pub rules: RuleSet,
    #[serde(default = "default_toggle")]
    pub toggle_key: String,
    #[serde(default)]
    pub enable_on_launch: bool,
    #[serde(default = "default_true")]
    pub sound: bool,
    #[serde(default = "default_volume")]
    pub volume: u32,
    #[serde(default)]
    pub restrict_app: bool,
    #[serde(default)]
    pub target_app: String,
    #[serde(default)]
    pub run_as_admin: bool,
    #[serde(default = "default_true")]
    pub overlay: bool,
    #[serde(default)]
    pub profiles: Vec<Profile>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            rules: RuleSet::default(),
            toggle_key: default_toggle(),
            enable_on_launch: false,
            sound: true,
            volume: default_volume(),
            restrict_app: false,
            target_app: String::new(),
            run_as_admin: false,
            overlay: true,
            profiles: Vec::new(),
        }
    }
}

fn config_path() -> PathBuf {
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        })
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("BetterFN").join("config.json")
}

fn legacy_config_path() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("config.json")))
}

pub fn load_settings() -> Settings {
    let text = std::fs::read_to_string(config_path()).ok().or_else(|| {
        legacy_config_path().and_then(|p| std::fs::read_to_string(p).ok())
    });
    let mut settings = text
        .and_then(|s| serde_json::from_str::<Settings>(&s).ok())
        .unwrap_or_default();
    settings.rules.sanitize();
    settings.volume = settings.volume.min(100);
    if key_to_vk(&settings.toggle_key) == 0 {
        settings.toggle_key = default_toggle();
    }
    settings.profiles.retain(|p| !p.app.trim().is_empty());
    for p in settings.profiles.iter_mut() {
        p.rules.sanitize();
    }
    settings
}

pub fn save_settings(settings: &Settings) {
    let path = config_path();
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    if let Ok(json) = serde_json::to_string_pretty(settings) {
        let _ = std::fs::write(path, json);
    }
}
