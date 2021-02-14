use crate::_utils::{components, data, helpers, logic_circuit};
use components::Component;
use data::{get_data, Input};
use helpers::{make_plasmid_dna, make_plasmid_part, make_plasmid_title};
use logic_circuit::Testbench;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Debug)]
pub struct GeneticCircuit {
	pub inputs: Vec<Input>,
	pub output: String,
	pub components: Vec<Component>,
}

impl GeneticCircuit {
	pub fn apply_rules(&self, gc: &mut GeneticCircuit) {
		let data = get_data();
		let rules = data.get_rules();
		gc.components.sort_by(|a, b| match (a, b) {
			(Component::Gene(a), Component::Gene(b)) => {
				let a_index = rules.gates.get(&a.group()).unwrap();
				let b_index = rules.gates.get(&b.group()).unwrap();
				a_index.cmp(b_index)
			}
		});

		for component in &mut gc.components {
			match component {
				Component::Gene(gene) => {
					gene.inputs().sort_by(|a, b| {
						let a_index = rules.promoters.get(a).unwrap();
						let b_index = rules.promoters.get(b).unwrap();
						a_index.cmp(b_index)
					});
				}
			}
		}
	}

	pub fn into_dna(&self) -> (String, String) {
		let data = get_data();
		let mut gates_plasmid = String::new();
		let mut promoter_colors = HashMap::new();

		let pre_gates = data.get_part("gates_pre_backbone");
		let mut gates_dna = pre_gates.seq.to_owned();

		gates_plasmid += &make_plasmid_part(&pre_gates.kind, 0, gates_dna.len(), &pre_gates.name, "white");

		for component in &self.components {
			match component {
				Component::Gene(gene) => {
					promoter_colors.insert(gene.promoter(), gene.color());
					for inp in &gene.inputs() {
						let part = data.get_part(&inp);
						let start = gates_dna.len();
						let end = start + part.seq.len();

						gates_dna += &part.seq;
						gates_plasmid += &make_plasmid_part(
							&part.kind,
							start,
							end,
							&part.name,
							promoter_colors.get(inp).unwrap_or(&"white".to_owned()),
						);
					}

					let bio_gate = data.get_gene(&gene.name());
					for part_name in &bio_gate.parts {
						let part = data.get_part(part_name);
						let start = gates_dna.len();
						let end = start + part.seq.len();

						gates_dna += &part.seq;
						gates_plasmid += &make_plasmid_part(&part.kind, start, end, &part.name, &gene.color());
					}
				}
			}
		}

		let post_gates1 = data.get_part("gates_post_backbone1");
		let post_gates2 = data.get_part("gates_post_backbone2");

		let start1 = gates_dna.len();
		let end1 = start1 + post_gates1.seq.len();

		gates_dna += &post_gates1.seq;

		let start2 = gates_dna.len();
		let end2 = start2 + post_gates2.seq.len();

		gates_dna += &post_gates2.seq;

		gates_plasmid += &make_plasmid_part(&post_gates1.kind, start1, end1, &post_gates1.name, "white");
		gates_plasmid += &make_plasmid_part(&post_gates2.kind, start2, end2, &post_gates2.name, "white");

		let gates_title = make_plasmid_title("gates-plasmid", gates_dna.len());
		let gates_plasmid_dna: String = make_plasmid_dna(&gates_dna);
		let final_gates_plasmid = gates_title + &gates_plasmid + &gates_plasmid_dna;

		(gates_dna, final_gates_plasmid)
	}

	pub fn test(&self) -> (f64, f64, f64, f64) {
		let mut cached = HashMap::new();
		for inp in &self.inputs {
			cached.insert(
				inp.name.to_owned(),
				(inp.rpu_off, inp.rpu_on, 0.0, inp.rpu_on / inp.rpu_off),
			);
		}

		for component in &self.components {
			match component {
				Component::Gene(gene) => gene.test_steady_state(&mut cached),
			}
		}

		cached[&self.output]
	}

	pub fn simulate(&self, testbench: Testbench) -> (HashMap<String, Vec<f64>>, HashMap<String, (f64, f64)>) {
		let data = get_data();
		let mut states = HashMap::new();
		let mut history: HashMap<String, Vec<f64>> = HashMap::new();
		let mut steady_states: HashMap<String, (f64, f64)> = HashMap::new();
		for inp in &self.inputs {
			states.insert(inp.promoter.to_owned(), inp.rpu_off);
			history.insert(inp.promoter.to_owned(), Vec::new());
			steady_states.insert(inp.promoter.to_owned(), (inp.rpu_off, inp.rpu_on));
		}
		for component in &self.components {
			match component {
				Component::Gene(gene) => {
					states.insert(gene.promoter(), 0.0);
					history.insert(gene.promoter(), Vec::new());
					gene.simulation_steady_state(&mut steady_states);
				}
			}
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

			for component in &self.components {
				match component {
					Component::Gene(gene) => gene.model_and_save(&mut states, &mut history),
				}
			}
		}
		(history, steady_states)
	}
}
