use crate::system::wide;
use std::mem;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;
use winapi::ctypes::c_void;
use winapi::shared::minwindef::{BOOL, LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{COLORREF, HBRUSH, HDC, HFONT, HGDIOBJ, HWND, RECT};
use winapi::um::winuser::{
    BeginPaint, CreateWindowExW, DefWindowProcW, DrawTextW, EndPaint, FillRect, GetClientRect,
    GetDC, GetSystemMetrics, InvalidateRect, KillTimer, PostMessageW, RegisterClassW, ReleaseDC,
    SetLayeredWindowAttributes, SetTimer, SetWindowPos, SetWindowRgn, ShowWindow, HWND_TOPMOST,
    PAINTSTRUCT, SM_CXSCREEN, SWP_NOACTIVATE, SWP_NOSIZE, SWP_SHOWWINDOW, SW_HIDE, WM_PAINT,
    WM_TIMER, WM_USER, WNDCLASSW,
};

const WM_OVERLAY_SHOW: UINT = WM_USER + 10;
const TIMER_ID: usize = 1;

const W: i32 = 140;
const H: i32 = 34;
const MARGIN: i32 = 20;

const FADE_IN: f32 = 180.0;
const FADE_OUT_AT: f32 = 1200.0;
const TOTAL: f32 = 1500.0;
const MAX_ALPHA: f32 = 238.0;

const WS_POPUP: u32 = 0x8000_0000;
const WS_EX_TOPMOST: u32 = 0x0000_0008;
const WS_EX_TRANSPARENT: u32 = 0x0000_0020;
const WS_EX_TOOLWINDOW: u32 = 0x0000_0080;
const WS_EX_LAYERED: u32 = 0x0008_0000;
const WS_EX_NOACTIVATE: u32 = 0x0800_0000;
const LWA_ALPHA: u32 = 0x2;
const DT_LEFT: u32 = 0x0;
const DT_RIGHT: u32 = 0x2;
const DT_VCENTER: u32 = 0x4;
const DT_SINGLELINE: u32 = 0x20;
const DT_NOPREFIX: u32 = 0x800;

static OVERLAY_HWND: AtomicUsize = AtomicUsize::new(0);
static OVERLAY_ON: AtomicBool = AtomicBool::new(false);
static SCALE_PCT: AtomicU32 = AtomicU32::new(100);
static START: Mutex<Option<Instant>> = Mutex::new(None);

#[link(name = "gdi32")]
unsafe extern "system" {
    fn CreateSolidBrush(color: COLORREF) -> HBRUSH;
    fn CreateFontW(
        height: i32,
        width: i32,
        escapement: i32,
        orientation: i32,
        weight: i32,
        italic: u32,
        underline: u32,
        strike_out: u32,
        char_set: u32,
        out_precision: u32,
        clip_precision: u32,
        quality: u32,
        pitch_and_family: u32,
        face: *const u16,
    ) -> HFONT;
    fn SelectObject(hdc: HDC, obj: HGDIOBJ) -> HGDIOBJ;
    fn DeleteObject(obj: HGDIOBJ) -> BOOL;
    fn SetBkMode(hdc: HDC, mode: i32) -> i32;
    fn SetTextColor(hdc: HDC, color: COLORREF) -> COLORREF;
    fn CreateRoundRectRgn(x1: i32, y1: i32, x2: i32, y2: i32, w: i32, h: i32) -> *mut c_void;
    fn GetDeviceCaps(hdc: HDC, index: i32) -> i32;
}

fn rgb(r: u32, g: u32, b: u32) -> COLORREF {
    r | (g << 8) | (b << 16)
}

fn sc(v: i32) -> i32 {
    v * SCALE_PCT.load(Ordering::SeqCst) as i32 / 100
}

fn ease_out(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    1.0 - (1.0 - x).powi(3)
}

unsafe fn fill(hdc: HDC, rc: &RECT, color: COLORREF) {
    let brush = CreateSolidBrush(color);
    FillRect(hdc, rc, brush);
    DeleteObject(brush as HGDIOBJ);
}

unsafe fn make_font(height_px: i32, weight: i32) -> HFONT {
    let face = wide("Segoe UI");
    CreateFontW(-height_px, 0, 0, 0, weight, 0, 0, 0, 1, 0, 0, 5, 0, face.as_ptr())
}

unsafe fn paint(hwnd: HWND) {
    let mut ps: PAINTSTRUCT = mem::zeroed();
    let hdc = BeginPaint(hwnd, &mut ps);
    if hdc.is_null() {
        return;
    }
    let on = OVERLAY_ON.load(Ordering::SeqCst);
    let mut rc: RECT = mem::zeroed();
    GetClientRect(hwnd, &mut rc);

    fill(hdc, &rc, rgb(18, 19, 22));
    let bar = RECT { left: 0, top: 0, right: sc(4), bottom: rc.bottom };
    fill(hdc, &bar, if on { rgb(52, 211, 120) } else { rgb(112, 116, 126) });

    SetBkMode(hdc, 1); // TRANSPARENT

    let small = make_font(sc(11), 600);
    let old = SelectObject(hdc, small as HGDIOBJ);
    SetTextColor(hdc, rgb(150, 154, 164));
    let mut left = RECT { left: sc(16), top: 0, right: rc.right, bottom: rc.bottom };
    let name = wide("Better FN");
    DrawTextW(hdc, name.as_ptr(), -1, &mut left, DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
    SelectObject(hdc, old);
    DeleteObject(small as HGDIOBJ);

    let big = make_font(sc(15), 700);
    let old = SelectObject(hdc, big as HGDIOBJ);
    SetTextColor(hdc, if on { rgb(255, 255, 255) } else { rgb(176, 180, 190) });
    let mut right = RECT { left: 0, top: 0, right: rc.right - sc(14), bottom: rc.bottom };
    let state = wide(if on { "ON" } else { "OFF" });
    DrawTextW(hdc, state.as_ptr(), -1, &mut right, DT_RIGHT | DT_VCENTER | DT_SINGLELINE | DT_NOPREFIX);
    SelectObject(hdc, old);
    DeleteObject(big as HGDIOBJ);

    EndPaint(hwnd, &ps);
}

unsafe fn apply_frame(hwnd: HWND, ms: f32) {
    let alpha = if ms < FADE_IN {
        ease_out(ms / FADE_IN)
    } else if ms > FADE_OUT_AT {
        (1.0 - (ms - FADE_OUT_AT) / (TOTAL - FADE_OUT_AT)).clamp(0.0, 1.0)
    } else {
        1.0
    };
    let slide = if ms < FADE_IN {
        ((1.0 - ease_out(ms / FADE_IN)) * sc(14) as f32) as i32
    } else {
        0
    };
    let x = (GetSystemMetrics(SM_CXSCREEN) - sc(W) - sc(MARGIN)).max(0);
    SetLayeredWindowAttributes(hwnd, 0, (alpha * MAX_ALPHA) as u8, LWA_ALPHA);
    SetWindowPos(
        hwnd,
        HWND_TOPMOST,
        x,
        sc(MARGIN) - slide,
        0,
        0,
        SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
    );
}

unsafe extern "system" fn overlay_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_OVERLAY_SHOW => {
            OVERLAY_ON.store(wparam != 0, Ordering::SeqCst);
            if let Ok(mut s) = START.lock() {
                *s = Some(Instant::now());
            }
            InvalidateRect(hwnd, std::ptr::null(), 1);
            apply_frame(hwnd, 0.0);
            SetTimer(hwnd, TIMER_ID, 15, None);
            0
        }
        WM_TIMER => {
            let elapsed = START
                .lock()
                .ok()
                .and_then(|s| s.map(|t| t.elapsed().as_secs_f32() * 1000.0));
            match elapsed {
                Some(ms) if ms < TOTAL => apply_frame(hwnd, ms),
                _ => {
                    KillTimer(hwnd, TIMER_ID);
                    ShowWindow(hwnd, SW_HIDE);
                }
            }
            0
        }
        WM_PAINT => {
            paint(hwnd);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

pub unsafe fn create_window(hinstance: winapi::shared::minwindef::HINSTANCE) {
    let screen = GetDC(std::ptr::null_mut());
    if !screen.is_null() {
        let dpi = GetDeviceCaps(screen, 88); 
        ReleaseDC(std::ptr::null_mut(), screen);
        if dpi > 0 {
            SCALE_PCT.store((dpi * 100 / 96) as u32, Ordering::SeqCst);
        }
    }

    let class_name = wide("BetterFnOverlay");
    let wc = WNDCLASSW {
        style: 0,
        lpfnWndProc: Some(overlay_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinstance,
        hIcon: std::ptr::null_mut(),
        hCursor: std::ptr::null_mut(),
        hbrBackground: std::ptr::null_mut(),
        lpszMenuName: std::ptr::null(),
        lpszClassName: class_name.as_ptr(),
    };
    RegisterClassW(&wc);

    let (w, h) = (sc(W), sc(H));
    let x = (GetSystemMetrics(SM_CXSCREEN) - w - sc(MARGIN)).max(0);
    let hwnd = CreateWindowExW(
        WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_LAYERED | WS_EX_TRANSPARENT,
        class_name.as_ptr(),
        class_name.as_ptr(),
        WS_POPUP,
        x,
        sc(MARGIN),
        w,
        h,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        hinstance,
        std::ptr::null_mut(),
    );
    if hwnd.is_null() {
        return;
    }
    SetLayeredWindowAttributes(hwnd, 0, 0, LWA_ALPHA);
    SetWindowRgn(hwnd, CreateRoundRectRgn(0, 0, w + 1, h + 1, sc(14), sc(14)) as _, 1);
    OVERLAY_HWND.store(hwnd as usize, Ordering::SeqCst);
}

pub fn show(on: bool) {
    let hwnd = OVERLAY_HWND.load(Ordering::SeqCst) as HWND;
    if !hwnd.is_null() {
        unsafe {
            PostMessageW(hwnd, WM_OVERLAY_SHOW, on as WPARAM, 0);
        }
    }
}
