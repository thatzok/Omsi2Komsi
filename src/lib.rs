extern crate user32;
extern crate winapi;

use windows::{Win32::Foundation::*, Win32::System::SystemServices::*};

use user32::MessageBoxA;
// use winapi::winuser::{MB_OK, MB_ICONINFORMATION};

use std::ffi::CString;

use libc::c_char;
use libc::c_float;

#[allow(non_camel_case_types)]
pub type uintptr_t = usize;

// __declspec(dllexport) void __stdcall PluginStart(void* aOwner)
// This function links our DLL to Omsi 2, thus it cannot be Safe (raw pointers, etc...)
#[allow(non_snake_case, unused_variables)]
#[no_mangle]
#[export_name = "PluginStart"]
pub unsafe extern "stdcall" fn PluginStart(aOwner: uintptr_t) {
    let lp_text = CString::new("Plugin started!").unwrap();
    let lp_caption = CString::new("Omsi2Komsi").unwrap();

    unsafe {
        // Create a message box
        MessageBoxA(
            std::ptr::null_mut(),
            lp_text.as_ptr(),
            lp_caption.as_ptr(),
            Default::default(),
        );
    }
}

// __declspec(dllexport) void __stdcall PluginFinalize()
// This function links our DLL to Omsi 2, thus it cannot be Safe (raw pointers, etc...)
#[allow(non_snake_case, unused_variables)]
#[no_mangle]
#[export_name = "PluginFinalize"]
pub unsafe extern "stdcall" fn PluginFinalize() {
    let lp_text = CString::new("Plugin Finalize!").unwrap();
    let lp_caption = CString::new("Omsi2Komsi").unwrap();

    unsafe {
        // Create a message box
        MessageBoxA(
            std::ptr::null_mut(),
            lp_text.as_ptr(),
            lp_caption.as_ptr(),
            Default::default(),
        );
    }
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
