use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Opts {
    /// comport für Kommandos an das Armaturenbrett
    #[structopt(short, long)]
    pub port: Option<String>,

    /// Find port with name
    #[structopt(short, long)]
    pub find: Option<String>,

    /// IP des PC auf dem TheBus läuft
    #[structopt(short, long, default_value = "127.0.0.1")]
    pub ip: String,

    /// Baud rate
    #[structopt(short, long, default_value = "115200")]
    pub baud: u32,

    /// Zeit in Millisekunden zwischen API aufrufen
    #[structopt(short, long, default_value = "200")]
    pub sleeptime: u64,

    /// debugging einschalten
    #[structopt(short, long)]
    pub debug: bool,

    /// debugging serial einschalten
    #[structopt(long)]
    pub debug_serial: bool,

    /// debugging command einschalten
    #[structopt(long)]
    pub debug_command: bool,

    /// nichts auf seriellen port ausgeben/einlesen
    #[structopt(long)]
    pub disable_serial: bool,

    /// Alle verfügbaren comports anzeigen
    #[structopt(short, long)]
    pub list: bool,

    /// Ausführliche Ausgabe einschalten
    #[structopt(short, long)]
    pub verbose: bool,

    /// Alle Werte zurücksetzen
    #[structopt(short, long)]
    pub clear: bool,
}
