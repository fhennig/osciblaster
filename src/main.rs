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

fn main() {
    println!("Hello, world!");
}
