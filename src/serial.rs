use crate::opts::Opts;
use serialport::{available_ports, SerialPortType};
use std::result::Result;

pub fn show_serial_comports() {
    match available_ports() {
        Ok(ports) => {
            match ports.len() {
                0 => println!("Kein port gefunden."),
                1 => println!("1 port gefunden:"),
                n => println!("{} ports gefunden:", n),
            };

            for p in ports {
                print!("  {}", p.port_name);
                match p.port_type {
                    SerialPortType::UsbPort(info) => {
                        print!(" Typ: USB");
                        print!(
                            "   Hersteller: {}",
                            info.manufacturer.as_ref().map_or("", String::as_str)
                        );
                        println!(
                            "   Produkt: {}",
                            info.product.as_ref().map_or("", String::as_str)
                        );
                    }
                    SerialPortType::BluetoothPort => {
                        println!("    Typ: Bluetooth");
                    }
                    SerialPortType::PciPort => {
                        println!("    Typ: PCI");
                    }
                    SerialPortType::Unknown => {
                        println!("    Typ: Unknown");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("{:?}", e);
            eprintln!("Error listing serial ports");
        }
    }
}

pub fn find_serial_comports(opts: &Opts) -> Result<String, &str> {
    // println!("Suche ...");

    match available_ports() {
        Ok(ports) => {
            for p in ports {
                // print!("  {}", p.port_name);
                match p.port_type {
                    SerialPortType::UsbPort(info) => {
                        let zeile = format!(
                            "{} {} {}",
                            p.port_name,
                            info.manufacturer.as_ref().map_or("", String::as_str),
                            info.product.as_ref().map_or("", String::as_str)
                        );

                        // println!("{:?}", zeile);
                        let aa = zeile.as_str().to_lowercase();
                        let a = aa.as_str();
                        let bb = opts.find.as_ref().unwrap().as_str().to_lowercase();
                        let b = bb.as_str();

                        let pp = p.port_name.to_string();

                        if a.contains(b) {
                            // println!("GEFUNDEN");
                            return Ok(pp);
                        }
                    }
                    SerialPortType::BluetoothPort => {}
                    SerialPortType::PciPort => {}
                    SerialPortType::Unknown => {}
                }
            }
        }
        Err(e) => {
            eprintln!("{:?}", e);
            eprintln!("Error listing serial ports");
        }
    }
    Err("Nicht gefunden.")
}
