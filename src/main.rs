use clap::{AppSettings, Clap};
use log::{debug, info, trace, warn};
use maplit::hashmap;
use rosc::OscPacket;
use simplelog as sl;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use anyhow::{Result, bail};

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct GpioPin {
    index: usize,
}

impl GpioPin {
    pub fn new(index: usize) -> Self {
        Self { index: index }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct OscPath {
    path: String,
}

impl OscPath {
    pub fn new(path: String) -> Self {
        Self { path: path }
    }
}

struct PiBlaster {
    current_values: HashMap<GpioPin, f32>,
    outfile: File,
}

impl PiBlaster {
    pub fn new(path: &String, pins: &Vec<GpioPin>) -> Self {
        let mut cvs = HashMap::new();
        for pin in pins {
            cvs.insert(*pin, 0.0);
        }
        Self {
            current_values: cvs,
            outfile: OpenOptions::new().write(true).open(path).unwrap(),
        }
    }

    pub fn set_pin(&mut self, pin: &GpioPin, value: f32) -> Result<()> {
        if !self.current_values.contains_key(pin) {
            bail!("Trying to set a key that hasn't been configured");
        }
        let current_value = *self.current_values.get(&pin).unwrap();
        if value == current_value {
            return Ok(());
        }
        self.current_values.insert(*pin, value);
        let s = format!("{}={}\n", pin.index, value);
        let s = s.as_bytes();
        trace!("Writing line to file");
        self.outfile.write_all(s)?;
        Ok(())
    }

    pub fn ensure_write_out(&mut self) -> Result<()> {
        trace!("Syncing data to file");
        self.outfile.sync_data()?;
        Ok(())
    }
}

struct OSCHandler {
    path_map: HashMap<OscPath, GpioPin>,
    piblaster: PiBlaster,
}

impl OSCHandler {
    pub fn new(path_map: HashMap<OscPath, GpioPin>, piblaster: PiBlaster) -> Self {
        Self {
            path_map: path_map,
            piblaster: piblaster,
        }
    }

    fn set_path(&mut self, path: &OscPath, value: f32) -> Result<()> {
        debug!("Setting {} to {}", path.path, value);
        let pin = self.path_map[path];
        self.piblaster.set_pin(&pin, value)?;
        Ok(())
    }

    fn handle_packet_internal(&mut self, packet: OscPacket) -> Result<()> {
        match packet {
            OscPacket::Message(msg) => {
                if msg.args.len() != 1 {
                    warn!("Received message with {} arguments (should be 1)", msg.args.len());
                }
                let val = match msg.args[0] {
                    rosc::OscType::Float(f) => Some(f),
                    rosc::OscType::Double(d) => Some(d as f32),
                    _ => {  // TODO the warning could be made more specific
                        warn!("Received wrong type, nly Float/Double supported");
                        None
                    },
                };
                if let Some(v) = val {
                    let path = OscPath::new(msg.addr);
                    if self.path_map.contains_key(&path) {
                        self.set_path(&path, v)?;
                    }
                }
            }
            OscPacket::Bundle(bundle) => {
                for packet in bundle.content {
                    self.handle_packet_internal(packet)?;
                }
            }
        };
        Ok(())
    }

    pub fn handle_packet(&mut self, packet: OscPacket) -> Result<()> {
        self.handle_packet_internal(packet)?;
        self.piblaster.ensure_write_out()?;
        Ok(())
    }
}

fn receive_osc_packets(addr: SocketAddrV4, mut osc_handler: OSCHandler) -> Result<()> {
    let sock = UdpSocket::bind(addr).unwrap();

    let mut buf = [0u8; rosc::decoder::MTU];

    loop {
        let (size, addr) = sock.recv_from(&mut buf)?;
        trace!("Received {} bytes from {}", size, addr);
        let packet = rosc::decoder::decode(&buf[..size]).unwrap();
        osc_handler.handle_packet(packet)?;
    }
}

// TODO
// Create a config file that defines which pin is mapped to which path
// 0: /topLeft/red
// 3: /topLeft/blue
// 4: /topLeft/green
// etc...

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// How much information to print. Can be provided up to three times
    #[clap(short, long, parse(from_occurrences))]
    verbose: isize,
    /// Print no output at all
    #[clap(short, long)]
    quiet: bool,
    /// The port to listen on
    #[clap(short, long)]
    port: u16,
}

fn level_filter_from_level_index(level_index: isize) -> sl::LevelFilter {
    match level_index {
        -1 => sl::LevelFilter::Off,
        0 => sl::LevelFilter::Warn,
        1 => sl::LevelFilter::Info,
        2 => sl::LevelFilter::Debug,
        _ => sl::LevelFilter::Trace,
    }
}

fn init_logger(verbosity: isize) {
    sl::TermLogger::init(
        level_filter_from_level_index(verbosity),
        sl::Config::default(),
        sl::TerminalMode::Stderr,
        sl::ColorChoice::Auto,
    ).expect("Could not create logger");
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let verbosity = if opts.quiet { -1 } else { opts.verbose };
    init_logger(verbosity);
    let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), opts.port);
    info!("Listening on address {}", addr);
    let piblaster = PiBlaster::new(&"./piblaster.out".to_string(), &vec![GpioPin::new(0)]);
    let osc_handler = OSCHandler::new(
        hashmap! {
            OscPath::new("/1/fader1".to_string()) => GpioPin::new(0)
        },
        piblaster,
    );
    receive_osc_packets(addr, osc_handler)?;
    Ok(())
}
