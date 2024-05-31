extern crate user32;
extern crate winapi;

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

#[allow(non_camel_case_types)]
pub type uintptr_t = usize;

static SHARED_PLUGIN_NUM: AtomicU32 = AtomicU32::new(0);

pub fn build_komsi_command(cmd: u8, wert: u32) -> Vec<u8> {
    let cmd_u8 = cmd as u8;
    let mut buffer: Vec<u8> = vec![cmd_u8];
    let mut s: Vec<u8> = wert.to_string().as_bytes().to_vec();

    buffer.append(&mut s);

    return buffer;
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

    // Clone the port
    // let mut portclone = port.try_clone().expect("Failed to clone");

    // let lp_text = CString::new("Plugin started!").unwrap();
    //let lp_caption = CString::new("Omsi2Komsi").unwrap();

    //unsafe {
    //   // Create a message box
    //  MessageBoxA(
    //     std::ptr::null_mut(),
    //    lp_text.as_ptr(),
    //   lp_caption.as_ptr(),
    //  Default::default(),
    //  );
    // }

    let mut altvar: u32 = 0;

    thread::spawn(move || loop {
        // let string = "CHANGE\x0a";
        // let buffer = string.as_bytes();

        let neuvar = SHARED_PLUGIN_NUM.load(Relaxed);

        if altvar != neuvar {
            let mut buffer: Vec<u8> = vec![0; 0];
            let mut b = build_komsi_command(70, neuvar);
            buffer.append(&mut b);
            let cmd = 10 as u8;
            let mut cb: Vec<u8> = vec![cmd];
            buffer.append(&mut cb);

            // Write to serial port
            let _ = port.write(&buffer);
        }
        altvar = neuvar;

        thread::sleep(Duration::from_millis(100));
    });
}

// __declspec(dllexport) void __stdcall PluginFinalize()
// This function links our DLL to Omsi 2, thus it cannot be Safe (raw pointers, etc...)
#[allow(non_snake_case, unused_variables)]
#[no_mangle]
#[export_name = "PluginFinalize"]
pub unsafe extern "stdcall" fn PluginFinalize() {
    // let lp_text = CString::new("Plugin Finalize!").unwrap();
    // let lp_caption = CString::new("Omsi2Komsi").unwrap();

    // unsafe {
    // // Create a message box
    // MessageBoxA(
    //   std::ptr::null_mut(),
    //   lp_text.as_ptr(),
    // lp_caption.as_ptr(),
    //   Default::default(),
    //  );
    //  }
}

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
