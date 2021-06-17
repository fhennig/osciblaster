use crate::osc_handler::OscPath;
use crate::piblaster::GpioPin;
use anyhow::{bail, Result};
use log::debug;
use std::collections::HashMap;
use std::fs;
use yaml_rust::{Yaml, YamlLoader};
use std::convert::TryFrom;

pub struct Config {
    path_pin_map: HashMap<OscPath, Vec<GpioPin>>,
    pibaster_dev_file: String,
    port: u16,
}

fn as_osc_path(yaml: &Yaml) -> Result<OscPath> {
    if let Some(s) = yaml.as_str() {
        Ok(OscPath::new(s.to_string()))
    } else {
        bail!("Could not convert value into a string.");
    }
}

fn as_gpio_pin_vector(yaml: &Yaml) -> Result<Vec<GpioPin>> {
    let mut v = vec![];
    if let Some(pin) = yaml.as_i64() {
        v.push(GpioPin::new(pin as usize));
    } else if let Some(vec) = yaml.as_vec() {
        for y in vec {
            if let Some(i) = y.as_i64() {
                v.push(GpioPin::new(i as usize));
            } else {
                bail!("Vector contains non-integer value.");
            }
        }
    }
    if v.len() == 0 {
        bail!("Could not parse integer or vector of integers.");
    }
    Ok(v)
}

impl Config {
    pub fn new() -> Result<Self> {
        let config_raw = fs::read_to_string("conf.yaml")?;
        // get the doc
        let mut docs = YamlLoader::load_from_str(&config_raw)?;
        debug!("Raw config: {:?}", docs);
        if docs.len() != 1 {
            bail!("Only a single doc is supported.");
        }
        let doc = docs.remove(0);
        // get the map
        let mut pin_map = HashMap::new();
        let map = doc["osc_pin_map"].as_hash();
        if let Some(m) = map {
            for (key, value) in m {
                let osc_path = as_osc_path(&key)?;
                let pin_vec = as_gpio_pin_vector(&value)?;
                pin_map.insert(osc_path, pin_vec);
            }
        } else {
            bail!("The osc_pin_map needs to be a dictionary.");
        }
        // get the piblaster path
        let path = if let Some(s) = doc["piblaster"].as_str() {
            s.to_string()
        } else {
            bail!("...")
        };
        // get port
        let port = if let Some(i) = doc["port"].as_i64() {
            u16::try_from(i)?
        } else {
            bail!("...")
        };
        Ok(Self {
            path_pin_map: pin_map,
            pibaster_dev_file: path,
            port: port,
        })
    }

    pub fn get_path_pin_map(&self) -> &HashMap<OscPath, Vec<GpioPin>> {
        &self.path_pin_map
    }

    pub fn get_all_used_pins(&self) -> Vec<GpioPin> {
        let mut v = vec![];
        for (_, pins) in &self.path_pin_map {
            for pin in pins {
                v.push(*pin);
            }
        }
        v
    }

    pub fn get_piblaster_dev_file(&self) -> &String {
        &self.pibaster_dev_file
    }

    pub fn get_port(&self) -> u16 {
        self.port
    }
}
