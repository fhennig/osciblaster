use rosc::OscPacket;
use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::Write;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct GpioPin {
    index: usize,
}

struct OscPath {
    path: String,
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

    fn set_path(path: OscPath, value: f32) {}

    pub fn handle_packet(packet: OscPacket) {
        match packet {
            OscPacket::Message(msg) => {
                println!("OSC address: {}", msg.addr);
                println!("OSC arguments: {:?}", msg.args);
            }
            OscPacket::Bundle(bundle) => {
                println!("OSC Bundle: {:?}", bundle);
            }
        }
    }
}

fn main() {
    println!("Hello, world!");
}
