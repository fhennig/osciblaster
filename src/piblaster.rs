use anyhow::{bail, Result};
use log::trace;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::Write;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct GpioPin {
    index: usize,
}

impl GpioPin {
    pub fn new(index: usize) -> Self {
        Self { index: index }
    }
}

pub struct PiBlaster {
    current_values: HashMap<GpioPin, f32>,
    outfile: File,
}

impl PiBlaster {
    pub fn new(path: &String, pins: &Vec<GpioPin>) -> Result<Self> {
        let mut cvs = HashMap::new();
        for pin in pins {
            cvs.insert(*pin, 0.0);
        }
        Ok(Self {
            current_values: cvs,
            outfile: OpenOptions::new().write(true).open(path)?,
        })
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
        self.outfile.write_all(s)?; // for a FIFO, synchronization is not necessary
        Ok(())
    }
}
