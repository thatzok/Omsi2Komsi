#[cfg(not(target_arch = "x86"))]
compile_error!("This plugin must be compiled for x86 (32-bit) to be compatible with OMSI!");

use libc::{c_char, c_float};
use std::ffi::OsString;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::os::windows::ffi::OsStringExt;
use std::slice;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering::Relaxed};
use std::sync::{Mutex, OnceLock, RwLock};
use std::thread;
use std::time::Duration;

#[allow(non_camel_case_types)]
pub type uintptr_t = usize;

const SHARED_ARRAY_SIZE: usize = 100;

static SHARED_ARRAY: [AtomicU32; SHARED_ARRAY_SIZE] = {
    const ARRAY_REPEAT_VALUE: AtomicU32 = AtomicU32::new(0);
    [ARRAY_REPEAT_VALUE; SHARED_ARRAY_SIZE]
};

static VAR_NAMES: OnceLock<Vec<String>> = OnceLock::new();
static STRING_VAR_NAMES: OnceLock<Vec<String>> = OnceLock::new();
static STRING_VAR_VALUES: RwLock<Vec<String>> = RwLock::new(Vec::new());
static HOTKEY: OnceLock<u32> = OnceLock::new();
static LOG_MESSAGES: Mutex<Vec<String>> = Mutex::new(Vec::new());
static WINDOW_VISIBLE: AtomicBool = AtomicBool::new(false);
static SYSTEM_VAR_COUNT: AtomicU32 = AtomicU32::new(0);

