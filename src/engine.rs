use crate::config::{Mode, Profile, RuleSet, Settings};
use crate::keys::{is_mouse_vk, key_to_vk, vk_matches, VK_LBUTTON};
use crate::overlay;
use crate::sound::play_toggle_sound;
use crate::system::foreground_exe;
use crate::tray::update_tray;
use eframe::egui;
use std::{
    mem,
    sync::{
        atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError, Sender},
        Mutex, OnceLock,
    },
    thread,
    time::{Duration, Instant},
};
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::um::{
    libloaderapi::GetModuleHandleW,
    processthreadsapi::{GetCurrentThread, SetThreadPriority},
    winuser::{
        CallNextHookEx, DispatchMessageW, GetAsyncKeyState, GetMessageW,
        MapVirtualKeyW, SendInput, SetWindowsHookExW, TranslateMessage, INPUT, INPUT_KEYBOARD,
        INPUT_MOUSE, KBDLLHOOKSTRUCT, KEYBDINPUT, KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP,
        KEYEVENTF_SCANCODE, LLKHF_INJECTED, MAPVK_VK_TO_VSC, MOUSEINPUT, MSG, MSLLHOOKSTRUCT,
        WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
    },
};

#[link(name = "winmm")]
unsafe extern "system" {
    fn timeBeginPeriod(period: u32) -> u32;
}

pub static ENABLED: AtomicBool = AtomicBool::new(false);
pub static TRIGGER_VK: AtomicU32 = AtomicU32::new(0x47);
pub static ACTION_VK: AtomicU32 = AtomicU32::new(0x48);
pub static TOGGLE_VK: AtomicU32 = AtomicU32::new(0x2D);
pub static DELAY_MS: AtomicU32 = AtomicU32::new(8);
pub static MODE: AtomicU32 = AtomicU32::new(0);
pub static REPEAT_MS: AtomicU32 = AtomicU32::new(50);

pub static ACTION_IS_DOWN: AtomicBool = AtomicBool::new(false);
static PRESSED_VK: AtomicU32 = AtomicU32::new(0);

pub static SOUND: AtomicBool = AtomicBool::new(true);
pub static SOUND_VOLUME: AtomicU32 = AtomicU32::new(70);
pub static OVERLAY: AtomicBool = AtomicBool::new(true);
pub static RESTRICT: AtomicBool = AtomicBool::new(false);
pub static APP_ALLOWED: AtomicBool = AtomicBool::new(true);
pub static TARGET_APP: Mutex<String> = Mutex::new(String::new());
static CTX: OnceLock<egui::Context> = OnceLock::new();

pub static TRIGGER_DOWN: AtomicBool = AtomicBool::new(false);
pub static TOGGLE_COUNT: AtomicU32 = AtomicU32::new(0);

pub static ACTIVE_PROFILE: AtomicUsize = AtomicUsize::new(usize::MAX);

static TOGGLE_HELD: AtomicBool = AtomicBool::new(false);
static TRIGGER_HELD: AtomicBool = AtomicBool::new(false);

static DEFAULT_RULES: Mutex<Option<RuleSet>> = Mutex::new(None);
static PROFILES: Mutex<Vec<Profile>> = Mutex::new(Vec::new());

static CAPTURE_ARMED: AtomicBool = AtomicBool::new(false);
static CAPTURED: AtomicU32 = AtomicU32::new(0);
const CAPTURE_CANCELLED: u32 = u32::MAX;

const WATCHDOG_EVERY: Duration = Duration::from_millis(50);

pub fn set_ctx(ctx: egui::Context) {
    let _ = CTX.set(ctx);
}

pub fn wake_ui() {
    if let Some(ctx) = CTX.get() {
        ctx.request_repaint();
    }
}

pub fn effective_active() -> bool {
    ENABLED.load(Ordering::SeqCst) && APP_ALLOWED.load(Ordering::SeqCst)
}

pub fn current_mode() -> Mode {
    Mode::from_u32(MODE.load(Ordering::SeqCst))
}

