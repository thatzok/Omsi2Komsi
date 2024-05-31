use crate::api::ApiVehicleType;
use crate::komsi::build_komsi_command;
use crate::komsi::build_komsi_command_u8;
use crate::komsi::KomsiCommandKind;
use crate::opts::Opts;

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
    pub lights_stop_request: u8,
    pub lights_stop_brake: u8,
    pub lights_high_beam: u8,
}

pub fn print_vehicle_state(v: &VehicleState) {
    print!("ignition:{} ", v.ignition);
    print!("engine:{} ", v.engine);
    print!("indicator:{} ", v.indicator);
    print!("fuel:{} ", v.fuel);
    print!("warn:{} ", v.lights_warning);
    print!("lights:{} ", v.lights_main);
    print!("lights-highbeam:{} ", v.lights_high_beam);
    print!("stop:{} ", v.lights_stop_request);
    print!("fixingbrake:{} ", v.fixing_brake);
    print!("stopbrake:{} ", v.lights_stop_brake);
    print!("doors:{} ", v.doors);
    print!("door1:{} ", v.lights_front_door);
    print!("door2:{} ", v.lights_second_door);
    print!("speed:{} ", v.speed);
    print!("maxspeed:{} ", v.maxspeed);

    println!(" ");
}

pub fn init_vehicle_state() -> VehicleState {
    let s = VehicleState {
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
        lights_stop_request: 0,
        maxspeed: 0,
        lights_high_beam: 0,
        fuel: 0,
        lights_stop_brake: 0,
    };
    return s;
}

pub fn get_vehicle_state(av: ApiVehicleType) -> VehicleState {
    let mut s = init_vehicle_state();

    match av.ignition_enabled.as_str() {
        "true" => s.ignition = 1,
        _ => s.ignition = 0,
    }

    match av.engine_started.as_str() {
        "true" => s.engine = 1,
        _ => s.engine = 0,
    }

    match av.warning_lights.as_str() {
        "true" => s.lights_warning = 1,
        _ => s.lights_warning = 0,
    }

    match av.passenger_doors_open.as_str() {
        "true" => s.doors = 1,
        _ => s.doors = 0,
    }

    match av.fixing_brake.as_str() {
        "true" => s.fixing_brake = 1,
        _ => s.fixing_brake = 0,
    }

    match av.traveller_light.as_str() {
        "true" => s.lights_high_beam = 1,
        _ => s.lights_high_beam = 0,
    }

    // wir prüfen nur ob gesetzt, nicht in welche Richtung (in api: -1,0,1 für links,aus,rechts)
    match av.indicator_state {
        0 => s.indicator = 0,  // aus
        -1 => s.indicator = 1, // an links
        1 => s.indicator = 2,  // an rechts
        _ => s.indicator = 0,
    }

    s.speed = av.speed.abs().round() as u32;
    s.maxspeed = av.allowed_speed.abs().round() as u32;

    s.fuel = (av.display_fuel * 100.0).trunc() as u32;

    s.lights_main = av.all_lamps.light_main.trunc() as u8;
    s.lights_front_door = av.all_lamps.front_door_light.trunc() as u8;
    s.lights_second_door = av.all_lamps.second_door_light.trunc() as u8;
    s.lights_stop_request = av.all_lamps.led_stop_request.trunc() as u8;
    s.lights_stop_brake = av.all_lamps.light_stopbrake.trunc() as u8;

    return s;
}

