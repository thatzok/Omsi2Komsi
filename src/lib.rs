#![allow(dead_code)]
#[cfg(not(target_arch = "x86"))]
compile_error!("This plugin must be compiled for x86 (32-bit) to be compatible with OMSI!");

use configparser::ini::Ini;
use core::sync::atomic::Ordering::Relaxed;
use libc::c_char;
use libc::c_float;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use atomic_float::AtomicF32;

use komsi::komsi::{build_komsi_command, build_komsi_command_eol, KomsiCommand};
use komsi::vehicle::{VehicleLogger, VehicleState};

#[allow(non_camel_case_types)]
pub type uintptr_t = usize;

struct GuiLogger;

impl VehicleLogger for GuiLogger {
    fn log(&self, msg: String) {
        log_message(msg);
    }
}

const SHARED_ARRAY_SIZE: usize = 30;

use std::sync::RwLock;

static VAR_NAMES: RwLock<Vec<String>> = RwLock::new(Vec::new());
static HOTKEY: OnceLock<u32> = OnceLock::new();
static LOG_MESSAGES: Mutex<Vec<String>> = Mutex::new(Vec::new());
static WINDOW_VISIBLE: AtomicBool = AtomicBool::new(false);
static SERIAL_PORT_ENABLED: AtomicBool = AtomicBool::new(false);
static DEBUG_MODE: AtomicBool = AtomicBool::new(false);
static SYSTEM_VAR_COUNT: AtomicUsize = AtomicUsize::new(0);

static SERIAL_PORTS: Mutex<Vec<Option<Box<dyn serialport::SerialPort>>>> = Mutex::new(Vec::new());

#[unsafe(no_mangle)]
pub extern "C" fn log_message_extern(msg: *const c_char) {
    if msg.is_null() {
        return;
    }
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
    door_enable: AtomicF32,
    time: AtomicF32,
    day: AtomicF32,
    month: AtomicF32,
    year: AtomicF32,
    odometer: AtomicF32,
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
    door_enable: AtomicF32::new(0.0),
    time: AtomicF32::new(0.0),
    day: AtomicF32::new(0.0),
    month: AtomicF32::new(0.0),
    year: AtomicF32::new(0.0),
    odometer: AtomicF32::new(0.0),
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
    DoorEnable,
    Time,
    Day,
    Month,
    Year,
    Odometer,
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
            x if x == OmsiDataField::DoorEnable as usize => OmsiDataField::DoorEnable,
            x if x == OmsiDataField::Time as usize => OmsiDataField::Time,
            x if x == OmsiDataField::Day as usize => OmsiDataField::Day,
            x if x == OmsiDataField::Month as usize => OmsiDataField::Month,
            x if x == OmsiDataField::Year as usize => OmsiDataField::Year,
            x if x == OmsiDataField::Odometer as usize => OmsiDataField::Odometer,
            _ => OmsiDataField::None,
        }
    }
}

static DATA_MAPPING: [AtomicUsize; SHARED_ARRAY_SIZE] =
    [const { AtomicUsize::new(OmsiDataField::None as usize) }; SHARED_ARRAY_SIZE];

