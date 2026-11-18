use crate::engine::{
    effective_active, request_toggle, shutdown, wake_ui, APP_ALLOWED, ENABLED, TARGET_APP,
    TOGGLE_VK,
};
use crate::icon::{load_icon_on, make_icon_rgba, rgba_to_hicon};
use crate::keys::vk_to_name;
use crate::system::wide;
use std::{
    mem,
    sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering},
    thread,
};
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HICON, HWND, POINT};
use winapi::um::{
    libloaderapi::GetModuleHandleW,
    shellapi::{
        Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
        NOTIFYICONDATAW,
    },
    winuser::{
        AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu,
        DestroyWindow, DispatchMessageW, FindWindowW, GetCursorPos, GetMessageW,
        GetSystemMetrics, PostMessageW, PostQuitMessage, RegisterClassW,
        RegisterWindowMessageW, SetForegroundWindow, SetMenuDefaultItem, ShowWindow,
        TrackPopupMenu, TranslateMessage, UnregisterClassW, CW_USEDEFAULT, MF_CHECKED,
        MF_SEPARATOR, MF_STRING, MF_UNCHECKED, MSG, SM_CXSMICON, SW_RESTORE, TPM_BOTTOMALIGN,
        TPM_LEFTALIGN, TPM_RETURNCMD, TPM_RIGHTBUTTON, WM_DESTROY, WM_LBUTTONUP, WM_NULL,
        WM_RBUTTONUP, WM_USER, WNDCLASSW,
    },
};

pub static SHOW_REQUEST: AtomicBool = AtomicBool::new(false);

static TRAY_HWND: AtomicUsize = AtomicUsize::new(0);
static TRAY_ICON_ON: AtomicUsize = AtomicUsize::new(0);
static TRAY_ICON_OFF: AtomicUsize = AtomicUsize::new(0);
static TASKBAR_CREATED: AtomicU32 = AtomicU32::new(0);

const TRAY_ICON_ID: u32 = 1;
const WM_TRAYICON: u32 = WM_USER + 1;
const WM_TRAY_REFRESH: u32 = WM_USER + 2;
const ID_TRAY_SHOW: u32 = 1001;
const ID_TRAY_TOGGLE: u32 = 1002;
const ID_TRAY_EXIT: u32 = 1003;

fn tray_tip() -> String {
    if !ENABLED.load(Ordering::SeqCst) {
        "Better FN: off".to_string()
    } else if !APP_ALLOWED.load(Ordering::SeqCst) {
        let target = TARGET_APP.lock().map(|t| t.clone()).unwrap_or_default();
        format!("Better FN: on (waiting for {})", target)
    } else {
        "Better FN: on".to_string()
    }
}

unsafe fn tray_data(hwnd: HWND) -> NOTIFYICONDATAW {
    let mut nid: NOTIFYICONDATAW = mem::zeroed();
    nid.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = TRAY_ICON_ID;
    nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP;
    nid.uCallbackMessage = WM_TRAYICON;
    let icon = if effective_active() {
        TRAY_ICON_ON.load(Ordering::SeqCst)
    } else {
        TRAY_ICON_OFF.load(Ordering::SeqCst)
    };
    nid.hIcon = icon as HICON;
    let tip: Vec<u16> = tray_tip().encode_utf16().collect();
    for (i, &c) in tip.iter().enumerate().take(127) {
        nid.szTip[i] = c;
    }
    nid
}

unsafe fn add_tray_icon(hwnd: HWND) {
    let mut nid = tray_data(hwnd);
    Shell_NotifyIconW(NIM_ADD, &mut nid);
}

unsafe fn modify_tray_icon(hwnd: HWND) {
    let mut nid = tray_data(hwnd);
    Shell_NotifyIconW(NIM_MODIFY, &mut nid);
}

pub fn update_tray() {
    let hwnd = TRAY_HWND.load(Ordering::SeqCst) as HWND;
    if !hwnd.is_null() {
        unsafe {
            PostMessageW(hwnd, WM_TRAY_REFRESH, 0, 0);
        }
    }
}

fn remove_tray_icon() {
    unsafe {
        let hwnd = TRAY_HWND.load(Ordering::SeqCst) as HWND;
        if !hwnd.is_null() {
            let mut nid: NOTIFYICONDATAW = mem::zeroed();
            nid.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = TRAY_ICON_ID;
            Shell_NotifyIconW(NIM_DELETE, &mut nid);
        }
    }
}

