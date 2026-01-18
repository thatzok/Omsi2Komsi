use crate::komsi::KomsiCommandKind;
use crate::komsi::build_komsi_command;
use crate::komsi::build_komsi_command_u8;
use crate::komsi::build_komsi_command_eol;

pub trait VehicleLogger {
    fn log(&self, msg: String);
}

#[derive(Debug)]
pub struct VehicleState {
    pub ignition: u8,
    pub engine: u8,
    pub doors: u8,
    pub speed: u32,
    pub maxspeed: u32,
    pub fuel: u32,
    pub indicator: u8,
    pub fixing_brake: u8,
    pub lights_warning: u8,
    pub lights_main: u8,
    pub lights_front_door: u8,
    pub lights_second_door: u8,
    pub lights_third_door: u8,
    pub lights_stop_request: u8,
    pub lights_stop_brake: u8,
    pub lights_high_beam: u8,
    pub battery_light: u8,
}

impl Default for VehicleState {
    fn default() -> Self {
        Self {
            ignition: 0,
            engine: 0,
            doors: 0,
            speed: 0,
            indicator: 0,
            fixing_brake: 0,
            lights_warning: 0,
            lights_main: 0,
            lights_front_door: 0,
            lights_second_door: 0,
            lights_third_door: 0,
            lights_stop_request: 0,
            maxspeed: 0,
            lights_high_beam: 0,
            fuel: 0,
            lights_stop_brake: 0,
            battery_light: 0,
        }
    }
}

