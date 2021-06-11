use yaml_rust::YamlLoader;
use log::info;
use std::fs;
use anyhow::Result;

pub fn load_config() -> Result<()> {
    let config_raw = fs::read_to_string("conf.yaml")?;
    let docs = YamlLoader::load_from_str(&config_raw)?;
    let doc = &docs[0];
    info!("{:?}", doc);
    Ok(())
}