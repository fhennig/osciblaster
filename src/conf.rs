use crate::osc_handler::OscPath;
use crate::piblaster::GpioPin;
use anyhow::{bail, Result};
use log::{info, debug};
use std::fs;
use yaml_rust::{Yaml, YamlLoader};
use std::collections::HashMap;

pub struct Config {
    pub path_pin_map: HashMap<OscPath, Vec<GpioPin>>,
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
        Ok(Self {
            path_pin_map: pin_map,
        })
    }
}
