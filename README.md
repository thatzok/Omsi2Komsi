# TheBus2Komsi

TheBus2Komsi is an API-Client for the TheBus Bus Simulator.<br>

TheBus2Komsi reads information (for speed, lamps, etc.) from the TheBus telemetry-API and sends them to the serial port (USB) using the KOMSI protocol.

An Arduino/ESP32 or similar connected to the USB port can then read these messages and display the data on a bus dashboard (e.g. speed on a speedometer, lamp lighting, etc.).

## Usage

The configuration is done via the file TheBus2Komsi.ini, which must be located in the same directory as TheBus2Komsi.exe.

```
# TheBus2Komsi.ini
# This file must be in the same directory as TheBus2Komsi.exe
#
# Normally you only need to change the portname to the one your are using
# 
# If you don't know which comport your Arduino/ESP32 is connected to, you can start the program with
# TheBus2Komsi -l

[default]
portname = com22
baudrate = 115200
sleeptime = 200
ip = 127.0.0.1
```


To get a list of all command line parameters, start the program with the "--help" option.

  ```sh
  TheBus2Komsi --help
  ```

Have fun!
