use crate::_utils::{assigner, data, error, genetic_circuit};
use assigner::GeneNetwork;
use data::{get_data, Output};
use error::Error;
use genetic_circuit::{Component, GeneticCircuit};
use serde::Serialize;
use std::collections::HashMap;

pub mod device;
pub use device::Device;

pub mod gate;
pub use gate::{Gate, GateKind};

#[derive(Serialize, Debug, Clone)]
pub struct Testbench {
	pub breakpoints: HashMap<u32, HashMap<String, bool>>,
}

#[derive(Clone)]
pub struct LogicCircuit {
	pub inputs: Vec<String>,
	pub output: String,
	pub devices: Vec<Device>,
	pub testbench: Testbench,
}

impl LogicCircuit {
	pub fn into_biological(&self, selected_genes: &Vec<usize>) -> GeneticCircuit {
		let data = get_data();
		let mut components = Vec::new();
		let mut cached: HashMap<String, Component> = HashMap::new();
		for (i, selected) in selected_genes.iter().rev().enumerate() {
			let device = self.devices.get(i).unwrap();
			let batch = device.into_biological(*selected, &mut cached);
			components.extend(batch);
		}

		let out_gene = cached.get(&self.output).unwrap();
		let output = Output::new(out_gene.name(), out_gene.promoter());
		let genetic_circuit = GeneticCircuit {
			inputs: self.inputs.iter().map(|x| data.get_input(&x).clone()).collect(),
			output,
			components,
			score: None,
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