#[allow(non_snake_case)]
#[unsafe(export_name = "PluginStart")]
pub unsafe extern "stdcall" fn PluginStart(_a_owner: uintptr_t) {
    let opl_path = ".\\plugins\\omsilogger.opl";

    // Read varlist from .opl file manually
    let mut var_names = Vec::new();
    let mut string_var_names = Vec::new();
    let mut hotkey_val = 0x79; // Default F10
    if let Ok(file) = File::open(opl_path) {
        let reader = BufReader::new(file);
        let mut in_varlist = false;
        let mut in_systemvarlist = false;
        let mut in_stringvarlist = false;
        let mut in_hotkey = false;
        let mut count = 0;
        let mut expected_count = 0;
        let mut system_var_names = Vec::new();

        for line in reader.lines() {
            if let Ok(l) = line {
                let l = l.trim();
                if l == "[varlist]" {
                    in_varlist = true;
                    in_systemvarlist = false;
                    in_hotkey = false;
                    count = 0;
                    expected_count = 0;
                    continue;
                }
                if l == "[systemvarlist]" {
                    in_systemvarlist = true;
                    in_varlist = false;
                    in_stringvarlist = false;
                    in_hotkey = false;
                    count = 0;
                    expected_count = 0;
                    continue;
                }
                if l == "[stringvarlist]" {
                    in_stringvarlist = true;
                    in_varlist = false;
                    in_systemvarlist = false;
                    in_hotkey = false;
                    count = 0;
                    expected_count = 0;
                    continue;
                }
                if l == "[hotkey]" {
                    in_hotkey = true;
                    in_varlist = false;
                    in_systemvarlist = false;
                    in_stringvarlist = false;
                    continue;
                }
                if in_systemvarlist {
                    if expected_count == 0 {
                        if let Ok(c) = l.parse::<usize>() {
                            expected_count = c;
                            if expected_count == 0 { in_systemvarlist = false; }
                        }
                    } else {
                        system_var_names.push(l.to_string());
                        count += 1;
                        if count >= expected_count {
                            in_systemvarlist = false;
                        }
                    }
                }
                if in_varlist {
                    if expected_count == 0 {
                        if let Ok(c) = l.parse::<usize>() {
                            expected_count = c;
                            if expected_count == 0 { in_varlist = false; }
                        }
                    } else {
                        var_names.push(l.to_string());
                        count += 1;
                        if count >= expected_count {
                            in_varlist = false;
                        }
                    }
                }
                if in_stringvarlist {
                    if expected_count == 0 {
                        if let Ok(c) = l.parse::<usize>() {
                            expected_count = c;
                            if expected_count == 0 { in_stringvarlist = false; }
                        }
                    } else {
                        string_var_names.push(l.to_string());
                        count += 1;
                        if count >= expected_count {
                            in_stringvarlist = false;
                        }
                    }
                }
                if in_hotkey {
                    if l.starts_with("0x") {
                        if let Ok(h) = u32::from_str_radix(&l[2..], 16) {
                            hotkey_val = h;
                        }
                    } else if let Ok(h) = l.parse::<u32>() {
                        hotkey_val = h;
                    }
                }
            }
        }
        SYSTEM_VAR_COUNT.store(system_var_names.len() as u32, Relaxed);
        let mut combined_names = system_var_names;
        combined_names.extend(var_names);
        var_names = combined_names;
    }

    let _ = VAR_NAMES.set(var_names);
    if let Ok(mut values) = STRING_VAR_VALUES.write() {
        *values = vec![String::new(); string_var_names.len()];
    }
    let _ = STRING_VAR_NAMES.set(string_var_names);
    let _ = HOTKEY.set(hotkey_val);

    // GUI Thread
    thread::spawn(|| {
        run_gui();
    });

    // Hotkey Listener Thread
    thread::spawn(move || {
        let hotkey = *HOTKEY.get().unwrap_or(&0x79);
        let mut pressed = false;
        loop {
            unsafe {
                let state =
                    windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(hotkey as i32);
                if (state as u16 & 0x8000) != 0 {
                    if !pressed {
                        let current = WINDOW_VISIBLE.load(Relaxed);
                        WINDOW_VISIBLE.store(!current, Relaxed);
                        pressed = true;
                    }
                } else {
                    pressed = false;
                }
            }
            thread::sleep(Duration::from_millis(50));
        }
    });

    thread::spawn(move || {
        let mut last_values = vec![0u32; SHARED_ARRAY_SIZE];
        let now_date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let log_file_path = format!("omsilogger_{}.txt", now_date);

        // Write a start message
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file_path)
        {
            let version = env!("CARGO_PKG_VERSION");
            let start_msg = format!(
                "{} --- omsilogger v{} started ---\n",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                version
            );
            let _ = file.write_all(start_msg.as_bytes());
        }

        loop {
            for i in 0..SHARED_ARRAY_SIZE {
                let current_val = SHARED_ARRAY[i].load(Relaxed);
                if current_val != last_values[i] {
                    let var_name = VAR_NAMES
                        .get()
                        .and_then(|v| v.get(i).cloned())
                        .unwrap_or_else(|| format!("Unknown_{}", i));

                    let now = chrono::Local::now();
                    let log_line = format!(
                        "{}: {}, old: {}, new: {}\n",
                        now.format("%Y-%m-%d %H:%M:%S"),
                        var_name,
                        last_values[i],
                        current_val
                    );

                    if let Ok(mut file) = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&log_file_path)
                    {
                        let _ = file.write_all(log_line.as_bytes());
                    }

                    if let Ok(mut messages) = LOG_MESSAGES.lock() {
                        messages.push(log_line.clone());
                        if messages.len() > 100 {
                            messages.remove(0);
                        }
                    }

                    last_values[i] = current_val;
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
    });
}

#[allow(non_snake_case, unused_variables)]
#[unsafe(export_name = "AccessVariable")]
pub unsafe extern "stdcall" fn AccessVariable(
    variableIndex: u16,
    value: *const c_float,
    writeValue: *const bool,
) {
    let offset = SYSTEM_VAR_COUNT.load(Relaxed) as usize;
    let index = variableIndex as usize + offset;
    if index < SHARED_ARRAY_SIZE {
        unsafe {
            let f = *value;
            let a = f.abs().round() as u32;
            SHARED_ARRAY[index].store(a, Relaxed);
        }
    }
}

#[allow(non_snake_case, unused_variables)]
#[unsafe(export_name = "AccessStringVariable")]
pub unsafe extern "stdcall" fn AccessStringVariable(
    variable_index: u16,
    pw_char_ptr: *const u16,
    _write_value: *mut bool,
) {
    if pw_char_ptr.is_null() {
        return;
    }

    let mut len = 0;
    while *pw_char_ptr.add(len) != 0 {
        len += 1;
    }
    let new_slice = slice::from_raw_parts(pw_char_ptr, len);

    let index = variable_index as usize;

    if let Ok(values) = STRING_VAR_VALUES.read() {
        if index >= values.len() {
            return;
        }

        if !is_equal_utf16_to_str(new_slice, &values[index]) {
            drop(values);

            let mut values_write = match STRING_VAR_VALUES.write() {
                Ok(v) => v,
                Err(_) => return,
            };

            let new_string = OsString::from_wide(new_slice).to_string_lossy().into_owned();

            let var_name = STRING_VAR_NAMES
                .get()
                .and_then(|names| names.get(index).cloned())
                .unwrap_or_else(|| format!("StringIndex_{}", index));

            let now = chrono::Local::now();
            let log_line = format!(
                "{}: {} = {}\n",
                now.format("%Y-%m-%d %H:%M:%S"),
                var_name,
                new_string
            );

            let now_date = now.format("%Y-%m-%d").to_string();
            let log_file_path = format!("omsilogger_{}.txt", now_date);

            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file_path)
            {
                let _ = file.write_all(log_line.as_bytes());
            }

            if let Ok(mut messages) = LOG_MESSAGES.lock() {
                messages.push(log_line.clone());
                if messages.len() > 100 {
                    messages.remove(0);
                }
            }

            values_write[index] = new_string;
        }
    }
}

