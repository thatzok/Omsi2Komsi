pub enum KomsiCommandKind {
    EOL = 10,                // used, end of command line      means EOL which is "\n"
    Ignition = 65,           // A1  A
    Engine = 66,             // A2  B
    PassengerDoorsOpen = 67, // A3  C
    Indicator = 68,          // A4  D
    FixingBrake = 69,        // A5  E
    LightsWarning = 70,      // A6  F
    LightsMain = 71,         // A7  G
    LightsFrontDoor = 72,    // A8  H
    LightsSecondDoor = 73,   // A9  I
    LightsThirdDoor = 74,    // A10  J
    LightsStopRequest = 75,  // A11 K
    LightsStopBrake = 76,    // A12 L
    LightsHighBeam = 77,     // A13 M
    BatteryLight = 78,       // N
    SimulatorType = 79,      // O
    DoorEnable = 80,         // P
    A17 = 81,                // Q
    A18 = 82,                // R
    A19 = 83,                // S
    A20 = 84,                // T
    A21 = 85,                // U
    A22 = 86,                // V
    A23 = 87,                // W
    A24 = 88,                // X
    A25 = 89,                // Y
    A26 = 90,                // Z

    MaxSpeed = 115,    // s
    RPM = 116,         // t
    Pressure = 117,    // u
    Temperature = 118, // v
    Oil = 119,         // w
    Fuel = 120,        // x
    Speed = 121,       // y
    Water = 122,       // z
}

pub fn build_komsi_command(cmd: KomsiCommandKind, wert: u32) -> Vec<u8> {
    let cmd_u8 = cmd as u8;
    let mut buffer: Vec<u8> = vec![cmd_u8];
    let mut s: Vec<u8> = wert.to_string().as_bytes().to_vec();

    buffer.append(&mut s);

    buffer
}

pub fn build_komsi_command_u8(cmd: KomsiCommandKind, wert: u8) -> Vec<u8> {
    let cmd_u8 = cmd as u8;
    let mut buffer: Vec<u8> = vec![cmd_u8];
    let mut s: Vec<u8> = wert.to_string().as_bytes().to_vec();

    buffer.append(&mut s);

    buffer
}

pub fn build_komsi_command_eol() -> Vec<u8> {
    let cmd_u8 = KomsiCommandKind::EOL as u8;
    let buffer: Vec<u8> = vec![cmd_u8];
    buffer
}
