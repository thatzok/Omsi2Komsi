#![allow(dead_code)]
#[cfg(not(target_arch = "x86"))]
compile_error!("This plugin must be compiled for x86 (32-bit) to be compatible with OMSI!");

mod komsi;
mod vehicle;

use configparser::ini::Ini;
use core::sync::atomic::Ordering::Relaxed;
use libc::c_char;
use libc::c_float;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::{OnceLock, Mutex};
use std::thread;
use std::time::Duration;

use atomic_float::AtomicF32;

use crate::vehicle::VehicleState;

#[allow(non_camel_case_types)]
pub type uintptr_t = usize;

const SHARED_ARRAY_SIZE: usize = 30;

use std::sync::RwLock;

static VAR_NAMES: RwLock<Vec<String>> = RwLock::new(Vec::new());
static HOTKEY: OnceLock<u32> = OnceLock::new();
static LOG_MESSAGES: Mutex<Vec<String>> = Mutex::new(Vec::new());
static WINDOW_VISIBLE: AtomicBool = AtomicBool::new(false);

#[unsafe(no_mangle)]
pub extern "C" fn log_message_extern(msg: *const c_char) {
    if msg.is_null() { return; }
    unsafe {
        let c_str = std::ffi::CStr::from_ptr(msg);
        if let Ok(s) = c_str.to_str() {
            log_message(s.to_string());
        }
    }
}

fn log_message(msg: String) {
    if let Ok(mut messages) = LOG_MESSAGES.lock() {
        messages.push(msg);
        if messages.len() > 20 {
            messages.remove(0);
        }
    }
}

struct OmsiData {
    ignition: AtomicF32,
    battery: AtomicF32,
    speed: AtomicF32,
    front_door: AtomicF32,
    second_door: AtomicF32,
    third_door: AtomicF32,
    stop_request: AtomicF32,
    light_main: AtomicF32,
    lights_high_beam: AtomicF32,
    fixing_brake: AtomicF32,
    indicator_left: AtomicF32,
    indicator_right: AtomicF32,
    fuel: AtomicF32,
    stop_brake: AtomicF32,
    door_loop: AtomicF32,
}

static OMSI_DATA: OmsiData = OmsiData {
    ignition: AtomicF32::new(0.0),
    battery: AtomicF32::new(0.0),
    speed: AtomicF32::new(0.0),
    front_door: AtomicF32::new(0.0),
    second_door: AtomicF32::new(0.0),
    third_door: AtomicF32::new(0.0),
    stop_request: AtomicF32::new(0.0),
    light_main: AtomicF32::new(0.0),
    lights_high_beam: AtomicF32::new(0.0),
    fixing_brake: AtomicF32::new(0.0),
    indicator_left: AtomicF32::new(0.0),
    indicator_right: AtomicF32::new(0.0),
    fuel: AtomicF32::new(0.0),
    stop_brake: AtomicF32::new(0.0),
    door_loop: AtomicF32::new(0.0),
};

#[repr(usize)]
#[derive(Clone, Copy, PartialEq, Debug)]
enum OmsiDataField {
    None,
    Ignition,
    Battery,
    Speed,
    FrontDoor,
    SecondDoor,
    ThirdDoor,
    StopRequest,
    LightMain,
    LightsHighBeam,
    FixingBrake,
    IndicatorLeft,
    IndicatorRight,
    Fuel,
    StopBrake,
    DoorLoop,
}

