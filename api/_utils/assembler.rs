use crate::_utils::{builder, data, devices, helpers};
use builder::LogicCircuit;
use data::{get_data, Gene, Input};
use devices::Device;
use helpers::{make_plasmid_dna, make_plasmid_part, make_plasmid_title};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Debug)]
pub struct GeneticCircuit {
	pub inputs: Vec<Input>,
	pub output: String,
	pub genes: Vec<Gene>,
	pub dna: String,
	pub plasmid: String,
	pub score: f64,
}

pub struct Assembler {
	selected_genes: Vec<usize>,
	score: f64,
}

impl Assembler {
	pub fn new(selected_genes: Vec<usize>, score: f64) -> Self {
		Self { selected_genes, score }
	}
	pub fn assemble(&self, lc: &LogicCircuit) -> GeneticCircuit {
		let data = get_data();
		let mut genes = Vec::new();
		let mut cached: HashMap<String, Gene> = HashMap::new();
		for (i, selected) in self.selected_genes.iter().rev().enumerate() {
			let device = lc.devices.get(i).unwrap();
			match device {
				Device::Gate(gate) => {
					let gene = gate.into_biological(*selected, &mut cached);
					genes.push(gene);
				}
			}
		}
		let (dna, plasmid) = self.make_dna(&genes);
		let genetic_circuit = GeneticCircuit {
			inputs: lc.inputs.iter().map(|x| data.get_input(&x).clone()).collect(),
			output: cached.get(&lc.output).unwrap().name.to_owned(),
			genes,
			dna,
			plasmid,
			score: self.score,
		};
		genetic_circuit
	}

	pub fn apply_rules(&self, gc: &mut GeneticCircuit) {
		let data = get_data();
		let rules = data.get_rules();
		gc.genes.sort_by(|a, b| {
			let a_index = rules.gates.get(&a.group()).unwrap();
			let b_index = rules.gates.get(&b.group()).unwrap();
			a_index.cmp(b_index)
		});

		for gene in &mut gc.genes {
			gene.inputs.sort_by(|a, b| {
				let a_index = rules.promoters.get(a).unwrap();
				let b_index = rules.promoters.get(b).unwrap();
				a_index.cmp(b_index)
			});
		}
	}

	pub fn make_dna(&self, genes: &Vec<Gene>) -> (String, String) {
		let data = get_data();
		let mut gates_plasmid = String::new();
		let mut promoter_colors = HashMap::new();

		let pre_gates = data.get_part("gates_pre_backbone");
		let mut gates_dna = pre_gates.seq.to_owned();

		gates_plasmid += &make_plasmid_part(&pre_gates.kind, 0, gates_dna.len(), &pre_gates.name, "white");

		for gene in genes {
			promoter_colors.insert(gene.promoter.to_owned(), gene.color.to_owned());
			for inp in &gene.inputs {
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

			let bio_gate = data.get_gene(&gene.name);
			for part_name in &bio_gate.parts {
				let part = data.get_part(part_name);
				let start = gates_dna.len();
				let end = start + part.seq.len();

				gates_dna += &part.seq;
				gates_plasmid += &make_plasmid_part(&part.kind, start, end, &part.name, &gene.color);
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
}
