# Omsi2Komsi

[Deutsche Vrsion](README.md)

Omsi2Komsi is a collection of tools for the "OMSI 2" bus simulator, written in Rust.

## Projects

### Omsi2Komsi (Main Plugin)

Omsi2Komsi is a plugin DLL that reads information (speed, lamps, etc.) from OMSI 2 and sends it to a serial port (USB) using the [KOMSI protocol](https://github.com/thatzok/Komsi-Protocol).

An Arduino/ESP32 or similar device connected to the USB port can then read these messages and display the data on a physical bus dashboard (e.g., speed on a speedometer, lamp lighting, etc.).

#### Download & Installation
1. Go to the latest releases page [latest Releases](https://github.com/thatzok/Omsi2Komsi/releases).
2. Download the file `omsi_2_komsi_vx.x.x_x86.zip`.
3. Extract the contents of the ZIP file into a folder of your choice.
4. You will find both `omsi2komsi.dll` and `omsilogger.dll` along with their `.opl` files.
5. Copy both `omsi2komsi.dll` and `omsi2komsi.opl` into the "`plugins`" directory of OMSI 2.
6. Edit the `omsi2komsi.opl` file and change the portname to the port where your Arduino/ESP32 is connected.
7. Start OMSI 2.

By default, press **F10** to toggle the visibility of the logger window and display diagnostic messages.

The configuration is done via the `omsi2komsi.opl` file, which must be located in the same directory as the DLL.


### OmsiLogger

OmsiLogger is a diagnostic tool that displays real-time values of OMSI 2 variables in an overlay window and logs them to a file.

#### Usage

1. Copy `omsilogger.dll` and `omsilogger.opl` into the `plugins` directory.
2. Start OMSI 2.
3. By default, press **F10** to toggle the logger window visibility.
4. It will log the changed values defined in `omsilogger.opl` to a file named `omsilogger_YYYY-MM-DD.txt` in the OMSI 2 directory.

The configuration file `omsilogger.opl` allows you to define the variables to monitor and the hotkey:



**Have fun!**



## License

This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the [GNU General Public License](LICENSE) for more details.

You should have received a copy of the GNU General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.

