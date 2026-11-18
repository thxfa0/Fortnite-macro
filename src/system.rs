use std::mem;
use std::thread;
use std::time::Duration;
use winapi::ctypes::c_void;
use winapi::shared::minwindef::{BOOL, DWORD, LPARAM};
use winapi::shared::winerror::ERROR_ALREADY_EXISTS;
use winapi::shared::windef::HWND;
use winapi::um::{
    errhandlingapi::GetLastError,
    handleapi::CloseHandle,
    processthreadsapi::{GetCurrentProcess, OpenProcess, OpenProcessToken},
    shellapi::ShellExecuteW,
    synchapi::CreateMutexW,
    winbase::QueryFullProcessImageNameW,
    winuser::{
        EnumWindows, GetForegroundWindow, GetWindow, GetWindowTextLengthW, GetWindowTextW,
        GetWindowThreadProcessId, IsWindowVisible, GW_OWNER,
    },
};

const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn exe_name_of_pid(pid: u32) -> Option<String> {
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return None;
        }
        let mut buf = [0u16; 520];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut len);
        CloseHandle(handle);
        if ok == 0 {
            return None;
        }
        let path = String::from_utf16_lossy(&buf[..len as usize]);
        path.rsplit('\\').next().map(|s| s.to_string())
    }
}

pub fn foreground_exe() -> Option<(u32, String)> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() {
            return None;
        }
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 {
            return None;
        }
        exe_name_of_pid(pid).map(|n| (pid, n))
    }
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let list = &mut *(lparam as *mut Vec<(String, String)>);

    if IsWindowVisible(hwnd) == 0 {
        return 1;
    }
    if !GetWindow(hwnd, GW_OWNER).is_null() {
        return 1;
    }
    let len = GetWindowTextLengthW(hwnd);
    if len <= 0 {
        return 1;
    }
    let mut buf = vec![0u16; len as usize + 1];
    let n = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
    if n <= 0 {
        return 1;
    }
    let title = String::from_utf16_lossy(&buf[..n as usize]);

    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, &mut pid);
    if pid == 0 || pid == std::process::id() {
        return 1;
    }
    if let Some(exe) = exe_name_of_pid(pid) {
        if !list.iter().any(|(e, _)| e.eq_ignore_ascii_case(&exe)) {
            list.push((exe, title));
        }
    }
    1
}

pub fn list_open_apps() -> Vec<(String, String)> {
    let mut list: Vec<(String, String)> = Vec::new();
    unsafe {
        EnumWindows(
            Some(enum_windows_proc),
            &mut list as *mut Vec<(String, String)> as LPARAM,
        );
    }
    list.sort_by_key(|(e, _)| e.to_lowercase());
    list
}

#[repr(C)]
struct TokenElevation {
    token_is_elevated: DWORD,
}

#[link(name = "advapi32")]
unsafe extern "system" {
    fn GetTokenInformation(
        token: *mut c_void,
        class: u32,
        info: *mut c_void,
        len: u32,
        ret_len: *mut u32,
    ) -> BOOL;
}

pub fn is_elevated() -> bool {
    const TOKEN_QUERY: u32 = 0x0008;
    const TOKEN_ELEVATION_CLASS: u32 = 20;
    unsafe {
        let mut token: *mut c_void = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }
        let mut elevation = TokenElevation { token_is_elevated: 0 };
        let mut ret = 0u32;
        let ok = GetTokenInformation(
            token,
            TOKEN_ELEVATION_CLASS,
            &mut elevation as *mut TokenElevation as *mut c_void,
            mem::size_of::<TokenElevation>() as u32,
            &mut ret,
        );
        CloseHandle(token);
        ok != 0 && elevation.token_is_elevated != 0
    }
}

pub fn relaunch_elevated(start_hidden: bool) -> bool {
    let exe = match std::env::current_exe() {
        Ok(p) => p,
        Err(_) => return false,
    };
    let mut args = String::from("--restart");
    if start_hidden {
        args.push_str(" --tray");
    }
    let verb = wide("runas");
    let file = wide(&exe.to_string_lossy());
    let params = wide(&args);
    unsafe {
        let r = ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            file.as_ptr(),
            params.as_ptr(),
            std::ptr::null(),
            1, 
        );
        (r as usize) > 32
    }
}
pub fn ensure_single_instance(start_hidden: bool, wait: bool) -> bool {
    let attempts = if wait { 50 } else { 1 };
    for i in 0..attempts {
        unsafe {
            let name = wide("Local\\BetterFN_SingleInstance");
            let handle = CreateMutexW(std::ptr::null_mut(), 0, name.as_ptr());
            if handle.is_null() {
                return true;
            }
            if GetLastError() != ERROR_ALREADY_EXISTS {
                return true;
            }
            CloseHandle(handle);
        }
        if i + 1 < attempts {
            thread::sleep(Duration::from_millis(100));
        }
    }
    if !start_hidden {
        crate::tray::show_main_window();
    }
    false
}
