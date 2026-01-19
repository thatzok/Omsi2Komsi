use crate::komsi::build_komsi_command;
use crate::komsi::build_komsi_command_eol;
use crate::komsi::build_komsi_command_u8;
use crate::komsi::KomsiCommandKind;

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
    pub door_enable: u8,
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
            door_enable: 0,
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
        print!("doorenable:{} ", self.door_enable);
        println!(" ");
    }

    pub fn compare(
        &self,
        new: &VehicleState,
        force: bool,
        logger: Option<&dyn VehicleLogger>,
    ) -> Vec<u8> {
        let mut buffer: Vec<u8> = vec![0; 0];

        self.handle_u8_field_change(
            self.ignition,
            new.ignition,
            "ignition",
            KomsiCommandKind::Ignition,
            logger,
            force,
            &mut buffer,
        );

        self.handle_u8_field_change(
            self.engine,
            new.engine,
            "engine",
            KomsiCommandKind::Engine,
            logger,
            force,
            &mut buffer,
        );

        self.handle_u8_field_change(
            self.doors,
            new.doors,
            "doors",
            KomsiCommandKind::PassengerDoorsOpen,
            logger,
            force,
            &mut buffer,
        );

        self.handle_u8_field_change(
            self.fixing_brake,
            new.fixing_brake,
            "fixing_brake",
            KomsiCommandKind::FixingBrake,
            logger,
            force,
            &mut buffer,
        );

        self.handle_u8_field_change(
            self.indicator,
            new.indicator,
            "indicator",
            KomsiCommandKind::Indicator,
            logger,
            force,
            &mut buffer,
        );

        self.handle_u8_field_change(
            self.lights_warning,
            new.lights_warning,
            "lights_warning",
            KomsiCommandKind::LightsWarning,
            logger,
            force,
            &mut buffer,
        );

        self.handle_u8_field_change(
            self.lights_main,
            new.lights_main,
            "lights_main",
            KomsiCommandKind::LightsMain,
            logger,
            force,
            &mut buffer,
        );

        self.handle_u8_field_change(
            self.lights_stop_request,
            new.lights_stop_request,
            "lights_stop_request",
            KomsiCommandKind::LightsStopRequest,
            logger,
            force,
            &mut buffer,
        );

        self.handle_u8_field_change(
            self.lights_stop_brake,
            new.lights_stop_brake,
            "lights_stop_brake",
            KomsiCommandKind::LightsStopBrake,
            logger,
            force,
            &mut buffer,
        );

        self.handle_u8_field_change(
            self.lights_front_door,
            new.lights_front_door,
            "lights_front_door",
            KomsiCommandKind::LightsFrontDoor,
            logger,
            force,
            &mut buffer,
        );

        self.handle_u8_field_change(
            self.lights_second_door,
            new.lights_second_door,
            "lights_second_door",
            KomsiCommandKind::LightsSecondDoor,
            logger,
            force,
            &mut buffer,
        );

        self.handle_u8_field_change(
            self.lights_third_door,
            new.lights_third_door,
            "lights_third_door",
            KomsiCommandKind::LightsThirdDoor,
            logger,
            force,
            &mut buffer,
        );

        self.handle_u8_field_change(
            self.lights_high_beam,
            new.lights_high_beam,
            "lights_high_beam",
            KomsiCommandKind::LightsHighBeam,
            logger,
            force,
            &mut buffer,
        );

        self.handle_u32_field_change(
            self.fuel,
            new.fuel,
            "fuel",
            KomsiCommandKind::Fuel,
            logger,
            force,
            &mut buffer,
        );

        self.handle_u32_field_change(
            self.speed,
            new.speed,
            "speed",
            KomsiCommandKind::Speed,
            logger,
            force,
            &mut buffer,
        );

        self.handle_u32_field_change(
            self.maxspeed,
            new.maxspeed,
            "maxspeed",
            KomsiCommandKind::MaxSpeed,
            logger,
            force,
            &mut buffer,
        );

        self.handle_u8_field_change(
            self.battery_light,
            new.battery_light,
            "battery_light",
            KomsiCommandKind::BatteryLight,
            logger,
            force,
            &mut buffer,
        );

        self.handle_u8_field_change(
            self.door_enable,
            new.door_enable,
            "door_enable",
            KomsiCommandKind::DoorEnable,
            logger,
            force,
            &mut buffer,
        );

        // zeilenende hinzu, wenn buffer nicht leer
        if buffer.len() > 0 {
            let mut b = build_komsi_command_eol();
            buffer.append(&mut b);
        }

        buffer
    }

    // Helper function for handling u8 field changes
    fn handle_u8_field_change(
        &self,
        old_value: u8,
        new_value: u8,
        field_name: &str,
        command_kind: KomsiCommandKind,
        logger: Option<&dyn VehicleLogger>,
        force: bool,
        buffer: &mut Vec<u8>,
    ) {
        if (old_value != new_value) || force {
            if let Some(l) = logger {
                l.log(format!("{}: {} -> {} ", field_name, old_value, new_value));
            }
            let mut b = build_komsi_command_u8(command_kind, new_value);
            buffer.append(&mut b);
        }
    }

    // Helper function for handling u32 field changes
    fn handle_u32_field_change(
        &self,
        old_value: u32,
        new_value: u32,
        field_name: &str,
        command_kind: KomsiCommandKind,
        logger: Option<&dyn VehicleLogger>,
        force: bool,
        buffer: &mut Vec<u8>,
    ) {
        if (old_value != new_value) || force {
            if let Some(l) = logger {
                l.log(format!("{}:  {} -> {} ", field_name, old_value, new_value));
            }
            let mut b = build_komsi_command(command_kind, new_value);
            buffer.append(&mut b);
        }
    }
}