impl From<usize> for OmsiDataField {
    fn from(v: usize) -> Self {
        match v {
            x if x == OmsiDataField::Ignition as usize => OmsiDataField::Ignition,
            x if x == OmsiDataField::Battery as usize => OmsiDataField::Battery,
            x if x == OmsiDataField::Speed as usize => OmsiDataField::Speed,
            x if x == OmsiDataField::FrontDoor as usize => OmsiDataField::FrontDoor,
            x if x == OmsiDataField::SecondDoor as usize => OmsiDataField::SecondDoor,
            x if x == OmsiDataField::ThirdDoor as usize => OmsiDataField::ThirdDoor,
            x if x == OmsiDataField::StopRequest as usize => OmsiDataField::StopRequest,
            x if x == OmsiDataField::LightMain as usize => OmsiDataField::LightMain,
            x if x == OmsiDataField::LightsHighBeam as usize => OmsiDataField::LightsHighBeam,
            x if x == OmsiDataField::FixingBrake as usize => OmsiDataField::FixingBrake,
            x if x == OmsiDataField::IndicatorLeft as usize => OmsiDataField::IndicatorLeft,
            x if x == OmsiDataField::IndicatorRight as usize => OmsiDataField::IndicatorRight,
            x if x == OmsiDataField::Fuel as usize => OmsiDataField::Fuel,
            x if x == OmsiDataField::StopBrake as usize => OmsiDataField::StopBrake,
            x if x == OmsiDataField::DoorLoop as usize => OmsiDataField::DoorLoop,
            _ => OmsiDataField::None,
        }
    }
}

static DATA_MAPPING: [AtomicUsize; SHARED_ARRAY_SIZE] =
    [const { AtomicUsize::new(OmsiDataField::None as usize) }; SHARED_ARRAY_SIZE];

fn run_gui() {
    use windows::{
        core::*,
        Win32::Graphics::Gdi::*,
        Win32::UI::WindowsAndMessaging::*,
        Win32::System::LibraryLoader::*,
    };

    unsafe {
        let instance = GetModuleHandleW(None).unwrap();
        let window_class = w!("Omsi2KomsiLogWindow");

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
            w!("Omsi2Komsi Log"),
            WS_POPUP | WS_BORDER,
            10, 10, 600, 400,
            None,
            None,
            Some(instance.into()),
            None,
        ).expect("Failed to create window");

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

extern "system" fn wndproc(window: windows::Win32::Foundation::HWND, message: u32, wparam: windows::Win32::Foundation::WPARAM, lparam: windows::Win32::Foundation::LPARAM) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        UI::WindowsAndMessaging::*,
    };

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
                let mem_bitmap = CreateCompatibleBitmap(hdc, rect.right - rect.left, rect.bottom - rect.top);
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
                        let mut wide_msg: Vec<u16> = msg.encode_utf16().chain(std::iter::once(0)).collect();
                        DrawTextW(mem_hdc, &mut wide_msg, &mut r, DT_LEFT | DT_SINGLELINE);
                        y -= 20;
                        if y < 0 { break; }
                    }
                }

                let _ = BitBlt(hdc, 0, 0, rect.right - rect.left, rect.bottom - rect.top, Some(mem_hdc), 0, 0, SRCCOPY);
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

pub fn get_vehicle_state_from_omsi(engineonvalue: u8) -> VehicleState {
    let mut s = VehicleState::new();

    s.ignition = OMSI_DATA.ignition.load(Relaxed) as u8;

    if s.ignition == 0 {
        return s;
    }

    let engineval = OMSI_DATA.battery.load(Relaxed) as u8;
    s.battery_light = engineval;

    // we use the value of the battery light for engine on/off state
    if engineval == engineonvalue {
        s.engine = 1;
    }

    s.speed = OMSI_DATA.speed.load(Relaxed) as u32;

    s.lights_front_door = OMSI_DATA.front_door.load(Relaxed) as u8;
    s.lights_second_door = OMSI_DATA.second_door.load(Relaxed) as u8;
    s.lights_third_door = OMSI_DATA.third_door.load(Relaxed) as u8;

    // Türschleife erst setzen und dann errechnen
    s.doors = OMSI_DATA.door_loop.load(Relaxed) as u8;
    if s.lights_front_door + s.lights_second_door + s.lights_third_door > 0 {
        s.doors = 1;
    }

    s.lights_stop_request = OMSI_DATA.stop_request.load(Relaxed) as u8;

    let ail = OMSI_DATA.light_main.load(Relaxed) as u8;

    s.lights_high_beam = OMSI_DATA.lights_high_beam.load(Relaxed) as u8;

    if ail > 0 {
        // TODO search different OMSI variable, because this one is always "2" when high beam is active
        s.lights_main = 1;
    }

    s.fixing_brake = OMSI_DATA.fixing_brake.load(Relaxed) as u8;

    let ail_val = OMSI_DATA.indicator_left.load(Relaxed) as u8;
    let air_val = OMSI_DATA.indicator_right.load(Relaxed) as u8;

    let aisum = ail_val + air_val;

    if aisum == 1 {   // left OR right
        // Blinker
        if ail_val == 1 {
            s.indicator = 1
        } else {
            s.indicator = 2;
        }
    }

    if aisum > 1 {    // left AND right means warning lights
        s.lights_warning = 1;
    }

    // fuel is in percent, so we multiply by 100
    let f = OMSI_DATA.fuel.load(Relaxed);
    s.fuel = (f.abs() * 100.0).round() as u32;


    s.lights_stop_brake = OMSI_DATA.stop_brake.load(Relaxed) as u8;

    s
}

