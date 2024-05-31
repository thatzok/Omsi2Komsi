use reqwest::blocking::Client;
use serde::Deserialize;
use std::time::Duration;
use crate::opts::Opts;

#[derive(Deserialize, Debug)]
pub struct ApiVehicleType {
    #[serde(rename = "ActorName")]
    pub actor_name: String,
    #[serde(rename = "IgnitionEnabled")]
    pub ignition_enabled: String,
    #[serde(rename = "EngineStarted")]
    pub engine_started: String,
    #[serde(rename = "WarningLights")]
    pub warning_lights: String,
    #[serde(rename = "PassengerDoorsOpen")]
    pub passenger_doors_open: String,
    #[serde(rename = "TravellerLight")]
    pub traveller_light: String,
    #[serde(rename = "FixingBrake")]
    pub fixing_brake: String,
    #[serde(rename = "Speed")]
    pub speed: f32,
    #[serde(rename = "AllowedSpeed")]
    pub allowed_speed: f32,
    #[serde(rename = "DisplayFuel")]
    pub display_fuel: f32,
    #[serde(rename = "IndicatorState")]
    pub indicator_state: i8,
    #[serde(rename = "AllLamps")]
    pub all_lamps: ApiLamps,
}

#[derive(Deserialize, Debug)]
pub struct ApiLamps {
    #[serde(rename = "LightMain")]
    pub light_main: f32,
    #[serde(rename = "Front Door Light")]
    pub front_door_light: f32,
    #[serde(rename = "Second Door Light")]
    pub second_door_light: f32,
    #[serde(rename = "LED StopRequest")]
    pub led_stop_request: f32,
    #[serde(rename = "ButtonLight BusStopBrake")]
    pub light_stopbrake: f32,
}

pub fn getapidata(opts: &Opts) -> Result<ApiVehicleType, Box<dyn std::error::Error>> {
    let request_url = format!("http://{}:37337/Vehicles/Current", opts.ip);

    let timeout = Duration::new(2, 0);
    let client = Client::new();

    if opts.debug {
        eprintln!("Fetching url {} ...", &request_url);
    }

    let response = client.get(&request_url).timeout(timeout).send()?; // wir warten auf die antwort
                                                                      // eprintln!("http get erfolgt");

    if !response.status().is_success() {
        Err("Error: response code")?
    }

    // eprintln!("http code OK");
    // eprintln!("Response: {:?} {}", response.version(), response.status());
    // eprintln!("Headers: {:#?}\n", response.headers());

    let api_vehicle: ApiVehicleType = response.json()?;
    if opts.debug {
        println!("{:?}", &api_vehicle);
    }
    Ok(api_vehicle)
}
