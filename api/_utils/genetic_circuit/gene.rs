use crate::_utils::{data, dna, genetic_circuit};
use data::{get_data, GeneData};
use dna::Dna;
use genetic_circuit::GeneticCircuit;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Gene {
	data: GeneData,
	color: String,
	inputs: Vec<String>,
}

impl Gene {
	pub fn new(data: GeneData, inputs: Vec<String>, color: String) -> Self {
		Self { data, inputs, color }
	}

	pub fn group(&self) -> String {
		self.data.group()
	}

	pub fn promoter(&self) -> String {
		self.data.promoter.to_string()
	}

	pub fn name(&self) -> String {
		self.data.name.to_string()
	}

	pub fn color(&self) -> String {
		self.color.to_string()
	}

	pub fn inputs(&self) -> Vec<String> {
		self.inputs.clone()
	}

	pub fn apply_rules(&mut self) {
		let data = get_data();
		let rules = data.get_rules();
		self.inputs().sort_by(|a, b| {
			let a_index = rules.promoters.get(a).unwrap();
			let b_index = rules.promoters.get(b).unwrap();
			a_index.cmp(b_index)
		});
	}

	pub fn transfer(&self, x: f64) -> f64 {
		let data = &self.data;
		data.params.ymin + (data.params.ymax - data.params.ymin) / (1.0 + (x / data.params.k).powf(data.params.n))
	}

	pub fn model(&self, sum: f64, state: f64) -> f64 {
		self.transfer(sum) - self.data.params.decay * state
	}

	pub fn model_and_save(&self, states: &mut HashMap<String, f64>, history: &mut HashMap<String, Vec<f64>>) {
		let promoter = &self.data.promoter;
		let sum: f64 = self.inputs.iter().map(|pro| states.get(pro).unwrap()).sum();
		let state = states.get(promoter).unwrap();
		let flux = self.model(sum, *state);
		let new_state = state + flux;
		states.insert(promoter.to_owned(), new_state);
		let hist = history.get_mut(promoter).unwrap();
		hist.push(new_state);
	}

	pub fn steady_state(&self, on: f64, off: f64) -> (f64, f64) {
		let data = &self.data;
		let steady_off = self.transfer(on) / data.params.decay;
		let steady_on = self.transfer(off) / data.params.decay;
		(steady_off, steady_on)
	}

	pub fn simulation_steady_state(&self, cached: &mut HashMap<String, (f64, f64)>) {
		let data = &self.data;
		let (mut sum_off, mut sum_on) = (0.0, 0.0);
		for inp in &self.inputs {
			let (off, on) = cached.get(inp).unwrap();
			sum_on += on;
			sum_off += off;
		}

		let (off, on) = self.steady_state(sum_on, sum_off);
		cached.insert(data.promoter.to_owned(), (off, on));
	}

	pub fn test_steady_state(&self, cached: &mut HashMap<String, (f64, f64, f64, f64)>) {
		let curr_std = if self.inputs().len() == 2 {
			let (coff0, con0, diff0, _) = cached[&self.inputs[0]];
			let (coff1, con1, diff1, _) = cached[&self.inputs[1]];
			let diff = (con0 - con1).abs() + (coff0 - coff1).abs() + diff0 + diff1;

			let next_off = coff0 + coff1;
			let next_on = con0.min(con1);
			let next_diff = diff;
			let next_score = (con0 + con1) / (coff0 + coff1);
			(next_off, next_on, next_diff, next_score)
		} else {
			cached[&self.inputs[0]]
		};

		let (off, on) = self.steady_state(curr_std.1, curr_std.0);
		cached.insert(self.promoter(), (off, on, curr_std.2, curr_std.3));
	}

	pub fn into_dna(&self, dna: &mut String, plasmid: &mut String, promoter_colors: &mut HashMap<String, String>) {
		let data = get_data();
		for inp in &self.inputs() {
			let part = data.get_part(&inp);
			let start = dna.len();
			let end = start + part.seq.len();

			dna.push_str(&part.seq);
			plasmid.push_str(&Dna::make_plasmid_part(
				&part.kind,
				start,
				end,
				&part.name,
				promoter_colors.get(inp).unwrap_or(&"white".to_owned()),
			));
		}

		for part_name in &self.data.parts {
			let part = data.get_part(part_name);
			let start = dna.len();
			let end = start + part.seq.len();

			dna.push_str(&part.seq);
			plasmid.push_str(&Dna::make_plasmid_part(
				&part.kind,
				start,
				end,
				&part.name,
				&self.color(),
			));
		}

		promoter_colors.insert(self.promoter(), self.color());
	}
}
