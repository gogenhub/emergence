use crate::_utils::{assigner, error, genetic_circuit};
use assigner::GeneNetwork;
use error::Error;
use genetic_circuit::{Component, GeneticCircuit, Signal};
use serde::Serialize;
use std::collections::HashMap;

mod device;
mod gate;
mod input;
mod output;

pub use device::Device;
pub use gate::{Gate, GateKind};
pub use input::Input;
pub use output::Output;

#[derive(Serialize, Debug, Clone)]
pub struct Testbench {
	pub breakpoints: HashMap<u32, HashMap<String, bool>>,
}

#[derive(Clone)]
pub struct LogicCircuit {
	pub inputs: Vec<Input>,
	pub outputs: Vec<Output>,
	pub devices: Vec<Device>,
	pub testbench: Testbench,
}

impl LogicCircuit {
	pub fn into_biological(&self, selected_genes: &Vec<usize>) -> GeneticCircuit {
		let mut components = Vec::new();
		let mut inputs = Vec::new();
		let mut cached: HashMap<String, Component> = HashMap::new();

		for inp in &self.inputs {
			let sig = inp.into_biological(&mut cached);
			let sigs: Vec<Signal> = sig.iter().map(|x| x.signal()).collect();
			inputs.extend(sigs);
		}

		for (i, selected) in selected_genes.iter().rev().enumerate() {
			let device = self.devices.get(i).unwrap();
			let batch = device.into_biological(*selected, &mut cached);
			components.extend(batch);
		}

		let genetic_circuit = GeneticCircuit {
			inputs,
			outputs: self
				.outputs
				.iter()
				.map(|x| x.into_biological(&cached))
				.collect(),
			components,
			score: None,
			simulation: None,
		};
		genetic_circuit
	}

	pub fn fit_into_biological(&self) -> Result<GeneticCircuit, Error> {
		let mut assn = GeneNetwork::init(self.clone(), 6000)?;
		let selected_genes = assn.fit()?;
		let mut gc = self.into_biological(&selected_genes);
		gc.test();
		Ok(gc)
	}
}