fn is_equal_utf16_to_str(utf16: &[u16], s: &str) -> bool {
    let mut s_utf16 = s.encode_utf16();
    let mut u_iter = utf16.iter().copied();

    loop {
        match (u_iter.next(), s_utf16.next()) {
            (Some(a), Some(b)) if a == b => continue,
            (None, None) => return true,
            _ => return false,
        }
    }
}

#[allow(non_snake_case, unused_variables)]
#[unsafe(export_name = "AccessSystemVariable")]
pub unsafe extern "stdcall" fn AccessSystemVariable(
    variableIndex: u16,
    value: *const c_float,
    writeValue: *const bool,
) {
    let index = variableIndex as usize;
    if index < SHARED_ARRAY_SIZE {
        unsafe {
            let f = *value;
            let a = f.abs().round() as u32;
            SHARED_ARRAY[index].store(a, Relaxed);
        }
    }
}

#[allow(non_snake_case, unused_variables)]
#[unsafe(export_name = "AccessTrigger")]
pub unsafe extern "stdcall" fn AccessTrigger(variableIndex: u16, triggerScript: *const bool) {}

#[allow(non_snake_case)]
#[unsafe(export_name = "PluginFinalize")]
pub unsafe extern "stdcall" fn PluginFinalize() {}

fn run_gui() {
    use windows::{
        core::*, Win32::Graphics::Gdi::*,
        Win32::System::LibraryLoader::*, Win32::UI::WindowsAndMessaging::*,
    };

    unsafe {
        let instance = GetModuleHandleW(None).unwrap();
        let window_class = w!("OmsiLoggerWindow");

        let wc = WNDCLASSW {
            hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
            hInstance: instance.into(),
            lpszClassName: window_class,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        RegisterClassW(&wc);

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST,
            window_class,
            w!("OMSI Logger"),
            WS_POPUP | WS_BORDER,
            10,
            10,
            600,
            400,
            None,
            None,
            Some(instance.into()),
            None,
        )
        .expect("Failed to create window");

        let mut msg = MSG::default();
        loop {
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            let is_visible = WINDOW_VISIBLE.load(Relaxed);
            let current_visible = IsWindowVisible(hwnd).as_bool();

            if is_visible && !current_visible {
                let _ = ShowWindow(hwnd, SW_SHOW);
                let _ = SetForegroundWindow(hwnd);
            } else if !is_visible && current_visible {
                let _ = ShowWindow(hwnd, SW_HIDE);
            }

            if is_visible {
                let _ = InvalidateRect(Some(hwnd), None, false);
            }

            thread::sleep(Duration::from_millis(16));
        }
    }
}

extern "system" fn wndproc(
    window: windows::Win32::Foundation::HWND,
    message: u32,
    wparam: windows::Win32::Foundation::WPARAM,
    lparam: windows::Win32::Foundation::LPARAM,
) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::{Foundation::*, Graphics::Gdi::*, UI::WindowsAndMessaging::*};

    unsafe {
        match message {
            WM_ERASEBKGND => {
                LRESULT(1) // Tell Windows we handled it to prevent flickering
            }
            WM_PAINT => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(window, &mut ps);

                let mut rect = RECT::default();
                let _ = GetClientRect(window, &mut rect);

                // Double Buffering
                let mem_hdc = CreateCompatibleDC(Some(hdc));
                let mem_bitmap =
                    CreateCompatibleBitmap(hdc, rect.right - rect.left, rect.bottom - rect.top);
                let old_bitmap = SelectObject(mem_hdc, HGDIOBJ(mem_bitmap.0));

                let hbr = CreateSolidBrush(COLORREF(0x000000)); // Black background
                FillRect(mem_hdc, &rect, hbr);
                let _ = DeleteObject(HGDIOBJ(hbr.0));

                SetTextColor(mem_hdc, COLORREF(0x00FF00)); // Green text
                SetBkMode(mem_hdc, TRANSPARENT);

                if let Ok(messages) = LOG_MESSAGES.lock() {
                    let mut y = rect.bottom - 25;
                    for msg in messages.iter().rev() {
                        let mut r = RECT {
                            left: 5,
                            top: y,
                            right: rect.right - 5,
                            bottom: y + 20,
                        };
                        let mut wide_msg: Vec<u16> =
                            msg.encode_utf16().chain(std::iter::once(0)).collect();
                        DrawTextW(mem_hdc, &mut wide_msg, &mut r, DT_LEFT | DT_SINGLELINE);
                        y -= 20;
                        if y < 0 {
                            break;
                        }
                    }
                }

                let _ = BitBlt(
                    hdc,
                    0,
                    0,
                    rect.right - rect.left,
                    rect.bottom - rect.top,
                    Some(mem_hdc),
                    0,
                    0,
                    SRCCOPY,
                );

                let _ = SelectObject(mem_hdc, old_bitmap);
                let _ = DeleteObject(HGDIOBJ(mem_bitmap.0));
                let _ = DeleteDC(mem_hdc);

                let _ = EndPaint(window, &ps);
                LRESULT(0)
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(window, message, wparam, lparam),
        }
    }
}
