mod api;
mod komsi;
mod opts;
mod serial;
mod vehicle;

// extern crate user32;
// extern crate winapi;

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

use crate::opts::Opts;
use crate::vehicle::VehicleState;
use crate::vehicle::compare_vehicle_states;
use crate::vehicle::init_vehicle_state;


#[allow(non_camel_case_types)]
pub type uintptr_t = usize;

static SHARED_PLUGIN_NUM: AtomicU32 = AtomicU32::new(0);

pub fn get_vehicle_state_from_omsi() -> VehicleState {
    let mut s = init_vehicle_state();

    s.lights_warning = SHARED_PLUGIN_NUM.load(Relaxed) as u8;

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

// __declspec(dllexport) void __stdcall PluginFinalize()
// This function links our DLL to Omsi 2, thus it cannot be Safe (raw pointers, etc...)
#[allow(non_snake_case, unused_variables)]
#[no_mangle]
#[export_name = "PluginFinalize"]
pub unsafe extern "stdcall" fn PluginFinalize() {}

// __declspec(dllexport) void __stdcall AccessTrigger(unsigned short triggerindex, bool* active)
// This function links our DLL to Omsi 2, thus it cannot be Safe (raw pointers, etc...)
#[allow(non_snake_case, unused_variables)]
#[no_mangle]
#[export_name = "AccessTrigger"]
pub unsafe extern "stdcall" fn AccessTrigger(variableIndex: u8, triggerScript: *const bool) {}

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
    if variableIndex == 0 {
        let f = *value;
        let a = f.round() as u32;

        SHARED_PLUGIN_NUM.store(a, Relaxed)
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
