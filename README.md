# Omsi2Komsi

[English version](README.en.md)

Omsi2Komsi ist eine Sammlung von Werkzeugen für den Bussimulator "OMSI 2", geschrieben in Rust.

## Omsi2Komsi (Haupt-Plugin)

Omsi2Komsi ist eine Plugin-DLL, die Informationen (Geschwindigkeit, Lampen usw.) aus OMSI 2 liest und über eine serielle
Schnittstelle (USB) mit dem KOMSI-Protokoll sendet.
Infos zum Komso-Protokoll gibt es hier: https://github.com/thatzok/Komsi-Protocol
Ein am USB-Anschluss angeschlossener Arduino/ESP32 oder ein ähnliches Gerät kann diese Nachrichten dann lesen und die
Daten auf einem physischen Bus-Dashboard anzeigen (z. B. Geschwindigkeit auf einem Tachometer, Lampenbeleuchtung usw.).

omsi_2_komsi_v2.4.0_x86.zip

## Download & Installation

1. Gehe zur Seite des [neuesten Releases](https://github.com/YourUsername/TheBusCmd/releases/latest).
2. Lade die Datei `omsi_2_komsi_vx.x.x_x86.zip` herunter.
3. Entpacke den Inhalt der ZIP-Datei in einen Ordner deiner Wahl.
4. Du findest darin sowohl `omsi2komsi.dll` als auch `omsilogger.dll` mit ihren .opl-Dateien.
5. Kopiere sowohl `omsi2komsi.dll` als auch `omsi2komsi.opl` in das Verzeichnis "`plugins`" von OMSI 2.
6. Bearbeite die Datei `omsi2komsi.opl` und ändern den `portname` auf den Anschluss, an dem der Arduino/ESP32
   angeschlossen ist.
7. Starte OMSI 2.
8. Standardmäßig drücke **F10**, um die Sichtbarkeit des Logger-Fensters umzuschalten und Diagnosemeldungen anzuzeigen.

Die Konfiguration erfolgt über die Datei `omsi2komsi.opl`, die sich auch im Pluginverzeichnis "`plugins`" von OMSI 2
befinden muss.
Aus der mitgelieferten Beispiel-Konfiguration sollten die Konfigurationsmöglichkeiten ersichtlich sein.
Zum Debugging und Fehler suchen kann man das Programm auch ohne serielle Schnittstelle (serialportenabled = false)
starten.

## OmsiLogger (Diagnosewerkzeug)

OmsiLogger ist ein Diagnosewerkzeug, das Echtzeitwerte von OMSI 2-Variablen in einem Overlay-Fenster anzeigt und in eine
Datei protokolliert.
Es kann dazu dienen herauszufinden, welche OMSI 2 Variablen überhaupt bei bestimmten Bussen eine Funktion haben um diese
dann später in der omsi2komsi.opl-Konfiguration zu verwenden.
Es können bis zu 100 Variablen in die [varlist] eingetragen werden. Werden mehr eingetragen, werden diese ignoriert.

### Verwendung

1. Kopiere `omsilogger.dll` und `omsilogger.opl` in das Verzeichnis "plugins" von OMSI 2.
2. Bearbeite die Datei omsilogger.opl und trage da die OMSI 2 Variablen ein, die geprüft werden sollen.
3. Starte OMSI 2.
4. Standardmäßig drücke **F10**, um die Sichtbarkeit des Logger-Fensters umzuschalten und Diagnosemeldungen anzuzeigen.
5. Wenn sich Variablenwerte ändern werden diese auch in eine Datei namens omsilogger_JJJJ-MM-TT.txt im OMSI
   2-Verzeichnis protokolliert.

Die Konfiguration erfolgt über die Datei omsilogger.opl, die sich auch im Pluginverzeichnis "plugins" von OMSI 2
befinden muss.
Aus der mitgelieferten Beispiel-Konfiguration sollten die Konfigurationsmöglichkeiten ersichtlich sein.

Omsi2Komsi und OmsiLogger sollten nicht gleichzeitig im Pluginverzeichnis "plugins" von OMSI 2 installiert sein.

**Viel Spaß!**

## Lizenz

Dieses Programm ist freie Software: Sie können es unter den Bedingungen der GNU General Public License, wie von der Free
Software Foundation veröffentlicht, entweder gemäß Version 3 der Lizenz oder (nach Ihrer Option) jeder späteren Version
weitergeben und/oder modifizieren.
Dieses Programm wird in der Hoffnung verteilt, dass es nützlich sein wird, aber OHNE JEDE GEWÄHRLEISTUNG; sogar ohne die
implizite Gewährleistung der MARKTFÄHIGKEIT oder EIGNUNG FÜR EINEN BESTIMMTEN ZWECK. Siehe die GNU General Public
License für weitere Details.
Sie sollten eine Kopie der GNU General Public License zusammen mit diesem Programm erhalten haben. Wenn nicht,
siehe <https://www.gnu.org/licenses/>.