fn is_extended(vk: u32) -> bool {
    matches!(
        vk,
        0x21..=0x28 | 0x2C | 0x2D | 0x2E | 0x5B | 0x5C | 0x6F | 0x90 | 0xA3 | 0xA5
    )
}

fn send_key(vk: u32, key_up: bool) {
    if vk == 0 {
        return;
    }
    if is_mouse_vk(vk) {
        send_mouse(vk, key_up);
        return;
    }
    unsafe {
        let scan = MapVirtualKeyW(vk, MAPVK_VK_TO_VSC) as u16;
        let mut input: INPUT = mem::zeroed();
        input.type_ = INPUT_KEYBOARD;
        let mut flags = KEYEVENTF_SCANCODE;
        if is_extended(vk) {
            flags |= KEYEVENTF_EXTENDEDKEY;
        }
        if key_up {
            flags |= KEYEVENTF_KEYUP;
        }
        *input.u.ki_mut() = KEYBDINPUT {
            wVk: 0,
            wScan: scan,
            dwFlags: flags,
            time: 0,
            dwExtraInfo: 0,
        };
        SendInput(1, &mut input, mem::size_of::<INPUT>() as i32);
    }
}

fn send_mouse(vk: u32, up: bool) {
    const LEFTDOWN: u32 = 0x0002;
    const LEFTUP: u32 = 0x0004;
    const RIGHTDOWN: u32 = 0x0008;
    const RIGHTUP: u32 = 0x0010;
    const MIDDLEDOWN: u32 = 0x0020;
    const MIDDLEUP: u32 = 0x0040;
    const XDOWN: u32 = 0x0080;
    const XUP: u32 = 0x0100;
    let (down_flag, up_flag, data) = match vk {
        0x01 => (LEFTDOWN, LEFTUP, 0),
        0x02 => (RIGHTDOWN, RIGHTUP, 0),
        0x04 => (MIDDLEDOWN, MIDDLEUP, 0),
        0x05 => (XDOWN, XUP, 1), 
        0x06 => (XDOWN, XUP, 2), 
        _ => return,
    };
    unsafe {
        let mut input: INPUT = mem::zeroed();
        input.type_ = INPUT_MOUSE;
        *input.u.mi_mut() = MOUSEINPUT {
            dx: 0,
            dy: 0,
            mouseData: data,
            dwFlags: if up { up_flag } else { down_flag },
            time: 0,
            dwExtraInfo: 0,
        };
        SendInput(1, &mut input, mem::size_of::<INPUT>() as i32);
    }
}

fn press_action() {
    if ACTION_IS_DOWN
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        let vk = ACTION_VK.load(Ordering::SeqCst);
        PRESSED_VK.store(vk, Ordering::SeqCst);
        send_key(vk, false);
    }
}

pub fn release_action() {
    if ACTION_IS_DOWN.swap(false, Ordering::SeqCst) {
        send_key(PRESSED_VK.load(Ordering::SeqCst), true);
    }
}

fn precise_sleep(d: Duration) {
    if d.is_zero() {
        return;
    }
    let end = Instant::now() + d;
    let coarse = Duration::from_millis(2);
    if d > coarse {
        thread::sleep(d - coarse);
    }
    while Instant::now() < end {
        std::hint::spin_loop();
    }
}

fn wait_delay() {
    precise_sleep(Duration::from_millis(DELAY_MS.load(Ordering::SeqCst) as u64));
}

fn repeat_interval() -> Duration {
    Duration::from_millis(REPEAT_MS.load(Ordering::SeqCst).max(1) as u64)
}

fn tap_action() {
    let interval = REPEAT_MS.load(Ordering::SeqCst) as u64;
    let hold = (interval / 2).clamp(1, 30);
    press_action();
    precise_sleep(Duration::from_millis(hold));
    release_action();
}

fn trigger_lost() -> bool {
    let vk = TRIGGER_VK.load(Ordering::SeqCst);
    unsafe { (GetAsyncKeyState(vk as i32) as u16 & 0x8000) == 0 }
}