pub fn compare_vehicle_states(
    old: &VehicleState,
    new: &VehicleState,
    force: bool,
) -> Vec<u8> {
    let verbose = false;
    let mut buffer: Vec<u8> = vec![0; 0];

    if (old.ignition != new.ignition) || force {
        if verbose {
            println!("ignition: {} -> {} ", old.ignition, new.ignition);
        }
        let mut b = build_komsi_command_u8(KomsiCommandKind::Ignition, new.ignition);
        buffer.append(&mut b);
    }

    if (old.engine != new.engine) || force {
        if verbose {
            println!("engine: {} -> {} ", old.engine, new.engine);
        }
        let mut b = build_komsi_command_u8(KomsiCommandKind::Engine, new.engine);
        buffer.append(&mut b);
    }

    if (old.doors != new.doors) || force {
        if verbose {
            println!("doors: {} -> {} ", old.doors, new.doors);
        }
        let mut b = build_komsi_command_u8(KomsiCommandKind::PassengerDoorsOpen, new.doors);
        buffer.append(&mut b);
    }

    if (old.fixing_brake != new.fixing_brake) || force {
        if verbose {
            println!(
                "fixing_brake: {} -> {} ",
                old.fixing_brake, new.fixing_brake
            );
            let mut b =
                build_komsi_command_u8(KomsiCommandKind::FixingBrake, new.fixing_brake);
            buffer.append(&mut b);
        }
    }

    if (old.indicator != new.indicator) || force {
        if verbose {
            println!("indicator: {} -> {} ", old.indicator, new.indicator);
        }
        let mut b = build_komsi_command_u8(KomsiCommandKind::Indicator, new.indicator);
        buffer.append(&mut b);
    }

    if (old.lights_warning != new.lights_warning) || force {
        if verbose {
            println!(
                "lights_warning: {} -> {} ",
                old.lights_warning, new.lights_warning
            );
        }
        let mut b =
            build_komsi_command_u8(KomsiCommandKind::LightsWarning, new.lights_warning);
        buffer.append(&mut b);
    }

    if (old.lights_main != new.lights_main) | force {
        if verbose {
            println!("lights_main: {} -> {} ", old.lights_main, new.lights_main);
        }
        let mut b = build_komsi_command_u8(KomsiCommandKind::LightsMain, new.lights_main);
        buffer.append(&mut b);
    }

    if (old.lights_stop_request != new.lights_stop_request) || force {
        if verbose {
            println!(
                "lights_stop_request: {} -> {} ",
                old.lights_stop_request, new.lights_stop_request
            );
        }
        let mut b = build_komsi_command_u8(
            KomsiCommandKind::LightsStopRequest,
            new.lights_stop_request,
          
        );
        buffer.append(&mut b);
    }

    if (old.lights_stop_brake != new.lights_stop_brake) || force {
        if verbose {
            println!(
                "lights_stop_brake: {} -> {} ",
                old.lights_stop_brake, new.lights_stop_brake
            );
        }
        let mut b = build_komsi_command_u8(
            KomsiCommandKind::LightsStopBrake,
            new.lights_stop_brake,
          
        );
        buffer.append(&mut b);
    }

    if (old.lights_front_door != new.lights_front_door) || force {
        if verbose {
            println!(
                "lights_front_door: {} -> {} ",
                old.lights_front_door, new.lights_front_door
            );
        }
        let mut b = build_komsi_command_u8(
            KomsiCommandKind::LightsFrontDoor,
            new.lights_front_door,
         
        );
        buffer.append(&mut b);
    }

    if (old.lights_second_door != new.lights_second_door) || force {
        if verbose {
            println!(
                "lights_second_door: {} -> {} ",
                old.lights_second_door, new.lights_second_door
            );
        }
        let mut b = build_komsi_command_u8(
            KomsiCommandKind::LightsSecondDoor,
            new.lights_second_door,
         );
        buffer.append(&mut b);
    }

    if (old.lights_high_beam != new.lights_high_beam) || force {
        if verbose {
            println!(
                "lights_high_beam: {} -> {} ",
                old.lights_high_beam, new.lights_high_beam
            );
        }
        let mut b = build_komsi_command_u8(
            KomsiCommandKind::LightsHighBeam,
            new.lights_high_beam,
        );
        buffer.append(&mut b);
    }

    if (old.fuel != new.fuel) || force {
        if verbose {
            println!("fuel:  {} -> {} ", old.fuel, new.fuel);
        }
        let mut b = build_komsi_command(KomsiCommandKind::Fuel, new.fuel);
        buffer.append(&mut b);
    }

    if (old.speed != new.speed) || force {
        if verbose {
            println!("speed:  {} -> {} ", old.speed, new.speed);
        }
        let mut b = build_komsi_command(KomsiCommandKind::Speed, new.speed);
        buffer.append(&mut b);
    }

    if (old.maxspeed != new.maxspeed) || force {
        if verbose {
            println!("maxspeed:  {} -> {} ", old.maxspeed, new.maxspeed);
        }
        let mut b = build_komsi_command(KomsiCommandKind::MaxSpeed, new.maxspeed);
        buffer.append(&mut b);
    }

    // zeilenende hinzu, wenn buffer nicht leer
    if buffer.len() > 0 {
        let cmd = KomsiCommandKind::EOL as u8;
        let mut cb: Vec<u8> = vec![cmd];
        buffer.append(&mut cb);
    }

    return buffer;
}
