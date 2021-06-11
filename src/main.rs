use rosc::OscPacket;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;
use maplit::hashmap;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct GpioPin {
    index: usize,
}

impl GpioPin {
    pub fn new(index: usize) -> Self {
        Self {
            index: index
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct OscPath {
    path: String,
}

impl OscPath {
    pub fn new(path: String) -> Self {
        Self {
            path: path
        }
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

    pub fn set_pin(&mut self, pin: &GpioPin, value: f32) {
        if !self.current_values.contains_key(pin) {
            println!("Missing key!");
            return;
        }
        let current_value = *self.current_values.get(&pin).unwrap();
        if value == current_value {
            return;
        }
        self.current_values.insert(*pin, value);
        let s = format!("{}={}\n", pin.index, value);
        let s = s.as_bytes();
        self.outfile.write_all(s);
    }

    pub fn ensure_write_out(&mut self) {
        self.outfile.sync_data();
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

    fn set_path(&mut self, path: &OscPath, value: f32) {
        let pin = self.path_map[path];
        self.piblaster.set_pin(&pin, value);
    }

    fn handle_packet_internal(&mut self, packet: OscPacket) {
        match packet {
            OscPacket::Message(msg) => {
                if msg.args.len() != 1 {
                    return;
                }
                let val = match msg.args[0] {
                    rosc::OscType::Float(f) => Some(f),
                    rosc::OscType::Double(d) => Some(d as f32),
                    _ => None
                };
                if let Some(v) = val {
                    let path = OscPath::new(msg.addr);
                    if self.path_map.contains_key(&path) {
                        self.set_path(&path, v);
                    }
                }
            }
            OscPacket::Bundle(bundle) => {
                for packet in bundle.content {
                    self.handle_packet_internal(packet);
                }
            }
        }
    }

    pub fn handle_packet(&mut self, packet: OscPacket) {
        self.handle_packet_internal(packet);
        self.piblaster.ensure_write_out();
    }
}

fn receive_osc_packets(addr: SocketAddrV4, mut osc_handler: OSCHandler) {
    let sock = UdpSocket::bind(addr).unwrap();

    let mut buf = [0u8; rosc::decoder::MTU];

    loop {
        match sock.recv_from(&mut buf) {
            Ok((size, addr)) => {
                let packet = rosc::decoder::decode(&buf[..size]).unwrap();
                osc_handler.handle_packet(packet);
            }
            Err(e) => {
                break;
            }
        }
    }
}

// TODO
// Create a config file that defines which pin is mapped to which path
// 0: /topLeft/red
// 3: /topLeft/blue
// 4: /topLeft/green
// etc...

fn main() {
    println!("Hello, world!");
    let addr = match SocketAddrV4::from_str("0.0.0.0:4243") {
        Ok(addr) => addr,
        Err(_) => panic!("lala"),
    };
    let piblaster = PiBlaster::new(&"./piblaster.out".to_string(), &vec![GpioPin::new(0)]);
    let osc_handler = OSCHandler::new(hashmap!{
        OscPath::new("/1/fader1".to_string()) => GpioPin::new(0)
    }, piblaster);
    receive_osc_packets(addr, osc_handler);
}
