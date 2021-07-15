use crate::genetic_circuit::Signal;
use fs_extra::file::read_to_string;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::{
	collections::{HashMap, HashSet},
	env,
};

static DATA: Lazy<Data> = Lazy::new(|| {
	let mut d = Data::new();
	d.load();
	d
});

pub fn get_data() -> &'static Lazy<Data> {
	&DATA
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum PartKind {
	Promoter,
	Cds,
	Ribozyme,
	Terminator,
	Rbs,
	Scar,
	SgRNA,
	Backbone,
	Actuator,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Part {
	pub kind: PartKind,
	pub name: String,
	pub seq: String,
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
pub struct GeneData {
	pub name: String,
	pub parts: Vec<String>,
	pub promoter: String,
	pub params: Params,
}

impl GeneData {
	pub fn group(&self) -> String {
		let group: Vec<&str> = self.name.split("_").collect();
		if group.len() < 2 {
			return "none".to_string();
		}
		group[1].to_string()
	}

	pub fn blacklist(&self, bl: &mut HashSet<String>) {
		bl.insert(self.group());
	}

	pub fn is_blacklisted(&self, bl: &HashSet<String>) -> bool {
		bl.contains(&self.group())
	}
}

#[derive(Deserialize)]
pub struct Rules {
	pub gates: HashMap<String, u32>,
	pub promoters: HashMap<String, u32>,
}

pub struct Data {
	pub genes: Vec<GeneData>,
	pub parts: HashMap<String, Part>,
	pub signals: HashMap<String, Signal>,
	pub rules: Rules,
	pub roadblock: HashSet<String>,
}

impl Data {
	pub fn new() -> Self {
		Self {
			genes: Vec::new(),
			parts: HashMap::new(),
			signals: HashMap::new(),
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
		let signals_path = format!("{}/static/signals.json", dir.display());
		let rules_path = format!("{}/static/rules.json", dir.display());
		let roadblock_path = format!("{}/static/roadblock.json", dir.display());

		let gates_f = read_to_string(gates_path).unwrap();
		let parts_f = read_to_string(parts_path).unwrap();
		let signals_f = read_to_string(signals_path).unwrap();
		let rules_f = read_to_string(rules_path).unwrap();
		let roadblock_f = read_to_string(roadblock_path).unwrap();

		let genes: Vec<GeneData> = from_str(&gates_f).unwrap();
		let parts: HashMap<String, Part> = from_str(&parts_f).unwrap();
		let signals: HashMap<String, Signal> = from_str(&signals_f).unwrap();
		let rules: HashMap<String, Vec<String>> = from_str(&rules_f).unwrap();
		let roadblock: HashSet<String> = from_str(&roadblock_f).unwrap();

		let gate_rules = rules.get("gates").unwrap();
		let promoter_rules = rules.get("promoters").unwrap();
		let new_rules: Rules = Rules {
			gates: gate_rules
				.iter()
				.enumerate()
				.map(|(i, name)| (name.to_string(), i as u32))
				.collect(),
			promoters: promoter_rules
				.iter()
				.enumerate()
				.map(|(i, name)| (name.to_string(), i as u32))
				.collect(),
		};

		self.genes = genes;
		self.parts = parts;
		self.signals = signals;
		self.rules = new_rules;
		self.roadblock = roadblock;
	}

	pub fn get_part(&self, name: &str) -> &Part {
		self.parts.get(name).unwrap()
	}

	pub fn get_gene_at(&self, i: usize) -> &GeneData {
		self.genes.get(i).unwrap()
	}

	pub fn get_rules(&self) -> &Rules {
		&self.rules
	}

	pub fn get_signal(&self, name: &str) -> &Signal {
		self.signals.get(name).unwrap()
	}

	pub fn has_actuator(&self, name: &str) -> bool {
		if !self.parts.contains_key(name) {
			return false;
		}
		let part = self.parts.get(name).unwrap();
		part.kind == PartKind::Actuator
	}

	pub fn has_signal(&self, name: &str) -> bool {
		self.signals.contains_key(name)
	}

	pub fn genes_len(&self) -> usize {
		self.genes.len()
	}

	pub fn signals_len(&self) -> usize {
		self.signals.len()
	}
}
