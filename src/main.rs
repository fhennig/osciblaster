mod conf;
mod osc_handler;
mod piblaster;
use anyhow::Result;
use clap::{AppSettings, Clap};
use conf::Config;
use log::{info, trace};
use osc_handler::OSCHandler;
use piblaster::{GpioPin, PiBlaster};
use simplelog as sl;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

fn receive_osc_packets(
    addr: SocketAddrV4,
    mut osc_handler: OSCHandler,
    running: Arc<AtomicBool>,
) -> Result<()> {
    let sock = UdpSocket::bind(addr)?;
    // Set a timeout as to not block indefinitely to allow for ctrlc handling
    sock.set_read_timeout(Some(Duration::new(1, 0)))?;

    let mut buf = [0u8; rosc::decoder::MTU];

    while running.load(Ordering::SeqCst) {
        match sock.recv_from(&mut buf) {
            // default case, handle packet
            Ok((size, addr)) => {
                trace!("Received {} bytes from {}", size, addr);
                let packet = rosc::decoder::decode(&buf[..size])?;
                osc_handler.handle_packet(packet)?;
            }
            // Error: either timeout or something went wrong
            Err(e) => {
                match e.kind() {
                    std::io::ErrorKind::WouldBlock => continue,
                    std::io::ErrorKind::Interrupted => continue, // interrupts are handled by ctrlc
                    _ => return Err(e.into()),
                }
            }
        }
    }
    Ok(())
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
    )
    .expect("Could not create logger");
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let verbosity = if opts.quiet { -1 } else { opts.verbose };
    init_logger(verbosity);
    let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), opts.port);
    info!("Listening on address {}", addr);
    let conf = Config::new()?;
    let piblaster = PiBlaster::new(&"./piblaster.out".to_string(), &vec![GpioPin::new(0)])?;
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;
    let osc_handler = OSCHandler::new(piblaster, conf.path_pin_map);
    receive_osc_packets(addr, osc_handler, running)?;
    Ok(())
}
