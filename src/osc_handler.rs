use anyhow::Result;
use log::{debug, warn};
use crate::piblaster::{GpioPin, PiBlaster};
use rosc::OscPacket;
use std::collections::HashMap;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct OscPath {
    path: String,
}

impl OscPath {
    pub fn new(path: String) -> Self {
        Self { path: path }
    }
}

pub struct OSCHandler {
    piblaster: PiBlaster,
    path_map: HashMap<OscPath, Vec<GpioPin>>,
}

impl OSCHandler {
    pub fn new(piblaster: PiBlaster, path_map: HashMap<OscPath, Vec<GpioPin>>) -> Self {
        Self {
            piblaster: piblaster,
            path_map: path_map,
        }
    }

    /// Sets all pins assigned to the path to the specified value.
    fn set_path(&mut self, path: &OscPath, value: f32) -> Result<()> {
        debug!("Setting {} to {}", path.path, value);
        let pins = &self.path_map[path];
        for pin in pins {
            self.piblaster.set_pin(&pin, value)?;
        }
        Ok(())
    }

    fn handle_packet_internal(&mut self, packet: OscPacket) -> Result<()> {
        match packet {
            OscPacket::Message(msg) => {
                if msg.args.len() != 1 {
                    warn!(
                        "Received message with {} arguments (should be 1)",
                        msg.args.len()
                    );
                }
                let val = match msg.args[0] {
                    rosc::OscType::Float(f) => Some(f),
                    rosc::OscType::Double(d) => Some(d as f32),
                    _ => {
                        // TODO the warning could be made more specific
                        warn!("Received wrong type, nly Float/Double supported");
                        None
                    }
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