enum Cmd {
    TriggerDown,
    TriggerUp,
    ToggleApp,
    Reset,
}

static CMD_TX: OnceLock<Mutex<Sender<Cmd>>> = OnceLock::new();

fn send_cmd(cmd: Cmd) {
    if let Some(tx) = CMD_TX.get() {
        if let Ok(tx) = tx.lock() {
            let _ = tx.send(cmd);
        }
    }
}

pub fn engine_reset() {
    send_cmd(Cmd::Reset);
}

pub fn request_toggle() {
    send_cmd(Cmd::ToggleApp);
}

pub fn shutdown() {
    ENABLED.store(false, Ordering::SeqCst);
    release_action();
}

fn toggle_enabled() {
    let was_enabled = ENABLED.fetch_xor(true, Ordering::SeqCst);
    if was_enabled {
        release_action();
    }
    TOGGLE_COUNT.fetch_add(1, Ordering::SeqCst);
    update_tray();
    wake_ui();
    if SOUND.load(Ordering::SeqCst) {
        play_toggle_sound(!was_enabled);
    }
    if OVERLAY.load(Ordering::SeqCst) {
        overlay::show(!was_enabled);
    }
}

fn worker(rx: Receiver<Cmd>) {
    unsafe {
        SetThreadPriority(GetCurrentThread(), 1); 
        timeBeginPeriod(1);
    }

    let mut held = false; 
    let mut next_tick: Option<Instant> = None; 
    let mut next_watch = Instant::now();

    loop {
        let wait = if held {
            let now = Instant::now();
            let mut t = next_watch.saturating_duration_since(now);
            if let Some(nt) = next_tick {
                t = t.min(nt.saturating_duration_since(now));
            }
            Some(t)
        } else {
            None
        };

        let cmd = match wait {
            Some(t) => match rx.recv_timeout(t) {
                Ok(c) => Some(c),
                Err(RecvTimeoutError::Timeout) => None,
                Err(RecvTimeoutError::Disconnected) => return,
            },
            None => match rx.recv() {
                Ok(c) => Some(c),
                Err(_) => return,
            },
        };

        if let Some(cmd) = cmd {
            match cmd {
                Cmd::TriggerDown => {
                    if effective_active() && !held {
                        held = true;
                        match current_mode() {
                            Mode::Hold => {
                                next_watch = Instant::now() + WATCHDOG_EVERY;
                                wait_delay();
                                press_action();
                            }
                            Mode::Repeat => {
                                wait_delay();
                                let start = Instant::now();
                                tap_action();
                                next_watch = start + WATCHDOG_EVERY;
                                next_tick = Some(start + repeat_interval());
                            }
                        }
                    }
                }
                Cmd::TriggerUp => {
                    if held {
                        held = false;
                        next_tick = None;
                        release_action();
                    }
                }
                Cmd::ToggleApp => {
                    held = false;
                    next_tick = None;
                    toggle_enabled();
                }
                Cmd::Reset => {
                    held = false;
                    next_tick = None;
                    release_action();
                }
            }
        }

        if held {
            let now = Instant::now();
            if now >= next_watch {
                next_watch = now + WATCHDOG_EVERY;
                if trigger_lost() {
                    held = false;
                    next_tick = None;
                    release_action();
                    TRIGGER_HELD.store(false, Ordering::SeqCst);
                    TRIGGER_DOWN.store(false, Ordering::SeqCst);
                    wake_ui();
                }
            }
            if held {
                if let Some(nt) = next_tick {
                    if now >= nt {
                        let start = Instant::now();
                        tap_action();
                        next_tick = Some(start + repeat_interval());
                    }
                }
            }
        }
    }
}

