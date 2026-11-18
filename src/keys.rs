pub const NAMED_KEYS: &[(&str, &str, u32)] = &[
    ("SPACE", "SPACE", 0x20),
    ("TAB", "TAB", 0x09),
    ("CAPS LOCK", "CAPS", 0x14),
    ("SHIFT", "SHIFT", 0x10),
    ("CTRL", "CTRL", 0x11),
    ("ALT", "ALT", 0x12),
    ("ENTER", "ENTER", 0x0D),
    ("BACKSPACE", "BACK", 0x08),
    ("INSERT", "INS", 0x2D),
    ("DELETE", "DEL", 0x2E),
    ("HOME", "HOME", 0x24),
    ("END", "END", 0x23),
    ("PAGE UP", "PGUP", 0x21),
    ("PAGE DOWN", "PGDN", 0x22),
    ("PAUSE", "PAUSE", 0x13),
    ("SCROLL LOCK", "SCROLL", 0x91),
    ("ESC", "ESC", 0x1B),
    ("UP", "UP", 0x26),
    ("DOWN", "DOWN", 0x28),
    ("LEFT", "LEFT", 0x25),
    ("RIGHT", "RIGHT", 0x27),
    ("LSHIFT", "LSHIFT", 0xA0),
    ("RSHIFT", "RSHIFT", 0xA1),
    ("LCTRL", "LCTRL", 0xA2),
    ("RCTRL", "RCTRL", 0xA3),
    ("LALT", "LALT", 0xA4),
    ("RALT", "RALT", 0xA5),
    ("LWIN", "LWIN", 0x5B),
    ("RWIN", "RWIN", 0x5C),
    ("PRINT SCREEN", "PRTSC", 0x2C),
    ("NUM LOCK", "NUMLK", 0x90),
    ("NUM 0", "NUM 0", 0x60),
    ("NUM 1", "NUM 1", 0x61),
    ("NUM 2", "NUM 2", 0x62),
    ("NUM 3", "NUM 3", 0x63),
    ("NUM 4", "NUM 4", 0x64),
    ("NUM 5", "NUM 5", 0x65),
    ("NUM 6", "NUM 6", 0x66),
    ("NUM 7", "NUM 7", 0x67),
    ("NUM 8", "NUM 8", 0x68),
    ("NUM 9", "NUM 9", 0x69),
    ("NUM *", "NUM *", 0x6A),
    ("NUM +", "NUM +", 0x6B),
    ("NUM -", "NUM -", 0x6D),
    ("NUM .", "NUM .", 0x6E),
    ("NUM /", "NUM /", 0x6F),
    (";", ";", 0xBA),
    ("=", "=", 0xBB),
    (",", ",", 0xBC),
    ("-", "-", 0xBD),
    (".", ".", 0xBE),
    ("/", "/", 0xBF),
    ("`", "`", 0xC0),
    ("[", "[", 0xDB),
    ("\\", "\\", 0xDC),
    ("]", "]", 0xDD),
    ("'", "'", 0xDE),
    ("LMB", "LMB", 0x01),
    ("RMB", "RMB", 0x02),
    ("MMB", "MMB", 0x04),
    ("MOUSE4", "MOUSE4", 0x05),
    ("MOUSE5", "MOUSE5", 0x06),
];

pub const VK_LBUTTON: u32 = 0x01;

pub fn is_mouse_vk(vk: u32) -> bool {
    matches!(vk, 0x01 | 0x02 | 0x04 | 0x05 | 0x06)
}

pub fn key_to_vk(key: &str) -> u32 {
    let k = key.trim().to_ascii_uppercase();
    if let Some((_, _, vk)) = NAMED_KEYS.iter().find(|(n, d, _)| *n == k || *d == k) {
        return *vk;
    }
    let b = k.as_bytes();
    if b.len() == 1 && b[0].is_ascii_alphanumeric() {
        return b[0] as u32; 
    }
    if let Some(n) = k.strip_prefix('F').and_then(|s| s.parse::<u32>().ok()) {
        if (1..=24).contains(&n) {
            return 0x70 + n - 1;
        }
    }
    if let Some(h) = k.strip_prefix("VK") {
        if let Ok(v) = u32::from_str_radix(h.trim(), 16) {
            if (1..=0xFE).contains(&v) {
                return v;
            }
        }
    }
    0
}

pub fn vk_to_config(vk: u32) -> String {
    if let Some((n, _, _)) = NAMED_KEYS.iter().find(|(_, _, v)| *v == vk) {
        return (*n).to_string();
    }
    match vk {
        0x30..=0x39 | 0x41..=0x5A => ((vk as u8) as char).to_string(),
        0x70..=0x87 => format!("F{}", vk - 0x6F),
        _ => format!("VK{:02X}", vk),
    }
}

pub fn vk_to_name(vk: u32) -> String {
    if let Some((_, d, _)) = NAMED_KEYS.iter().find(|(_, _, v)| *v == vk) {
        return (*d).to_string();
    }
    match vk {
        0x30..=0x39 | 0x41..=0x5A => ((vk as u8) as char).to_string(),
        0x70..=0x87 => format!("F{}", vk - 0x6F),
        _ => format!("0x{:X}", vk),
    }
}

pub fn vk_matches(configured: u32, event: u32) -> bool {
    if configured == event {
        return true;
    }
    matches!(
        (configured, event),
        (0x10, 0xA0) | (0x10, 0xA1) | (0x11, 0xA2) | (0x11, 0xA3) | (0x12, 0xA4) | (0x12, 0xA5)
    )
}

pub const KEY_ROWS: &[&[&str]] = &[
    &["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"],
    &["A", "S", "D", "F", "G", "H", "J", "K", "L"],
    &["Z", "X", "C", "V", "B", "N", "M"],
    &["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],
    &["F1", "F2", "F3", "F4", "F5", "F6"],
    &["F7", "F8", "F9", "F10", "F11", "F12"],
    &["SPACE", "TAB", "CAPS LOCK", "SHIFT"],
    &["CTRL", "ALT", "ENTER", "BACKSPACE"],
];

pub const MOUSE_ROW: &[&str] = &["MMB", "MOUSE4", "MOUSE5", "RMB", "LMB"];

pub const TOGGLE_ROWS: &[&[&str]] = &[
    &["F1", "F2", "F3", "F4", "F5", "F6"],
    &["F7", "F8", "F9", "F10", "F11", "F12"],
    &["INSERT", "DELETE", "HOME", "END"],
    &["PAGE UP", "PAGE DOWN", "PAUSE", "SCROLL LOCK"],
];

pub fn key_width(name: &str) -> f32 {
    if name.starts_with('F') && name.len() > 1 && name[1..].chars().all(|c| c.is_ascii_digit())
    {
        46.0
    } else if name.chars().count() == 1 {
        26.0
    } else if name.starts_with("MOUSE") || name.ends_with("MB") {
        54.0
    } else {
        71.0
    }
}
