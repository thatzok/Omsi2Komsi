#![allow(dead_code)]

mod komsi;
mod vehicle;

// use windows::{Win32::Foundation::*, Win32::System::SystemServices::*};

// use user32::MessageBoxA;
// use winapi::winuser::{MB_OK, MB_ICONINFORMATION};

// use std::ffi::CString;
// use std::io;

use std::thread;
// use std::thread::sleep;
use std::time::{Duration};

use core::sync::atomic::Ordering::Relaxed;
use libc::c_char;
use libc::c_float;
use std::sync::atomic::{ AtomicU32};

use configparser::ini::Ini;

// use crate::opts::Opts;
use crate::vehicle::compare_vehicle_states;
use crate::vehicle::init_vehicle_state;
use crate::vehicle::VehicleState;

#[allow(non_camel_case_types)]
pub type uintptr_t = usize;

const SHARED_ARRAY_SIZE: usize = 30;

static SHARED_ARRAY: [AtomicU32; SHARED_ARRAY_SIZE] = [
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
    AtomicU32::new(0),
];

pub fn get_vehicle_state_from_omsi(engineonvalue: u8) -> VehicleState {
    let mut s = init_vehicle_state();

    let ignition = &SHARED_ARRAY[0];

    s.ignition = ignition.load(Relaxed) as u8;

    if s.ignition == 0 {
        return s;
    }

    let engine = &SHARED_ARRAY[1];
    let engineval = engine.load(Relaxed) as u8;
    s.battery_light = engineval;

    // we use the value of the battery light for engine on/off state
    if engineval == engineonvalue {
        s.engine = 1;
    }

    let speed = &SHARED_ARRAY[2];
    s.speed = speed.load(Relaxed);

    let door_light_1 = &SHARED_ARRAY[3];
    let door_light_2 = &SHARED_ARRAY[4];
    let door_light_3 = &SHARED_ARRAY[5];

    s.lights_front_door = door_light_1.load(Relaxed) as u8;
    s.lights_second_door = door_light_2.load(Relaxed) as u8;
    s.lights_third_door = door_light_3.load(Relaxed) as u8;

    // Türschleife errechnen
    if s.lights_front_door + s.lights_second_door + s.lights_third_door > 0 {
        s.doors = 1;
    }

    let haltewunsch = &SHARED_ARRAY[6];
    s.lights_stop_request = haltewunsch.load(Relaxed) as u8;

    let ailight = &SHARED_ARRAY[7];
    let ail = ailight.load(Relaxed) as u8;

    let lightsfern = &SHARED_ARRAY[8];
    s.lights_high_beam = lightsfern.load(Relaxed) as u8;

    if ail > 0 {
        // TODO search different OMSI variable, because this one is always "2" when high beam is active
        s.lights_main = 1;
    }

    let fixing_brake = &SHARED_ARRAY[9];
    s.fixing_brake = fixing_brake.load(Relaxed) as u8;

    let aileft = &SHARED_ARRAY[10];
    let airight = &SHARED_ARRAY[11];
    let ail = aileft.load(Relaxed);
    let air = airight.load(Relaxed);

    let aisum = ail + air;

    if aisum == 1 {
        // Blinker
        if ail == 1 {
            s.indicator = 1
        } else {
            s.indicator = 2;
        }
    }

    if aisum > 1 {
        // when left and right both are on its warning lights
        s.lights_warning = 1;
    }

    let fuel = &SHARED_ARRAY[12];
    s.fuel = fuel.load(Relaxed);

    let stop_brake = &SHARED_ARRAY[13];
    s.lights_stop_brake = stop_brake.load(Relaxed) as u8;

    return s;
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
    let _ = config.load(".\\plugins\\Omsi2Komsi.opl");

    let baudrate = config.getint("omsi2komsi", "baudrate").unwrap().unwrap() as u32;
    let portname = config.get("omsi2komsi", "portname").unwrap();

    let engineonvalue = config
        .getint("omsi2komsi", "engineonvalue")
        .unwrap()
        .unwrap() as u8;

    let mut port = serialport::new(portname, baudrate)
        .open()
        .expect("Failed to open serial port");

    let mut vehicle_state = init_vehicle_state();

    // send SimulatorType:OMSI
    let string = "O0\x0a";
    let buffer = string.as_bytes();
    let _ = port.write(buffer);

    thread::spawn(move || loop {
        // get data from OMSI
        let newstate = get_vehicle_state_from_omsi(engineonvalue);

        // compare and create cmd buf
        let cmdbuf = compare_vehicle_states(&vehicle_state, &newstate, false);

        // replace after compare for next round
        vehicle_state = newstate;

        if cmdbuf.len() > 0 {
            // Write to serial port
            let _ = port.write(&cmdbuf);
        }

        thread::sleep(Duration::from_millis(100));
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

    unsafe {
        if index < SHARED_ARRAY_SIZE {
            let f = *value;
            if index == 12 {
                // special case for fuel tank, because it is percentage
                let hun = 100 as f32;
                let b = f.abs() * hun;
                let a = b.round() as u32;
                let vari = &SHARED_ARRAY[index];
                vari.store(a, Relaxed);
            } else {
                let a = f.abs().round() as u32;
                let vari = &SHARED_ARRAY[index];
                vari.store(a, Relaxed);
            }
        }
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