/// This function is called when the plugin is loaded by Omsi 2.
///
/// Original C declaration:
/// ```c
/// __declspec(dllexport) void __stdcall PluginStart(void* aOwner)
/// ```
///
/// # Safety
/// This function links our DLL to Omsi 2, thus it cannot be Safe (raw pointers, etc...)
#[allow(non_snake_case, unused_variables)]
// #[unsafe(no_mangle)]
#[unsafe(export_name = "PluginStart")]
pub unsafe extern "stdcall" fn PluginStart(aOwner: uintptr_t) {
    // load config

    // TODO checking for file not found and elements not found
    // now we get config ini
    let mut config = Ini::new();
    let _ = config.load(".\\plugins\\omsi2komsi.opl");

    let baudrate = config.getint("omsi2komsi", "baudrate").unwrap().unwrap() as u32;
    let portname = config.get("omsi2komsi", "portname").unwrap();

    let engineonvalue = config
        .getint("omsi2komsi", "engineonvalue")
        .unwrap()
        .unwrap() as u8;

    if let Ok(content) = std::fs::read_to_string(".\\plugins\\omsi2komsi.opl") {
        let mut in_varlist = false;
        let mut in_datamappings = false;
        let mut in_hotkey = false;
        let mut hotkey_val = 0x79; // Default F10
        let mut var_map: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();
        let mut temp_var_names: Vec<String> = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            if line == "[varlist]" {
                in_varlist = true;
                in_datamappings = false;
                in_hotkey = false;
                continue;
            }
            if line == "[datamappings]" {
                in_varlist = false;
                in_datamappings = true;
                in_hotkey = false;
                continue;
            }
            if line == "[hotkey]" {
                in_varlist = false;
                in_datamappings = false;
                in_hotkey = true;
                continue;
            }
            if line.starts_with('[') {
                in_varlist = false;
                in_datamappings = false;
                in_hotkey = false;
                continue;
            }

            if in_varlist {
                // first line after [varlist] is the count, skip it
                if let Ok(_) = line.parse::<u32>() {
                    continue;
                }

                let var_name = line.to_lowercase();
                var_map.insert(var_name.clone(), temp_var_names.len());
                temp_var_names.push(var_name);
            }

            if in_hotkey {
                if line.starts_with("0x") {
                    if let Ok(h) = u32::from_str_radix(&line[2..], 16) {
                        hotkey_val = h;
                    }
                } else if let Ok(h) = line.parse::<u32>() {
                    hotkey_val = h;
                }
            }

            if in_datamappings {
                let parts: Vec<&str> = line.split('=').collect();
                if parts.len() == 2 {
                    let target = parts[0].trim().to_lowercase();
                    let source = parts[1].trim().to_lowercase();

                    if let Some(&idx) = var_map.get(&source) {
                        let field = match target.as_str() {
                            "ignition" => OmsiDataField::Ignition,
                            "battery" => OmsiDataField::Battery,
                            "speed" => OmsiDataField::Speed,
                            "frontdoor" => OmsiDataField::FrontDoor,
                            "seconddoor" => OmsiDataField::SecondDoor,
                            "thirddoor" => OmsiDataField::ThirdDoor,
                            "stoprequest" => OmsiDataField::StopRequest,
                            "lightmain" => OmsiDataField::LightMain,
                            "lightshighbeam" => OmsiDataField::LightsHighBeam,
                            "fixingbrake" => OmsiDataField::FixingBrake,
                            "indicatorleft" => OmsiDataField::IndicatorLeft,
                            "indicatorright" => OmsiDataField::IndicatorRight,
                            "fuel" => OmsiDataField::Fuel,
                            "stopbrake" => OmsiDataField::StopBrake,
                            _ => OmsiDataField::None,
                        };

                        if field != OmsiDataField::None && idx < SHARED_ARRAY_SIZE {
                            DATA_MAPPING[idx].store(field as usize, Relaxed);
                        }
                    }
                }
            }
        }

        if let Ok(mut var_names) = VAR_NAMES.write() {
            *var_names = temp_var_names;
        }
        let _ = HOTKEY.set(hotkey_val);
    }

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
                let state = windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState(hotkey as i32);
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

    let mut port = serialport::new(portname, baudrate)
        .open()
        .expect("Failed to open serial port");

    let mut vehicle_state = VehicleState::new();

    // send SimulatorType:OMSI
    let string = "O0\x0a";
    let buffer = string.as_bytes();
    let _ = port.write(buffer);

    thread::spawn(move || {
        loop {
            // get data from OMSI
            let newstate = get_vehicle_state_from_omsi(engineonvalue);

            let verbose = WINDOW_VISIBLE.load(Relaxed);

            // compare and create cmd buf
            let cmdbuf = vehicle_state.compare(&newstate, false, verbose);

            if verbose && cmdbuf.len() > 0 {
                // simple log of the command buffer or some representation
                log_message(format!("Sent {} bytes: {:?}", cmdbuf.len(), cmdbuf));
            }

            // replace after compare for next round
            vehicle_state = newstate;

            if cmdbuf.len() > 0 {
                // Write to serial port
                let _ = port.write(&cmdbuf);
            }

            thread::sleep(Duration::from_millis(100));
        }
    });
}

