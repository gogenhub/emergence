pub mod component;
pub mod gene;

pub use component::Component;
pub use gene::Gene;

use crate::_utils::{data, dna, logic_circuit};
use data::{get_data, Input, Output};
use dna::Dna;
use logic_circuit::Testbench;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Debug)]
pub struct SimulationData {
	history: HashMap<String, Vec<f64>>,
	steady_states: HashMap<String, (f64, f64)>,
}

#[derive(Serialize, Debug)]
pub struct GeneticCircuit {
	pub inputs: Vec<Input>,
	pub output: Output,
	pub components: Vec<Component>,
	pub score: Option<f64>,
	pub simulation: Option<SimulationData>,
}

impl GeneticCircuit {
	pub fn apply_rules(&mut self) {
		let data = get_data();
		let rules = data.get_rules();
		self.components.sort_by(|a, b| {
			let a_index = rules.gates.get(&a.group()).unwrap();
			let b_index = rules.gates.get(&b.group()).unwrap();
			a_index.cmp(b_index)
		});

		for comp in &mut self.components {
			comp.apply_rules();
		}
	}

	pub fn inv_diff_error(x: f64) -> f64 {
		(-x / 10.0).exp()
	}

	pub fn into_dna(&self) -> Dna {
		let data = get_data();
		let mut gates_plasmid = String::new();
		let mut promoter_colors = HashMap::new();

		let pre_gates = data.get_part("gates_pre_backbone");
		let mut gates_dna = pre_gates.seq.to_owned();

		gates_plasmid += &Dna::make_plasmid_part(&pre_gates.kind, 0, gates_dna.len(), &pre_gates.name, "white");

		for comp in &self.components {
			comp.into_dna(&mut gates_dna, &mut gates_plasmid, &mut promoter_colors);
		}

		let post_gates1 = data.get_part("gates_post_backbone1");
		let post_gates2 = data.get_part("gates_post_backbone2");

		let start1 = gates_dna.len();
		let end1 = start1 + post_gates1.seq.len();

		gates_dna += &post_gates1.seq;

		let start2 = gates_dna.len();
		let end2 = start2 + post_gates2.seq.len();

		gates_dna += &post_gates2.seq;

		gates_plasmid += &Dna::make_plasmid_part(&post_gates1.kind, start1, end1, &post_gates1.name, "white");
		gates_plasmid += &Dna::make_plasmid_part(&post_gates2.kind, start2, end2, &post_gates2.name, "white");

		let gates_title = Dna::make_plasmid_title("gates-plasmid", gates_dna.len());
		let gates_plasmid_dna: String = Dna::make_plasmid_dna(&gates_dna);
		let final_gates_plasmid = gates_title + &gates_plasmid + &gates_plasmid_dna;

		Dna {
			raw: gates_dna,
			plasmid: final_gates_plasmid,
		}
	}

	pub fn test(&mut self) -> f64 {
		let mut cached = HashMap::new();
		for inp in &self.inputs {
			cached.insert(inp.promoter(), (inp.rpu_off, inp.rpu_on, 0.0, inp.rpu_on / inp.rpu_off));
		}

		for comp in &self.components {
			comp.test_steady_state(&mut cached);
		}

		let (_, _, diff, score) = cached[&self.output.promoter()];
		let diff_err = Self::inv_diff_error(diff);
		let diff_score = diff_err * score;

		self.score = Some(diff_score);
		diff_score
	}

	pub fn simulate(&mut self, testbench: Testbench) {
		let data = get_data();
		let mut states = HashMap::new();
		let mut history: HashMap<String, Vec<f64>> = HashMap::new();
		let mut steady_states: HashMap<String, (f64, f64)> = HashMap::new();
		for inp in &self.inputs {
			states.insert(inp.promoter.to_owned(), inp.rpu_off);
			history.insert(inp.promoter.to_owned(), Vec::new());
			steady_states.insert(inp.promoter.to_owned(), (inp.rpu_off, inp.rpu_on));
		}
		for comp in &self.components {
			states.insert(comp.promoter(), 0.0);
			history.insert(comp.promoter(), Vec::new());
			comp.simulation_steady_state(&mut steady_states);
		}
		for i in 0..1000 {
			if testbench.breakpoints.contains_key(&i) {
				let bp = testbench.breakpoints.get(&i).unwrap();
				for (name, val) in bp {
					let inp = data.get_input(name);
					states.insert(
						inp.promoter.to_owned(),
						if *val == true { inp.rpu_on } else { inp.rpu_off },
					);
				}
			}

			for inp in &self.inputs {
				let state = states.get(&inp.promoter).unwrap();
				let hist = history.get_mut(&inp.promoter).unwrap();
				hist.push(*state);
			}

			for comp in &self.components {
				comp.model_and_save(&mut states, &mut history);
			}
		}
		self.simulation = Some(SimulationData { history, steady_states })
	}
}
