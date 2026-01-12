#[cfg(not(target_arch = "x86"))]
compile_error!("This plugin must be compiled for x86 (32-bit) to be compatible with OMSI!");

use std::thread;
use std::time::{Duration};
use core::sync::atomic::Ordering::Relaxed;
use libc::c_char;
use libc::c_float;
use std::sync::atomic::{AtomicU32};
use std::fs::OpenOptions;
use std::io::Write;
use std::fs::File;
use std::io::{BufRead, BufReader};

use std::sync::OnceLock;

#[allow(non_camel_case_types)]
pub type uintptr_t = usize;

const SHARED_ARRAY_SIZE: usize = 100;

static SHARED_ARRAY: [AtomicU32; SHARED_ARRAY_SIZE] = {
    const ARRAY_REPEAT_VALUE: AtomicU32 = AtomicU32::new(0);
    [ARRAY_REPEAT_VALUE; SHARED_ARRAY_SIZE]
};

static VAR_NAMES: OnceLock<Vec<String>> = OnceLock::new();

#[allow(non_snake_case)]
#[unsafe(export_name = "PluginStart")]
pub unsafe extern "stdcall" fn PluginStart(_a_owner: uintptr_t) {
    let opl_path = ".\\plugins\\omsilogger.opl";

    // Read varlist from .opl file manually
    let mut var_names = Vec::new();
    if let Ok(file) = File::open(opl_path) {
        let reader = BufReader::new(file);
        let mut in_varlist = false;
        let mut count = 0;
        let mut expected_count = 0;

        for line in reader.lines() {
            if let Ok(l) = line {
                let l = l.trim();
                if l == "[varlist]" {
                    in_varlist = true;
                    continue;
                }
                if in_varlist {
                    if expected_count == 0 {
                        if let Ok(c) = l.parse::<usize>() {
                            expected_count = c;
                        }
                    } else {
                        var_names.push(l.to_string());
                        count += 1;
                        if count >= expected_count {
                            break;
                        }
                    }
                }
            }
        }
    }
    
    let _ = VAR_NAMES.set(var_names);

    thread::spawn(move || {
        let mut last_values = vec![0u32; SHARED_ARRAY_SIZE];
        let now_date = chrono::Local::now().format("%Y-%m-%d").to_string();
        let log_file_path = format!("omsilogger_{}.log", now_date);

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
                    let var_name = VAR_NAMES.get()
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
    variableIndex: u8,
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
#[unsafe(export_name = "AccessStringVariable")]
pub unsafe extern "stdcall" fn AccessStringVariable(
    variableIndex: u8,
    firstCharacterAddress: *const c_char,
    writeValue: *const bool,
) {
}

#[allow(non_snake_case, unused_variables)]
#[unsafe(export_name = "AccessSystemVariable")]
pub unsafe extern "stdcall" fn AccessSystemVariable(
    variableIndex: u8,
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
pub unsafe extern "stdcall" fn AccessTrigger(
    variableIndex: u8,
    triggerScript: *const bool,
) {
}

#[allow(non_snake_case)]
#[unsafe(export_name = "PluginFinalize")]
pub unsafe extern "stdcall" fn PluginFinalize() {
}