/// This function is called by Omsi 2 to access variables from the plugin.
///
/// Original C declaration:
/// ```c
/// __declspec(dllexport) void __stdcall AccessVariable(unsigned short varindex, float* value, bool* write)
/// ```
///
/// # Parameters
/// * `variableIndex` - The index of the variable to access
/// * `value` - Pointer to the float value to read or write
/// * `writeValue` - Pointer to a boolean indicating whether to write the value
///
/// # Safety
/// This function links our DLL to Omsi 2, thus it cannot be Safe (raw pointers, etc...)
#[allow(non_snake_case, unused_variables)]
// #[unsafe(no_mangle)]
#[unsafe(export_name = "AccessVariable")]
pub unsafe extern "stdcall" fn AccessVariable(
    variableIndex: u8,
    value: *const c_float,
    writeValue: *const bool,
) {
    let index = variableIndex as usize;

    let field = {
        let mut ofield = OmsiDataField::None;
        if let Ok(names) = VAR_NAMES.read() {
            if index < names.len() {
                if index < SHARED_ARRAY_SIZE {
                    let field_idx = DATA_MAPPING[index].load(Relaxed);
                    ofield = OmsiDataField::from(field_idx);
                }
            }
        }
        ofield
    };

    if field == OmsiDataField::None {
        return;
    }

    let f = unsafe { *value };
    let val_to_store = f as f32;

    match field {
        OmsiDataField::Ignition => OMSI_DATA.ignition.store(val_to_store, Relaxed),
        OmsiDataField::Battery => OMSI_DATA.battery.store(val_to_store, Relaxed),
        OmsiDataField::Speed => OMSI_DATA.speed.store(val_to_store, Relaxed),
        OmsiDataField::FrontDoor => OMSI_DATA.front_door.store(val_to_store, Relaxed),
        OmsiDataField::SecondDoor => OMSI_DATA.second_door.store(val_to_store, Relaxed),
        OmsiDataField::ThirdDoor => OMSI_DATA.third_door.store(val_to_store, Relaxed),
        OmsiDataField::StopRequest => OMSI_DATA.stop_request.store(val_to_store, Relaxed),
        OmsiDataField::LightMain => OMSI_DATA.light_main.store(val_to_store, Relaxed),
        OmsiDataField::LightsHighBeam => OMSI_DATA.lights_high_beam.store(val_to_store, Relaxed),
        OmsiDataField::FixingBrake => OMSI_DATA.fixing_brake.store(val_to_store, Relaxed),
        OmsiDataField::IndicatorLeft => OMSI_DATA.indicator_left.store(val_to_store, Relaxed),
        OmsiDataField::IndicatorRight => OMSI_DATA.indicator_right.store(val_to_store, Relaxed),
        OmsiDataField::Fuel => OMSI_DATA.fuel.store(val_to_store, Relaxed),
        OmsiDataField::StopBrake => OMSI_DATA.stop_brake.store(val_to_store, Relaxed),
        OmsiDataField::DoorLoop => OMSI_DATA.door_loop.store(val_to_store, Relaxed),
        OmsiDataField::None => {}
    }
}

