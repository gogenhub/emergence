use crate::_utils::{data, helpers};
use colors_transform::{Color, Hsl};
use data::{get_data, Gene};
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
		let mut gene = data.get_gene_at(i).clone();
		let mut inputs = Vec::new();
		for inp in &self.inputs {
			let input = if data.has_input(&inp) {
				data.get_input(inp).promoter.to_owned()
			} else {
				cached.get(inp).unwrap().promoter.to_owned()
			};
			inputs.push(input);
		}
		gene.inputs = inputs;

		let val = map(i, 0, data.genes_len(), 0, 355);
		let color_hex = Hsl::from(val as f32, 100.0, 50.0).to_rgb().to_css_hex_string();
		gene.color = color_hex;
		cached.insert(self.output.to_owned(), gene.clone());
		gene
	}

	pub fn test_steady_state(&self, i: usize, cached: &mut HashMap<String, (f64, f64, f64)>) {
		let (off, on, diff) = match self.kind {
			GateKind::Not => cached[&self.inputs[0]],
			GateKind::Nor => {
				let (coff0, con0, diff0) = cached[&self.inputs[0]];
				let (coff1, con1, diff1) = cached[&self.inputs[1]];
				let diff = (con0 - con1).abs() + (coff0 - coff1).abs() + diff0 + diff1;
				(coff0 + coff1, con0.min(con1), diff)
			}
		};

		let data = get_data();
		let gene = data.get_gene_at(i);
		let (off, on) = gene.steady_state(on, off);
		cached.insert(self.output.to_string(), (off, on, diff));
	}
}
