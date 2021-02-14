use crate::_utils::{assigner, components, data, devices, genetic_circuit, helpers};
use assigner::GeneNetwork;
use components::{Component, Gene};
use data::get_data;
use devices::Device;
use genetic_circuit::GeneticCircuit;
use helpers::Error;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Debug, Clone)]
pub struct Testbench {
	pub breakpoints: HashMap<u32, HashMap<String, bool>>,
}

#[derive(Debug, Clone)]
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
		let mut cached: HashMap<String, Gene> = HashMap::new();
		for (i, selected) in selected_genes.iter().rev().enumerate() {
			let device = self.devices.get(i).unwrap();
			match device {
				Device::Gate(gate) => {
					let gene = gate.into_biological(*selected, &mut cached);
					components.push(Component::Gene(gene));
				}
			}
		}
		let genetic_circuit = GeneticCircuit {
			inputs: self.inputs.iter().map(|x| data.get_input(&x).clone()).collect(),
			output: cached.get(&self.output).unwrap().name(),
			components,
		};
		genetic_circuit
	}

	pub fn fit_into_biological(&self) -> Result<GeneticCircuit, Error> {
		let mut assn = GeneNetwork::init(self.clone(), 6000)?;
		let selected_genes = assn.fit()?;
		Ok(self.into_biological(&selected_genes))
	}
}
