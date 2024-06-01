mod api;
mod komsi;
mod opts;
mod serial;
mod vehicle;

use windows::{Win32::Foundation::*, Win32::System::SystemServices::*};

use user32::MessageBoxA;
// use winapi::winuser::{MB_OK, MB_ICONINFORMATION};

use std::ffi::CString;
use std::io;

use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};

use core::sync::atomic::Ordering::Relaxed;
use libc::c_char;
use libc::c_float;
use std::sync::atomic::{self, AtomicU32};

// use crate::opts::Opts;
use crate::vehicle::compare_vehicle_states;
use crate::vehicle::init_vehicle_state;
use crate::vehicle::VehicleState;

#[allow(non_camel_case_types)]
pub type uintptr_t = usize;

// static SHARED_PLUGIN_NUM: AtomicU32 = AtomicU32::new(0);

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

pub fn get_vehicle_state_from_omsi() -> VehicleState {
    let mut s = init_vehicle_state();

    let ignition = &SHARED_ARRAY[0];

    s.ignition = ignition.load(Relaxed) as u8;

    if s.ignition == 0 {
        return s;
    }

    let engine = &SHARED_ARRAY[1];
    s.engine = 1 - engine.load(Relaxed) as u8; // TODO manchmal invertieren (1-engine)

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

    // if s.lights_high_beam > 0 {
    //     // Fernlicht erhöht AI_LIGHT um 1, wenn gesetzt
    //    ail = ail - 1;
    // }

    if ail > 0 {
        // TODO andere variable, weil  bei fernlicht immer auf "2"
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
        // warnblinker
        s.lights_warning = 1;
    }

    let fuel = &SHARED_ARRAY[12];
    s.fuel = fuel.load(Relaxed);

    let stop_brake = &SHARED_ARRAY[13];
    s.lights_stop_brake = stop_brake.load(Relaxed) as u8;

    return s;
}

// __declspec(dllexport) void __stdcall PluginStart(void* aOwner)
// This function links our DLL to Omsi 2, thus it cannot be Safe (raw pointers, etc...)
#[allow(non_snake_case, unused_variables)]
#[no_mangle]
#[export_name = "PluginStart"]
pub unsafe extern "stdcall" fn PluginStart(aOwner: uintptr_t) {
    let mut port = serialport::new("COM22", 115200)
        .open()
        .expect("Failed to open serial port");

    let mut vehicle_state = init_vehicle_state();

    thread::spawn(move || loop {
        // get data from OMSI
        let newstate = get_vehicle_state_from_omsi();

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

// __declspec(dllexport) void __stdcall AccessVariable(unsigned short varindex, float* value, bool* write)
// This function links our DLL to Omsi 2, thus it cannot be Safe (raw pointers, etc...)
#[allow(non_snake_case, unused_variables)]
#[no_mangle]
#[export_name = "AccessVariable"]
pub unsafe extern "stdcall" fn AccessVariable(
    variableIndex: u8,
    value: *const c_float,
    writeValue: *const bool,
) {
    let index = variableIndex as usize;

    if index < SHARED_ARRAY_SIZE {
        let f = *value;
        if index == 12 {
            // sonderbehandlung bei "tankinhalt in %"
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

// __declspec(dllexport) void __stdcall AccessStringVariable(unsigned short varindex, wchar_t* value, bool* write)
// This function links our DLL to Omsi 2, thus it cannot be Safe (raw pointers, etc...)
#[allow(non_snake_case, unused_variables)]
#[no_mangle]
#[export_name = "AccessStringVariable"]
pub unsafe extern "stdcall" fn AccessStringVariable(
    variableIndex: u8,
    firstCharacterAddress: *const c_char,
    writeValue: *const bool,
) {
}

// __declspec(dllexport) void __stdcall AccessSystemVariable(unsigned short varindex, float* value)
// This function links our DLL to Omsi 2, thus it cannot be Safe (raw pointers, etc...)
#[allow(non_snake_case, unused_variables)]
#[no_mangle]
#[export_name = "AccessSystemVariable"]
pub unsafe extern "stdcall" fn AccessSystemVariable(
    variableIndex: u8,
    value: *const c_float,
    writeValue: *const bool,
) {
}

// __declspec(dllexport) void __stdcall AccessTrigger(unsigned short triggerindex, bool* active)
// This function links our DLL to Omsi 2, thus it cannot be Safe (raw pointers, etc...)
#[allow(non_snake_case, unused_variables)]
#[no_mangle]
#[export_name = "AccessTrigger"]
pub unsafe extern "stdcall" fn AccessTrigger(variableIndex: u8, triggerScript: *const bool) {}

// __declspec(dllexport) void __stdcall PluginFinalize()
// This function links our DLL to Omsi 2, thus it cannot be Safe (raw pointers, etc...)
#[allow(non_snake_case, unused_variables)]
#[no_mangle]
#[export_name = "PluginFinalize"]
pub unsafe extern "stdcall" fn PluginFinalize() {}
