use crate::_utils::{data, genetic_circuit};
use colors_transform::{Color, Hsl};
use data::get_data;
use genetic_circuit::{Component, Gene};
use std::collections::HashMap;

pub fn map(num: u32, in_min: u32, in_max: u32, out_min: u32, out_max: u32) -> u32 {
	(num - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}

#[derive(Clone, Debug, PartialEq)]
pub enum GateKind {
	Not,
	Nor,
}

#[derive(Debug, Clone)]
pub struct Gate {
	pub output: String,
	pub inputs: Vec<String>,
	pub kind: GateKind,
}

impl Gate {
	pub fn num_biological(&self) -> usize {
		let data = get_data();
		data.genes_len()
	}

	pub fn into_biological(
		&self,
		i: usize,
		cached: &mut HashMap<String, Component>,
	) -> Vec<Component> {
		let data = get_data();
		let gene_data = data.get_gene_at(i).clone();

		let mut inputs = Vec::new();
		for inp in &self.inputs {
			let input = cached.get(inp).unwrap().promoter().to_string();
			inputs.push(input);
		}

		let val = map(i as u32, 0, data.genes_len() as u32, 0, 355);
		let color_hex = Hsl::from(val as f32, 100.0, 50.0)
			.to_rgb()
			.to_css_hex_string();
		let gene = Gene {
			inputs,
			data: gene_data,
			color: color_hex,
		};
		cached.insert(self.output.to_string(), Component::Gene(gene.clone()));
		vec![Component::Gene(gene)]
	}
}
