# Omsi2Komsi

Omsi2Komsi is a plugin DLL for the "OMSI 2" bus simulator .<br>

Omsi2Komsi reads information (for speed, lamps, etc.) from OMSI 2 and sends them to the serial port (USB) using the [KOMSI protocol](https://github.com/thatzok/Komsi-Protocol)

An Arduino/ESP32 or similar connected to the USB port can then read these messages and display the data on a bus dashboard (e.g. speed on a speedometer, lamp lighting, etc.).

## Usage

* Copy both files omsi2komsi.dll and omsi2komsi.opl into the plugin directory of OMSI 2
* Edit the file omsi2komsi.opl and change the portname to the one where your Arduino/ESP32 is connected to
* Start OMSI 2

The configuration is done via the file omsi2komsi.opl, which must be located in the same OMSI 2 plugin directory where omsi2komsi.dll is located.

```

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


Have fun!
