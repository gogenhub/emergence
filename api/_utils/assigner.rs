use crate::_utils::{builder, helpers, kd_tree};
use builder::{Gate, LogicCircut};
use fs_extra::file::read_to_string;
use helpers::{assign_err, Error};
use kd_tree::{KdTree, LeafNode};
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::collections::{HashMap, HashSet};
use std::env;

#[derive(Deserialize, Serialize, Debug, Clone)]
enum PartKind {
	Promoter,
	Cds,
	Ribozyme,
	Terminator,
	Rbs,
	Scar,
	SgRNA,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Part {
	kind: PartKind,
	name: String,
	seq: String,
}

#[derive(Deserialize, Debug)]
struct Input {
	name: String,
	promoter: String,
	rpu_off: f64,
	rpu_on: f64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Output {
	inputs: Vec<String>,
	name: String,
	seq: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct Params {
	ymax: f64,
	ymin: f64,
	#[serde(alias = "K")]
	k: f64,
	n: f64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct BioGate {
	name: String,
	parts: Vec<String>,
	promoter: String,
	#[serde(default = "Vec::new")]
	inputs: Vec<String>,
	params: Params,
}

pub struct Assigner {
	kd: KdTree,
	gates: HashMap<String, BioGate>,
	parts: HashMap<String, Part>,
	inputs: HashMap<String, Input>,
	outputs: HashMap<String, String>,
	pub loaded: bool,
}

fn transfer(x: f64, params: &Params) -> f64 {
	params.ymin + (params.ymax - params.ymin) / (1.0 + (x / params.k).powf(params.n))
}

fn get_group(curr: String) -> String {
	let group: Vec<&str> = curr.split("_").collect();
	group[1].to_owned()
}

impl Assigner {
	pub fn new() -> Self {
		Self {
			kd: KdTree::new(3),
			gates: HashMap::new(),
			parts: HashMap::new(),
			inputs: HashMap::new(),
			outputs: HashMap::new(),
			loaded: false,
		}
	}

	pub fn load(&mut self) {
		let dir = env::current_dir().unwrap();
		let tree_path = format!("{}/static/tree.json", dir.display());
		let gates_path = format!("{}/static/gates.json", dir.display());
		let parts_path = format!("{}/static/parts.json", dir.display());
		let inputs_path = format!("{}/static/inputs.json", dir.display());
		let outputs_path = format!("{}/static/outputs.json", dir.display());

		let trees_f = read_to_string(tree_path).unwrap();
		let gates_f = read_to_string(gates_path).unwrap();
		let parts_f = read_to_string(parts_path).unwrap();
		let inputs_f = read_to_string(inputs_path).unwrap();
		let outputs_f = read_to_string(outputs_path).unwrap();

		let tree: KdTree = from_str(&trees_f).unwrap();
		let gates: HashMap<String, BioGate> = from_str(&gates_f).unwrap();
		let parts: HashMap<String, Part> = from_str(&parts_f).unwrap();
		let inputs: HashMap<String, Input> = from_str(&inputs_f).unwrap();
		let outputs: HashMap<String, String> = from_str(&outputs_f).unwrap();

		self.kd = tree;
		self.gates = gates;
		self.parts = parts;
		self.inputs = inputs;
		self.outputs = outputs;
		self.loaded = true;
	}

	pub fn assign_gates(
		&mut self,
		lc: LogicCircut,
	) -> Result<(HashMap<String, BioGate>, Output, HashMap<String, Part>), Error> {
		if !self.outputs.contains_key(&lc.output.name) {
			return Err(assign_err(
				format!("Output '{}' not found.", lc.output.name),
				(lc.output.pos, lc.output.name.len()),
			));
		}
		for input in &lc.inputs {
			if !self.inputs.contains_key(&input.name) {
				return Err(assign_err(
					format!("Input '{}' not found.", input.name),
					(input.pos, input.name.len()),
				));
			}
		}

		let (promoter, assigned_gates) = self.try_walk(&lc, 1.0)?;

		let output = Output {
			inputs: vec![promoter],
			name: lc.output.name.to_owned(),
			seq: self.outputs.get(&lc.output.name).unwrap().to_owned(),
		};

		let mut parts = HashMap::new();
		for (_, gate) in &assigned_gates {
			for part in &gate.parts {
				let p = self.parts.get(part).cloned().unwrap();
				parts.insert(part.to_owned(), p);
			}

			for input in &gate.inputs {
				let p = self.parts.get(input).cloned().unwrap();
				parts.insert(input.to_owned(), p);
			}
		}

		for input in &output.inputs {
			let p = self.parts.get(input).cloned().unwrap();
			parts.insert(input.to_owned(), p);
		}

		for input in &lc.inputs {
			let inp = self.inputs.get(&input.name).unwrap();
			let p = self.parts.get(&inp.promoter).cloned().unwrap();
			parts.insert(inp.promoter.to_owned(), p);
		}

		Ok((assigned_gates, output, parts))
	}

	fn try_walk(
		&self,
		lc: &LogicCircut,
		min: f64,
	) -> Result<(String, HashMap<String, BioGate>), Error> {
		let mut assigned_gates = HashMap::new();
		let (_, _, new_min, _, promoter, _) = self.walk_back(
			lc.output.name.to_owned(),
			&lc.gates,
			&HashSet::new(),
			&mut HashMap::new(),
			&mut assigned_gates,
			min,
		)?;

		let chres = self.try_walk(lc, new_min);
		if chres.is_ok() {
			let (pr, assgt) = chres.unwrap();
			return Ok((pr, assgt));
		} else {
			return Ok((promoter, assigned_gates));
		}
	}

	fn walk_back(
		&self,
		curr_gate: String,
		gates: &HashMap<String, Gate>,
		ext_bl: &HashSet<String>,
		gate_bl: &mut HashMap<String, HashSet<String>>,
		assigned_gates: &mut HashMap<String, BioGate>,
		min: f64,
	) -> Result<(f64, f64, f64, String, String, HashSet<String>), Error> {
		if !gate_bl.contains_key(&curr_gate) {
			gate_bl.insert(curr_gate.to_owned(), HashSet::new());
		}
		if self.inputs.contains_key(&curr_gate) {
			let in_rpus = self.inputs.get(&curr_gate).unwrap();
			return Ok((
				in_rpus.rpu_off,
				in_rpus.rpu_on,
				in_rpus.rpu_on / in_rpus.rpu_off,
				"0_0".to_string(),
				in_rpus.promoter.to_owned(),
				HashSet::new(),
			));
		}

		let gate = gates.get(&curr_gate).unwrap();
		let mut res_bl = ext_bl.clone();
		let (mut new_on, mut new_off, mut new_min, mut names, mut promoters): (
			f64,
			f64,
			f64,
			Vec<String>,
			Vec<String>,
		) = (0.0, f64::MAX, f64::MAX, vec![], vec![]);
		for inp in &gate.inputs {
			let (con, coff, cmin, name, promoter, bl) =
				self.walk_back(inp.to_owned(), gates, &res_bl, gate_bl, assigned_gates, min)?;
			res_bl.extend(bl);
			names.push(name);
			promoters.push(promoter);
			new_on = con.max(new_on);
			new_off = coff.min(new_off);
			new_min = cmin.min(new_min);
		}

		let mut gbl = gate_bl.get(&curr_gate).cloned().unwrap();
		gbl.extend(res_bl.clone());
		let node = self.kd.search(vec![new_on, new_off, 1000.0], &gbl);

		let (name, max_on, max_off, max_rpu) = self.get_on_off(new_on, new_off, node);
		if max_rpu <= min {
			if self.inputs.contains_key(&gate.inputs[0]) {
				return Err(assign_err(
					"Failed to find optimal genetic circuit!".to_owned(),
					(0, 0),
				));
			}
			let currgbl = gate_bl.get_mut(&gate.inputs[0].to_owned()).unwrap();
			currgbl.insert(names[0].to_owned());
			return self.walk_back(curr_gate, gates, ext_bl, gate_bl, assigned_gates, min);
		}

		res_bl.insert(get_group(name.to_owned()));

		let mut bio_gate = self.gates.get(&name).cloned().unwrap();
		let curr_promoter = bio_gate.promoter.to_owned();
		bio_gate.inputs = promoters;
		assigned_gates.insert(curr_gate, bio_gate);

		Ok((
			max_off,
			max_on,
			new_min.min(max_rpu),
			name,
			curr_promoter,
			res_bl,
		))
	}

	fn get_on_off(&self, on: f64, off: f64, node: Option<LeafNode>) -> (String, f64, f64, f64) {
		if node.is_none() {
			return ("0_0".to_string(), 0.0, 0.0, 0.0);
		}
		let node = node.unwrap();
		let gate = self.gates.get(&node.name).unwrap();
		let new_on = transfer(on, &gate.params);
		let new_off = transfer(off, &gate.params);

		(node.name, new_on, new_off, new_on / new_off)
	}
}
