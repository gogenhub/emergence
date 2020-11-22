use fs_extra::file::read_to_string;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::collections::{HashMap, HashSet};
use std::env;

static DATA: Lazy<Data> = Lazy::new(|| {
	let mut d = Data::new();
	d.load();
	d
});

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum PartKind {
	Promoter,
	Cds,
	Ribozyme,
	Terminator,
	Rbs,
	Scar,
	SgRNA,
	Backbone,
	Output,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Part {
	pub kind: PartKind,
	pub name: String,
	pub seq: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Input {
	pub name: String,
	pub promoter: String,
	pub rpu_off: f64,
	pub rpu_on: f64,
}

#[derive(Serialize, Debug)]
pub struct Gene {
	pub inputs: Vec<String>,
	pub promoter: String,
	pub color: String,
	pub name: String,
}

#[derive(Serialize, Debug)]
pub struct OutputGene {
	pub name: String,
	pub inputs: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct GeneticCircuit {
	pub output: OutputGene,
	pub inputs: Vec<Input>,
	pub genes: Vec<Gene>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Params {
	pub ymax: f64,
	pub ymin: f64,
	#[serde(alias = "K")]
	pub k: f64,
	pub n: f64,
	pub decay: f64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct BioGate {
	pub name: String,
	pub parts: Vec<String>,
	pub promoter: String,
	pub params: Params,
}

#[derive(Clone, Debug)]
pub struct AssignedGate {
	on: f64,
	off: f64,
	min: f64,
	name: String,
	rpu: f64,
}

#[derive(Deserialize)]
pub struct Rules {
	pub gates: HashMap<String, u32>,
	pub promoters: HashMap<String, u32>,
}

pub struct Data {
	pub gates: HashMap<String, BioGate>,
	pub gates_vec: Vec<BioGate>,
	pub parts: HashMap<String, Part>,
	pub inputs: HashMap<String, Input>,
	pub outputs: HashMap<String, String>,
	pub rules: Rules,
	pub roadblock: HashSet<String>,
}

impl Data {
	pub fn new() -> Self {
		Self {
			gates: HashMap::new(),
			gates_vec: Vec::new(),
			parts: HashMap::new(),
			inputs: HashMap::new(),
			outputs: HashMap::new(),
			rules: Rules {
				gates: HashMap::new(),
				promoters: HashMap::new(),
			},
			roadblock: HashSet::new(),
		}
	}

	pub fn load(&mut self) {
		let dir = env::current_dir().unwrap();
		let gates_path = format!("{}/static/gates.json", dir.display());
		let parts_path = format!("{}/static/parts.json", dir.display());
		let inputs_path = format!("{}/static/inputs.json", dir.display());
		let outputs_path = format!("{}/static/outputs.json", dir.display());
		let rules_path = format!("{}/static/rules.json", dir.display());
		let roadblock_path = format!("{}/static/roadblock.json", dir.display());

		let gates_f = read_to_string(gates_path).unwrap();
		let parts_f = read_to_string(parts_path).unwrap();
		let inputs_f = read_to_string(inputs_path).unwrap();
		let outputs_f = read_to_string(outputs_path).unwrap();
		let rules_f = read_to_string(rules_path).unwrap();
		let roadblock_f = read_to_string(roadblock_path).unwrap();

		let gates: HashMap<String, BioGate> = from_str(&gates_f).unwrap();
		let parts: HashMap<String, Part> = from_str(&parts_f).unwrap();
		let inputs: HashMap<String, Input> = from_str(&inputs_f).unwrap();
		let outputs: HashMap<String, String> = from_str(&outputs_f).unwrap();
		let rules: HashMap<String, Vec<String>> = from_str(&rules_f).unwrap();
		let roadblock: HashSet<String> = from_str(&roadblock_f).unwrap();

		let gate_rules = rules.get("gates").unwrap();
		let promoter_rules = rules.get("promoters").unwrap();
		let new_rules: Rules = Rules {
			gates: gate_rules
				.iter()
				.enumerate()
				.map(|(i, name)| (name.to_owned(), i as u32))
				.collect(),
			promoters: promoter_rules
				.iter()
				.enumerate()
				.map(|(i, name)| (name.to_owned(), i as u32))
				.collect(),
		};

		self.gates = gates.clone();
		self.gates_vec = gates.values().cloned().collect();
		self.parts = parts;
		self.inputs = inputs;
		self.rules = new_rules;
		self.outputs = outputs;
		self.roadblock = roadblock;
	}
}

pub fn get_part(name: &str) -> &Part {
	DATA.parts.get(name).unwrap()
}

pub fn get_gate(name: &str) -> &BioGate {
	DATA.gates.get(name).unwrap()
}

pub fn get_gate_at(i: usize) -> &'static BioGate {
	DATA.gates_vec.get(i).unwrap()
}

pub fn get_rules() -> &'static Rules {
	&DATA.rules
}

pub fn get_input(name: &str) -> &Input {
	DATA.inputs.get(name).unwrap()
}

pub fn has_input(name: &str) -> bool {
	DATA.inputs.contains_key(name)
}

pub fn get_output(name: &str) -> &str {
	DATA.outputs.get(name).unwrap()
}

pub fn has_output(name: &str) -> bool {
	DATA.outputs.contains_key(name)
}

pub fn gates_len() -> usize {
	DATA.gates_vec.len()
}
