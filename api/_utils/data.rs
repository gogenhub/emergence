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

pub fn get_data() -> &'static Lazy<Data> {
	&DATA
}

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
pub struct Gene {
	pub name: String,
	pub parts: Vec<String>,
	pub promoter: String,
	#[serde(default)]
	pub color: String,
	#[serde(default)]
	pub inputs: Vec<String>,
	pub params: Params,
	#[serde(default)]
	pub state: f64,
}

impl Gene {
	pub fn group(&self) -> String {
		let group: Vec<&str> = self.name.split("_").collect();
		if group.len() < 2 {
			return "none".to_owned();
		}
		group[1].to_owned()
	}

	pub fn transfer(&self, x: f64) -> f64 {
		self.params.ymin + (self.params.ymax - self.params.ymin) / (1.0 + (x / self.params.k).powf(self.params.n))
	}

	pub fn model(&self, sum: f64) -> f64 {
		self.transfer(sum) - self.params.decay * self.state
	}

	pub fn steady_state(&self, steady_states: HashMap<String, (f64, f64)>) -> (f64, f64) {
		let (mut sum_off, mut sum_on) = (0.0, 0.0);
		for inp in &self.inputs {
			let (off, on) = steady_states.get(inp).unwrap();
			sum_on += on;
			sum_off += off;
		}

		let steady_off = self.transfer(sum_on) / self.params.decay;
		let steady_on = self.transfer(sum_off) / self.params.decay;
		(steady_off, steady_on)
	}
}

#[derive(Deserialize)]
pub struct Rules {
	pub gates: HashMap<String, u32>,
	pub promoters: HashMap<String, u32>,
}

pub struct Data {
	pub genes: HashMap<String, Gene>,
	pub genes_vec: Vec<Gene>,
	pub parts: HashMap<String, Part>,
	pub inputs: HashMap<String, Input>,
	pub outputs: HashMap<String, String>,
	pub rules: Rules,
	pub roadblock: HashSet<String>,
}

impl Data {
	pub fn new() -> Self {
		Self {
			genes: HashMap::new(),
			genes_vec: Vec::new(),
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
		let gates_path = format!("{}/static/genes.json", dir.display());
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

		let genes: HashMap<String, Gene> = from_str(&gates_f).unwrap();
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

		self.genes = genes.clone();
		self.genes_vec = genes.values().cloned().collect();
		self.parts = parts;
		self.inputs = inputs;
		self.rules = new_rules;
		self.outputs = outputs;
		self.roadblock = roadblock;
	}

	pub fn get_part(&self, name: &str) -> &Part {
		self.parts.get(name).unwrap()
	}

	pub fn get_gene(&self, name: &str) -> &Gene {
		self.genes.get(name).unwrap()
	}

	pub fn get_gene_at(&self, i: usize) -> &Gene {
		self.genes_vec.get(i).unwrap()
	}

	pub fn get_rules(&self) -> &Rules {
		&self.rules
	}

	pub fn get_input(&self, name: &str) -> &Input {
		self.inputs.get(name).unwrap()
	}

	pub fn has_input(&self, name: &str) -> bool {
		self.inputs.contains_key(name)
	}

	pub fn genes_len(&self) -> usize {
		self.genes_vec.len()
	}
}