/// This function is called by Omsi 2 to access string variables from the plugin.
///
/// Original C declaration:
/// ```c
/// __declspec(dllexport) void __stdcall AccessStringVariable(unsigned short varindex, wchar_t* value, bool* write)
/// ```
///
/// # Parameters
/// * `variableIndex` - The index of the string variable to access
/// * `firstCharacterAddress` - Pointer to the first character of the string
/// * `writeValue` - Pointer to a boolean indicating whether to write the value
///
/// # Safety
/// This function links our DLL to Omsi 2, thus it cannot be Safe (raw pointers, etc...)
#[allow(non_snake_case, unused_variables)]
// #[unsafe(no_mangle)]
#[unsafe(export_name = "AccessStringVariable")]
pub unsafe extern "stdcall" fn AccessStringVariable(
    variableIndex: u8,
    firstCharacterAddress: *const c_char,
    writeValue: *const bool,
) {
}

/// This function is called by Omsi 2 to access system variables from the plugin.
///
/// Original C declaration:
/// ```c
/// __declspec(dllexport) void __stdcall AccessSystemVariable(unsigned short varindex, float* value)
/// ```
///
/// # Parameters
/// * `variableIndex` - The index of the system variable to access
/// * `value` - Pointer to the float value to read or write
/// * `writeValue` - Pointer to a boolean indicating whether to write the value
///
/// # Safety
/// This function links our DLL to Omsi 2, thus it cannot be Safe (raw pointers, etc...)
#[allow(non_snake_case, unused_variables)]
// #[unsafe(no_mangle)]
#[unsafe(export_name = "AccessSystemVariable")]
pub unsafe extern "stdcall" fn AccessSystemVariable(
    variableIndex: u8,
    value: *const c_float,
    writeValue: *const bool,
) {
}

/// This function is called by Omsi 2 to access triggers from the plugin.
///
/// Original C declaration:
/// ```c
/// __declspec(dllexport) void __stdcall AccessTrigger(unsigned short triggerindex, bool* active)
/// ```
///
/// # Parameters
/// * `variableIndex` - The index of the trigger to access
/// * `triggerScript` - Pointer to a boolean indicating whether the trigger is active
///
/// # Safety
/// This function links our DLL to Omsi 2, thus it cannot be Safe (raw pointers, etc...)
#[allow(non_snake_case, unused_variables)]
// #[unsafe(no_mangle)]
#[unsafe(export_name = "AccessTrigger")]
pub unsafe extern "stdcall" fn AccessTrigger(variableIndex: u8, triggerScript: *const bool) {}

/// This function is called when the plugin is unloaded by Omsi 2.
///
/// Original C declaration:
/// ```c
/// __declspec(dllexport) void __stdcall PluginFinalize()
/// ```
///
/// # Safety
/// This function links our DLL to Omsi 2, thus it cannot be Safe (raw pointers, etc...)
#[allow(non_snake_case, unused_variables)]
// #[unsafe(no_mangle)]
#[unsafe(export_name = "PluginFinalize")]
pub unsafe extern "stdcall" fn PluginFinalize() {}
