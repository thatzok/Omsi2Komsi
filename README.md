# Omsi2Komsi

Omsi2Komsi is a collection of tools for the "OMSI 2" bus simulator, written in Rust.

## Projects

### Omsi2Komsi (Main Plugin)

Omsi2Komsi is a plugin DLL that reads information (speed, lamps, etc.) from OMSI 2 and sends it to a serial port (USB) using the [KOMSI protocol](https://github.com/thatzok/Komsi-Protocol).

An Arduino/ESP32 or similar device connected to the USB port can then read these messages and display the data on a physical bus dashboard (e.g., speed on a speedometer, lamp lighting, etc.).

#### Usage

1. Copy both `omsi2komsi.dll` and `omsi2komsi.opl` into the `plugins` directory of OMSI 2.
2. Edit the `omsi2komsi.opl` file and change the `portname` to the one where your Arduino/ESP32 is connected.
3. Start OMSI 2.

The configuration is done via the `omsi2komsi.opl` file, which must be located in the same directory as the DLL.

```ini
[dll]
omsi2komsi.dll

[varlist]
14
elec_busbar_main
cockpit_light_batterie
Velocity
door_light_1
door_light_2
door_light_3
haltewunsch
AI_Light
lights_fern
cockpit_light_feststellbremse
AI_Blinker_L
AI_Blinker_R
tank_percent
bremse_halte

[omsi2komsi]
portname = com22
baudrate = 115200
engineonvalue = 1
```

### OmsiLogger

OmsiLogger is a diagnostic tool that displays real-time values of OMSI 2 variables in an overlay window and logs them to a file.

#### Usage

1. Copy `omsilogger.dll` (compiled from the example) and `omsilogger.opl` into the `plugins` directory.
2. By default, press **F11** to toggle the logger window visibility.
3. It will log the changed values defined in `omsilogger.opl` to a file named `omsilogger_YYYY-MM-DD.txt` in the OMSI 2 directory.

The configuration file `omsilogger.opl` allows you to define the variables to monitor and the hotkey:

```ini
[dll]
omsilogger.dll

[varlist]
2
Velocity
engine_n

[hotkey]
0x7A
```
*(0x7A is the virtual key code for F11)*

## Development

To compile the project for OMSI 2 (which requires 32-bit):

```bash
cargo build --release --target i686-pc-windows-msvc
cargo build --example omsilogger --release --target i686-pc-windows-msvc
```

Have fun!