fn run_gui() {
    use windows::{
        core::*, Win32::Graphics::Gdi::*,
        Win32::System::LibraryLoader::*, Win32::UI::WindowsAndMessaging::*,
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
                        let _ = DrawTextW(mem_hdc, &mut wide_msg, &mut r, DT_LEFT | DT_SINGLELINE);
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

pub fn get_vehicle_state_from_omsi(engineonvalue: u8) -> VehicleState {
    let mut s = VehicleState::new();

    s.ignition = OMSI_DATA.ignition.load(Relaxed) > 0.0;

    if !s.ignition {
        return s;
    }

    let engineval = OMSI_DATA.battery.load(Relaxed) as u8;
    s.battery_light = engineval > 0;

    // we use the value of the battery light for engine on/off state
    if engineval == engineonvalue {
        s.engine = true;
    }

    s.speed = OMSI_DATA.speed.load(Relaxed) as u32;

    s.lights_front_door = OMSI_DATA.front_door.load(Relaxed) > 0.0;
    s.lights_second_door = OMSI_DATA.second_door.load(Relaxed) > 0.0;
    s.lights_third_door = OMSI_DATA.third_door.load(Relaxed) > 0.0;

    s.door_enable = OMSI_DATA.door_enable.load(Relaxed) > 0.0;

    // Türschleife erst setzen und dann errechnen
    s.doors = OMSI_DATA.door_loop.load(Relaxed) > 0.0;
    if s.lights_front_door || s.lights_second_door || s.lights_third_door || s.door_enable {
        s.doors = true;
    }

    s.lights_stop_request = OMSI_DATA.stop_request.load(Relaxed) > 0.0;

    let ail = OMSI_DATA.light_main.load(Relaxed) as u8;

    s.lights_high_beam = OMSI_DATA.lights_high_beam.load(Relaxed) > 0.0;

    if ail > 0 {
        // TODO search different OMSI variable, because this one is always "2" when high beam is active
        s.lights_main = true;
    }

    s.fixing_brake = OMSI_DATA.fixing_brake.load(Relaxed) > 0.0;

    let ail_val = OMSI_DATA.indicator_left.load(Relaxed) as u8;
    let air_val = OMSI_DATA.indicator_right.load(Relaxed) as u8;

    let aisum = ail_val + air_val;

    if aisum == 1 {
        // left OR right
        // Blinker
        if ail_val == 1 {
            s.indicator = 1
        } else {
            s.indicator = 2;
        }
    }

    if aisum > 1 {
        // left AND right means warning lights
        s.lights_warning = true;
    }

    // fuel is in percent, so we multiply by 100
    let f = OMSI_DATA.fuel.load(Relaxed);
    s.fuel = (f.abs() * 100.0).round() as u8;

    s.lights_stop_brake = OMSI_DATA.stop_brake.load(Relaxed) > 0.0;

    let time_sec = OMSI_DATA.time.load(Relaxed) as u32;
    s.datetime.hour = (time_sec / 3600) as u8;
    s.datetime.min = ((time_sec % 3600) / 60) as u8;
    s.datetime.sec = (time_sec % 60) as u8;

    s.datetime.day = OMSI_DATA.day.load(Relaxed) as u8;
    s.datetime.month = OMSI_DATA.month.load(Relaxed) as u8;
    s.datetime.year = OMSI_DATA.year.load(Relaxed) as u16;

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

    let mut config_path = ".\\plugins\\omsi2komsi.opl";
    if !std::path::Path::new(config_path).exists() {
        config_path = "omsi2komsi.opl";
    }

    let mut config = Ini::new();
    let _ = config.load(config_path);

    let baudrate = config.getint("omsi2komsi", "baudrate").ok().flatten().unwrap_or(115200) as u32;
    let mut portnames = Vec::new();
    if let Some(p) = config.get("omsi2komsi", "portname") {
        if !p.is_empty() {
            portnames.push(p);
        }
    }
    for i in 2..=5 {
        let key = format!("portname{}", i);
        if let Some(p) = config.get("omsi2komsi", &key) {
            if !p.is_empty() {
                portnames.push(p);
            }
        }
    }

    if portnames.is_empty() {
        portnames.push("com1".to_string());
    }

    let serial_enabled = config
        .getbool("omsi2komsi", "serialportenabled")
        .ok()
        .flatten()
        .unwrap_or(false);
    SERIAL_PORT_ENABLED.store(serial_enabled, Relaxed);

    let debug_mode = config
        .getbool("omsi2komsi", "debug")
        .ok()
        .flatten()
        .unwrap_or(false);
    DEBUG_MODE.store(debug_mode, Relaxed);

    let engineonvalue = config
        .getint("omsi2komsi", "engineonvalue")
        .ok()
        .flatten()
        .unwrap_or(1) as u8;

    if let Ok(content) = std::fs::read_to_string(config_path) {
        log_message(format!("Loading config from {}", config_path));
        let mut in_varlist = false;
        let mut in_systemvarlist = false;
        let mut in_datamappings = false;
        let mut in_hotkey = false;
        let mut hotkey_val = 0x79; // Default F10
        let mut temp_system_var_names: Vec<String> = Vec::new();
        let mut temp_var_names: Vec<String> = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            if line == "[varlist]" {
                in_varlist = true;
                in_systemvarlist = false;
                in_datamappings = false;
                in_hotkey = false;
                continue;
            }
            if line == "[systemvarlist]" {
                in_varlist = false;
                in_systemvarlist = true;
                in_datamappings = false;
                in_hotkey = false;
                continue;
            }
            if line == "[datamappings]" {
                in_varlist = false;
                in_systemvarlist = false;
                in_datamappings = true;
                in_hotkey = false;
                continue;
            }
            if line == "[hotkey]" {
                in_varlist = false;
                in_systemvarlist = false;
                in_datamappings = false;
                in_hotkey = true;
                continue;
            }
            if line.starts_with('[') {
                in_varlist = false;
                in_systemvarlist = false;
                in_datamappings = false;
                in_hotkey = false;
                continue;
            }

            if in_systemvarlist {
                let var_name = line.to_lowercase();
                // skip if the line is just the count (integer)
                if temp_system_var_names.is_empty() && var_name.parse::<u32>().is_ok() {
                    continue;
                }

                temp_system_var_names.push(var_name);
            }

            if in_varlist {
                let var_name = line.to_lowercase();
                // skip if the line is just the count (integer)
                if temp_var_names.is_empty() && var_name.parse::<u32>().is_ok() {
                    continue;
                }

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
                        "doorenable" => OmsiDataField::DoorEnable,
                        "doorloop" => OmsiDataField::DoorLoop,
                        "time" => OmsiDataField::Time,
                        "day" => OmsiDataField::Day,
                        "month" => OmsiDataField::Month,
                        "year" => OmsiDataField::Year,
                        "odometer" => OmsiDataField::Odometer,
                        _ => OmsiDataField::None,
                    };

                    if field != OmsiDataField::None {
                        for source_part in source.split(',') {
                            let source_part = source_part.trim();
                            
                            // Map source_part to index
                            let mut found_idx = None;
                            for (i, name) in temp_system_var_names.iter().enumerate() {
                                if name == source_part {
                                    found_idx = Some(i);
                                    break;
                                }
                            }
                            if found_idx.is_none() {
                                for (i, name) in temp_var_names.iter().enumerate() {
                                    if name == source_part {
                                        found_idx = Some(temp_system_var_names.len() + i);
                                        break;
                                    }
                                }
                            }

                            if let Some(idx) = found_idx {
                                if idx < SHARED_ARRAY_SIZE {
                                    log_message(format!(
                                        "Mapping variable '{}' (index {}) to {:?}",
                                        source_part, idx, field
                                    ));
                                    DATA_MAPPING[idx].store(field as usize, Relaxed);
                                }
                            }
                        }
                    }
                }
            }
        }

        SYSTEM_VAR_COUNT.store(temp_system_var_names.len(), Relaxed);
        let mut combined_names = temp_system_var_names;
        combined_names.extend(temp_var_names);

        if let Ok(mut var_names) = VAR_NAMES.write() {
            *var_names = combined_names;
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

    let mut vehicle_state = VehicleState::new();

    let portnames_clone = portnames.clone();
    {
        let mut ports = SERIAL_PORTS.lock().unwrap();
        for _ in 0..portnames_clone.len() {
            ports.push(None);
        }
    }

    thread::spawn(move || {
        loop {
            // get data from OMSI
            let newstate = get_vehicle_state_from_omsi(engineonvalue);

            let verbose = WINDOW_VISIBLE.load(Relaxed);
            let debug = DEBUG_MODE.load(Relaxed);
            // compare and create cmd buf
            let logger = if verbose {
                Some(&GuiLogger as &dyn VehicleLogger)
            } else {
                None
            };
            let cmdbuf = vehicle_state.compare(&newstate, false, logger);

            // log when debug=true in config section omsi2komsi
            if verbose && debug && cmdbuf.len() > 0  {
                // simple log of the command buffer or some representation
                log_message(format!("Sent {} bytes: {:?}", cmdbuf.len(), cmdbuf));
            }

            // replace after compare for next round
            vehicle_state = newstate;

            if cmdbuf.len() > 0 && SERIAL_PORT_ENABLED.load(Relaxed) {
                let mut ports_guard = SERIAL_PORTS.lock().unwrap();
                for (i, portname_item) in portnames_clone.iter().enumerate() {
                    if ports_guard[i].is_none() {
                        match serialport::new(portname_item, baudrate)
                            .timeout(Duration::from_millis(10))
                            .open()
                        {
                            Ok(mut p) => {
                                log_message(format!("Serial port {} opened successfully", portname_item));
                                // send SimulatorType:OMSI
                                let mut init_buf = Vec::new();
                                let simulator_type = KomsiCommand::SimulatorType(0);
                                init_buf.extend_from_slice(&build_komsi_command(simulator_type));
                                init_buf.extend_from_slice(&build_komsi_command_eol());
                                if let Err(e) = p.write_all(&init_buf) {
                                    log_message(format!("Failed to send init string to {}: {}", portname_item, e));
                                }
                                ports_guard[i] = Some(p);
                            }
                            Err(e) => {
                                log_message(format!("Failed to open serial port {}: {}", portname_item, e));
                            }
                        }
                    }

                    if let Some(ref mut p) = ports_guard[i] {
                        if let Err(e) = p.write_all(&cmdbuf) {
                            log_message(format!("Serial write error on {}: {}. Closing port.", portname_item, e));
                            ports_guard[i] = None;
                        }
                    }
                }
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
    variableIndex: u16,
    value: *const c_float,
    writeValue: *const bool,
) {
    let offset = SYSTEM_VAR_COUNT.load(Relaxed);
    let index = variableIndex as usize + offset;

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
        OmsiDataField::DoorEnable => OMSI_DATA.door_enable.store(val_to_store, Relaxed),
        OmsiDataField::Time => OMSI_DATA.time.store(val_to_store, Relaxed),
        OmsiDataField::Day => OMSI_DATA.day.store(val_to_store, Relaxed),
        OmsiDataField::Month => OMSI_DATA.month.store(val_to_store, Relaxed),
        OmsiDataField::Year => OMSI_DATA.year.store(val_to_store, Relaxed),
        OmsiDataField::Odometer => OMSI_DATA.odometer.store(val_to_store, Relaxed),
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
    variableIndex: u16,
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
    variableIndex: u16,
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
        OmsiDataField::DoorEnable => OMSI_DATA.door_enable.store(val_to_store, Relaxed),
        OmsiDataField::Time => OMSI_DATA.time.store(val_to_store, Relaxed),
        OmsiDataField::Day => OMSI_DATA.day.store(val_to_store, Relaxed),
        OmsiDataField::Month => OMSI_DATA.month.store(val_to_store, Relaxed),
        OmsiDataField::Year => OMSI_DATA.year.store(val_to_store, Relaxed),
        OmsiDataField::Odometer => OMSI_DATA.odometer.store(val_to_store, Relaxed),
        OmsiDataField::None => {}
    }
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
pub unsafe extern "stdcall" fn AccessTrigger(variableIndex: u16, triggerScript: *const bool) {}

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
