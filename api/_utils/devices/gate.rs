use crate::_utils::{components, data, helpers};
use colors_transform::{Color, Hsl};
use components::Gene;
use data::get_data;
use helpers::map;
use std::collections::{HashMap, HashSet};

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
	pub fn num_components(&self) -> usize {
		let data = get_data();
		data.genes_len()
	}

	pub fn blacklist(&self, i: usize, bl: &mut HashSet<String>) {
		let data = get_data();
		let gene = data.get_gene_at(i);
		bl.insert(gene.group());
	}

	pub fn is_blacklisted(&self, i: usize, bl: &HashSet<String>) -> bool {
		let data = get_data();
		let gene = data.get_gene_at(i);
		bl.contains(&gene.group())
	}

	pub fn into_biological(&self, i: usize, cached: &mut HashMap<String, Gene>) -> Gene {
		let data = get_data();
		let gene_data = data.get_gene_at(i).clone();

		let mut inputs = Vec::new();
		for inp in &self.inputs {
			let input = if data.has_input(&inp) {
				data.get_input(inp).promoter.to_owned()
			} else {
				cached.get(inp).unwrap().promoter().to_owned()
			};
			inputs.push(input);
		}

		let val = map(i, 0, data.genes_len(), 0, 355);
		let color_hex = Hsl::from(val as f32, 100.0, 50.0).to_rgb().to_css_hex_string();
		let gene = Gene::new(gene_data, inputs, color_hex);
		cached.insert(self.output.to_owned(), gene.clone());
		gene
	}
}