pub fn quit_app() -> ! {
    shutdown();
    remove_tray_icon();
    std::process::exit(0);
}

pub fn show_main_window() {
    unsafe {
        let title = wide("Better FN");
        let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
        if !hwnd.is_null() {
            ShowWindow(hwnd, SW_RESTORE);
            SetForegroundWindow(hwnd);
        }
    }
    SHOW_REQUEST.store(true, Ordering::SeqCst);
    wake_ui();
}

unsafe fn show_tray_menu(hwnd: HWND) {
    let mut pt: POINT = mem::zeroed();
    GetCursorPos(&mut pt);

    let menu = CreatePopupMenu();
    if menu.is_null() {
        return;
    }

    let toggle_name = vk_to_name(TOGGLE_VK.load(Ordering::SeqCst));
    let show_text = wide("Show Better FN");
    let active_text = wide(&format!("Active  ({})", toggle_name));
    let exit_text = wide("Exit");
    let check = if ENABLED.load(Ordering::SeqCst) {
        MF_CHECKED
    } else {
        MF_UNCHECKED
    };

    AppendMenuW(menu, MF_STRING, ID_TRAY_SHOW as usize, show_text.as_ptr());
    AppendMenuW(
        menu,
        MF_STRING | check,
        ID_TRAY_TOGGLE as usize,
        active_text.as_ptr(),
    );
    AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null());
    AppendMenuW(menu, MF_STRING, ID_TRAY_EXIT as usize, exit_text.as_ptr());
    SetMenuDefaultItem(menu, ID_TRAY_SHOW, 0);

    SetForegroundWindow(hwnd);

    let cmd = TrackPopupMenu(
        menu,
        TPM_RETURNCMD | TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RIGHTBUTTON,
        pt.x,
        pt.y,
        0,
        hwnd,
        std::ptr::null(),
    ) as u32;

    PostMessageW(hwnd, WM_NULL, 0, 0);
    DestroyMenu(menu);

    if cmd == ID_TRAY_SHOW {
        show_main_window();
    } else if cmd == ID_TRAY_TOGGLE {
        request_toggle();
    } else if cmd == ID_TRAY_EXIT {
        quit_app();
    }
}

unsafe extern "system" fn tray_wnd_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_TRAYICON {
        let mouse_msg = lparam as UINT;
        if mouse_msg == WM_LBUTTONUP {
            show_main_window();
        } else if mouse_msg == WM_RBUTTONUP {
            show_tray_menu(hwnd);
        }
        return 0;
    }

    if msg == WM_TRAY_REFRESH {
        modify_tray_icon(hwnd);
        return 0;
    }

    let taskbar_created = TASKBAR_CREATED.load(Ordering::SeqCst);
    if taskbar_created != 0 && msg == taskbar_created {
        add_tray_icon(hwnd);
        return 0;
    }

    if msg == WM_DESTROY {
        PostQuitMessage(0);
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

pub fn start_tray() {
    thread::spawn(|| unsafe {
        let class_name = wide("BetterFnTray");
        let hinstance = GetModuleHandleW(std::ptr::null());
        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(tray_wnd_proc),
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
        crate::overlay::create_window(hinstance);

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            class_name.as_ptr(),
            0,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            hinstance,
            std::ptr::null_mut(),
        );
        TRAY_HWND.store(hwnd as usize, Ordering::SeqCst);

        let tb = wide("TaskbarCreated");
        TASKBAR_CREATED.store(RegisterWindowMessageW(tb.as_ptr()), Ordering::SeqCst);

        let size = GetSystemMetrics(SM_CXSMICON).max(16);
        let icon_on = load_icon_on(size);
        let icon_off = rgba_to_hicon(&make_icon_rgba(size as usize, false), size as usize);
        TRAY_ICON_ON.store(icon_on as usize, Ordering::SeqCst);
        TRAY_ICON_OFF.store(
            if icon_off.is_null() { icon_on as usize } else { icon_off as usize },
            Ordering::SeqCst,
        );
        add_tray_icon(hwnd);

        let mut msg: MSG = mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        remove_tray_icon();
        DestroyWindow(hwnd);
        UnregisterClassW(class_name.as_ptr(), hinstance);
    });
}