fn handle_input(vk: u32, down: bool) -> bool {
    if CAPTURE_ARMED.load(Ordering::SeqCst) {
        if vk == VK_LBUTTON || !down {
            return false; 
        }
        CAPTURED.store(
            if vk == 0x1B { CAPTURE_CANCELLED } else { vk },
            Ordering::SeqCst,
        );
        CAPTURE_ARMED.store(false, Ordering::SeqCst);
        wake_ui();
        return true;
    }

    if !is_mouse_vk(vk) && vk_matches(TOGGLE_VK.load(Ordering::SeqCst), vk) {
        if down {
            if !TOGGLE_HELD.swap(true, Ordering::SeqCst) {
                send_cmd(Cmd::ToggleApp);
            }
        } else {
            TOGGLE_HELD.store(false, Ordering::SeqCst);
        }
    }

    if vk_matches(TRIGGER_VK.load(Ordering::SeqCst), vk) {
        if down {
            if !TRIGGER_HELD.swap(true, Ordering::SeqCst) {
                let active = effective_active();
                TRIGGER_DOWN.store(active, Ordering::SeqCst);
                if active {
                    send_cmd(Cmd::TriggerDown);
                }
                wake_ui();
            }
        } else {
            TRIGGER_HELD.store(false, Ordering::SeqCst);
            TRIGGER_DOWN.store(false, Ordering::SeqCst);
            send_cmd(Cmd::TriggerUp);
            wake_ui();
        }
    }
    false
}

unsafe extern "system" fn keyboard_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 && lparam != 0 {
        let info = *(lparam as *const KBDLLHOOKSTRUCT);
        if (info.flags & LLKHF_INJECTED) == 0 {
            let msg = wparam as UINT;
            let down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
            let up = msg == WM_KEYUP || msg == WM_SYSKEYUP;
            if (down || up) && handle_input(info.vkCode, down) {
                return 1;
            }
        }
    }
    CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
}

unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    const LLMHF_INJECTED: u32 = 0x01;
    const WM_LBUTTONDOWN: u32 = 0x0201;
    const WM_LBUTTONUP: u32 = 0x0202;
    const WM_RBUTTONDOWN: u32 = 0x0204;
    const WM_RBUTTONUP: u32 = 0x0205;
    const WM_MBUTTONDOWN: u32 = 0x0207;
    const WM_MBUTTONUP: u32 = 0x0208;
    const WM_XBUTTONDOWN: u32 = 0x020B;
    const WM_XBUTTONUP: u32 = 0x020C;

    if code >= 0 && lparam != 0 {
        let info = &*(lparam as *const MSLLHOOKSTRUCT);
        if (info.flags & LLMHF_INJECTED) == 0 {
            let xbutton = if (info.mouseData >> 16) & 0xFFFF == 1 { 0x05 } else { 0x06 };
            let (vk, down) = match wparam as UINT {
                WM_LBUTTONDOWN => (0x01, true),
                WM_LBUTTONUP => (0x01, false),
                WM_RBUTTONDOWN => (0x02, true),
                WM_RBUTTONUP => (0x02, false),
                WM_MBUTTONDOWN => (0x04, true),
                WM_MBUTTONUP => (0x04, false),
                WM_XBUTTONDOWN => (xbutton, true),
                WM_XBUTTONUP => (xbutton, false),
                _ => (0, false),
            };
            if vk != 0 && handle_input(vk, down) {
                return 1;
            }
        }
    }
    CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
}

fn start_hooks() {
    thread::spawn(|| unsafe {
        let module = GetModuleHandleW(std::ptr::null());
        let kb = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_proc), module, 0);
        let ms = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), module, 0);
        if kb.is_null() && ms.is_null() {
            return;
        }
        let mut msg: MSG = mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    });
}

pub enum Captured {
    Key(u32),
    Cancelled,
}

pub fn start_capture() {
    CAPTURED.store(0, Ordering::SeqCst);
    CAPTURE_ARMED.store(true, Ordering::SeqCst);
}

pub fn stop_capture() {
    CAPTURE_ARMED.store(false, Ordering::SeqCst);
}

