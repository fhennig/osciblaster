use crate::osc_handler::OscPath;
use crate::piblaster::GpioPin;
use anyhow::{bail, Result};
use log::info;
use std::fs;
use yaml_rust::{Yaml, YamlLoader};

pub fn load_config() -> Result<()> {
    let config_raw = fs::read_to_string("conf.yaml")?;
    let docs = YamlLoader::load_from_str(&config_raw)?;
    let doc = &docs[0];
    info!("{:?}", doc);
    Ok(())
}

pub struct Config {
    yaml: Yaml,
}

impl Config {
    pub fn new() -> Result<Self> {
        let config_raw = fs::read_to_string("conf.yaml")?;
        let mut docs = YamlLoader::load_from_str(&config_raw)?;
        if docs.len() != 0 {
            bail!("Only a single doc is supported.");
        }
        let doc = docs.remove(0);
        info!("{:?}", doc);
        Ok(Self { yaml: doc })
    }
}