impl VehicleState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn print(&self) {
        print!("ignition:{} ", self.ignition);
        print!("engine:{} ", self.engine);
        print!("indicator:{} ", self.indicator);
        print!("fuel:{} ", self.fuel);
        print!("warn:{} ", self.lights_warning);
        print!("lights:{} ", self.lights_main);
        print!("lights-highbeam:{} ", self.lights_high_beam);
        print!("stop:{} ", self.lights_stop_request);
        print!("fixingbrake:{} ", self.fixing_brake);
        print!("stopbrake:{} ", self.lights_stop_brake);
        print!("doors:{} ", self.doors);
        print!("door1:{} ", self.lights_front_door);
        print!("door2:{} ", self.lights_second_door);
        print!("door3:{} ", self.lights_third_door);
        print!("speed:{} ", self.speed);
        print!("maxspeed:{} ", self.maxspeed);
        print!("batterylight:{} ", self.battery_light);
        println!(" ");
    }

    pub fn compare(&self, new: &VehicleState, force: bool, logger: Option<&dyn VehicleLogger>) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![0; 0];

        if (self.ignition != new.ignition) || force {
            if let Some(l) = logger {
                l.log(format!("ignition: {} -> {} ", self.ignition, new.ignition));
            }
            let mut b = build_komsi_command_u8(KomsiCommandKind::Ignition, new.ignition);
            buffer.append(&mut b);
        }

        if (self.engine != new.engine) || force {
            if let Some(l) = logger {
                l.log(format!("engine: {} -> {} ", self.engine, new.engine));
            }
            let mut b = build_komsi_command_u8(KomsiCommandKind::Engine, new.engine);
            buffer.append(&mut b);
        }

        if (self.doors != new.doors) || force {
            if let Some(l) = logger {
                l.log(format!("doors: {} -> {} ", self.doors, new.doors));
            }
            let mut b = build_komsi_command_u8(KomsiCommandKind::PassengerDoorsOpen, new.doors);
            buffer.append(&mut b);
        }

        if (self.fixing_brake != new.fixing_brake) || force {
            if let Some(l) = logger {
                l.log(format!(
                    "fixing_brake: {} -> {} ",
                    self.fixing_brake, new.fixing_brake
                ));
            }
            let mut b = build_komsi_command_u8(KomsiCommandKind::FixingBrake, new.fixing_brake);
            buffer.append(&mut b);
        }

        if (self.indicator != new.indicator) || force {
            if let Some(l) = logger {
                l.log(format!("indicator: {} -> {} ", self.indicator, new.indicator));
            }
            let mut b = build_komsi_command_u8(KomsiCommandKind::Indicator, new.indicator);
            buffer.append(&mut b);
        }

        if (self.lights_warning != new.lights_warning) || force {
            if let Some(l) = logger {
                l.log(format!(
                    "lights_warning: {} -> {} ",
                    self.lights_warning, new.lights_warning
                ));
            }
            let mut b = build_komsi_command_u8(KomsiCommandKind::LightsWarning, new.lights_warning);
            buffer.append(&mut b);
        }

        if (self.lights_main != new.lights_main) || force {
            if let Some(l) = logger {
                l.log(format!("lights_main: {} -> {} ", self.lights_main, new.lights_main));
            }
            let mut b = build_komsi_command_u8(KomsiCommandKind::LightsMain, new.lights_main);
            buffer.append(&mut b);
        }

        if (self.lights_stop_request != new.lights_stop_request) || force {
            if let Some(l) = logger {
                l.log(format!(
                    "lights_stop_request: {} -> {} ",
                    self.lights_stop_request, new.lights_stop_request
                ));
            }
            let mut b = build_komsi_command_u8(
                KomsiCommandKind::LightsStopRequest,
                new.lights_stop_request,
            );
            buffer.append(&mut b);
        }

        if (self.lights_stop_brake != new.lights_stop_brake) || force {
            if let Some(l) = logger {
                l.log(format!(
                    "lights_stop_brake: {} -> {} ",
                    self.lights_stop_brake, new.lights_stop_brake
                ));
            }
            let mut b =
                build_komsi_command_u8(KomsiCommandKind::LightsStopBrake, new.lights_stop_brake);
            buffer.append(&mut b);
        }

        if (self.lights_front_door != new.lights_front_door) || force {
            if let Some(l) = logger {
                l.log(format!(
                    "lights_front_door: {} -> {} ",
                    self.lights_front_door, new.lights_front_door
                ));
            }
            let mut b =
                build_komsi_command_u8(KomsiCommandKind::LightsFrontDoor, new.lights_front_door);
            buffer.append(&mut b);
        }

        if (self.lights_second_door != new.lights_second_door) || force {
            if let Some(l) = logger {
                l.log(format!(
                    "lights_second_door: {} -> {} ",
                    self.lights_second_door, new.lights_second_door
                ));
            }
            let mut b =
                build_komsi_command_u8(KomsiCommandKind::LightsSecondDoor, new.lights_second_door);
            buffer.append(&mut b);
        }

        if (self.lights_third_door != new.lights_third_door) || force {
            if let Some(l) = logger {
                l.log(format!(
                    "lights_third_door: {} -> {} ",
                    self.lights_third_door, new.lights_third_door
                ));
            }
            let mut b =
                build_komsi_command_u8(KomsiCommandKind::LightsThirdDoor, new.lights_third_door);
            buffer.append(&mut b);
        }

        if (self.lights_high_beam != new.lights_high_beam) || force {
            if let Some(l) = logger {
                l.log(format!(
                    "lights_high_beam: {} -> {} ",
                    self.lights_high_beam, new.lights_high_beam
                ));
            }
            let mut b =
                build_komsi_command_u8(KomsiCommandKind::LightsHighBeam, new.lights_high_beam);
            buffer.append(&mut b);
        }

        if (self.fuel != new.fuel) || force {
            if let Some(l) = logger {
                l.log(format!("fuel:  {} -> {} ", self.fuel, new.fuel));
            }
            let mut b = build_komsi_command(KomsiCommandKind::Fuel, new.fuel);
            buffer.append(&mut b);
        }

        if (self.speed != new.speed) || force {
            if let Some(l) = logger {
                l.log(format!("speed:  {} -> {} ", self.speed, new.speed));
            }
            let mut b = build_komsi_command(KomsiCommandKind::Speed, new.speed);
            buffer.append(&mut b);
        }

        if (self.maxspeed != new.maxspeed) || force {
            if let Some(l) = logger {
                l.log(format!("maxspeed:  {} -> {} ", self.maxspeed, new.maxspeed));
            }
            let mut b = build_komsi_command(KomsiCommandKind::MaxSpeed, new.maxspeed);
            buffer.append(&mut b);
        }

        if (self.battery_light != new.battery_light) || force {
            if let Some(l) = logger {
                l.log(format!(
                    "batterylight:  {} -> {} ",
                    self.battery_light, new.battery_light
                ));
            }
            let mut b = build_komsi_command_u8(KomsiCommandKind::BatteryLight, new.battery_light);
            buffer.append(&mut b);
        }

        // zeilenende hinzu, wenn buffer nicht leer
        if buffer.len() > 0 {
            let mut b = build_komsi_command_eol();
            buffer.append(&mut b);
        }

        buffer
    }
}
