pub mod gate;

pub use gate::{Gate, GateKind};

#[derive(Debug, Clone)]
pub enum Device {
	Gate(Gate),
}