pub fn capture_armed() -> bool {
    CAPTURE_ARMED.load(Ordering::SeqCst)
}

pub fn poll_capture() -> Option<Captured> {
    match CAPTURED.swap(0, Ordering::SeqCst) {
        0 => None,
        CAPTURE_CANCELLED => Some(Captured::Cancelled),
        vk => Some(Captured::Key(vk)),
    }
}
fn apply_rules(r: &RuleSet) {
    TRIGGER_VK.store(key_to_vk(&r.trigger), Ordering::SeqCst);
    ACTION_VK.store(key_to_vk(&r.action), Ordering::SeqCst);
    DELAY_MS.store(r.delay_ms, Ordering::SeqCst);
    MODE.store(r.mode.as_u32(), Ordering::SeqCst);
    REPEAT_MS.store(r.repeat_ms, Ordering::SeqCst);
}

fn resolve_and_apply(force: bool, reset: bool) {
    let default = match DEFAULT_RULES.lock().ok().and_then(|d| d.clone()) {
        Some(d) => d,
        None => return,
    };
    let profiles: Vec<Profile> = PROFILES.lock().map(|p| p.clone()).unwrap_or_default();
    let restrict = RESTRICT.load(Ordering::SeqCst);
    let target = TARGET_APP.lock().map(|t| t.clone()).unwrap_or_default();

    let need_fg = !profiles.is_empty() || (restrict && !target.is_empty());
    let fg: Option<String> = if need_fg {
        foreground_exe().map(|(_, name)| name)
    } else {
        None
    };

    let matched = fg
        .as_ref()
        .and_then(|n| profiles.iter().position(|p| p.app.eq_ignore_ascii_case(n)));
    let (allowed, slot, rules) = match matched {
        Some(i) => (true, i + 1, profiles[i].rules.clone()),
        None => {
            let allowed = if restrict && !target.is_empty() {
                fg.as_deref()
                    .map(|n| n.eq_ignore_ascii_case(&target))
                    .unwrap_or(false)
            } else {
                true
            };
            (allowed, 0, default)
        }
    };

    let prev_slot = ACTIVE_PROFILE.swap(slot, Ordering::SeqCst);
    if force || prev_slot != slot {
        apply_rules(&rules);
        if reset || prev_slot != slot {
            send_cmd(Cmd::Reset);
        }
    }

    let prev_allowed = APP_ALLOWED.swap(allowed, Ordering::SeqCst);
    if prev_allowed != allowed {
        if !allowed {
            send_cmd(Cmd::Reset);
        }
        update_tray();
        wake_ui();
    } else if prev_slot != slot {
        wake_ui();
    }
}

pub fn apply_settings(s: &Settings, reset: bool) {
    TOGGLE_VK.store(key_to_vk(&s.toggle_key), Ordering::SeqCst);
    SOUND.store(s.sound, Ordering::SeqCst);
    SOUND_VOLUME.store(s.volume, Ordering::SeqCst);
    OVERLAY.store(s.overlay, Ordering::SeqCst);
    RESTRICT.store(s.restrict_app, Ordering::SeqCst);
    if let Ok(mut t) = TARGET_APP.lock() {
        *t = s.target_app.clone();
    }
    if let Ok(mut d) = DEFAULT_RULES.lock() {
        *d = Some(s.rules.clone());
    }
    if let Ok(mut p) = PROFILES.lock() {
        *p = s.profiles.clone();
    }
    resolve_and_apply(true, reset);
}

fn foreground_watcher_loop() {
    loop {
        thread::sleep(Duration::from_millis(120));
        resolve_and_apply(false, false);
    }
}

fn start_foreground_watcher() {
    thread::spawn(foreground_watcher_loop);
}

pub fn start() {
    let (tx, rx) = mpsc::channel::<Cmd>();
    let _ = CMD_TX.set(Mutex::new(tx));
    let _ = thread::Builder::new()
        .name("betterfn-engine".into())
        .spawn(move || worker(rx));
    start_hooks();
    start_foreground_watcher();
}
