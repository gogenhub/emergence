use crate::_utils::{builder, helpers, kd_tree};
use builder::{Gate, LogicCircut};
use fs_extra::file::read_to_string;
use helpers::{assign_err, get_promoter_kind, Error};
use kd_tree::KdTree;
use meval::{Context, Expr};
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::collections::{HashMap, HashSet};
use std::{env, f64::MAX, fs};

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
pub struct BioGate {
	name: String,
	parts: Vec<String>,
	promoter: String,
	#[serde(default = "Vec::new")]
	input_promoters: Vec<String>,
	equation: String,
	params: HashMap<String, f64>,
}

#[derive(Deserialize, Eq, Hash, Clone, Debug)]
pub enum PromoterKind {
	Repressor,
	Activator,
	Unknown,
}

impl PartialEq for PromoterKind {
	fn eq(&self, other: &PromoterKind) -> bool {
		if matches!(self, PromoterKind::Repressor) && matches!(other, PromoterKind::Repressor) {
			return true;
		}

		if matches!(self, PromoterKind::Activator) && matches!(other, PromoterKind::Activator) {
			return true;
		}

		false
	}
}

pub struct Assigner {
	trees: HashMap<PromoterKind, KdTree>,
	gates: HashMap<String, BioGate>,
	parts: HashMap<String, Part>,
	inputs: HashMap<String, Input>,
	outputs: HashMap<String, String>,
	pub loaded: bool,
}

impl Assigner {
	pub fn new() -> Self {
		Self {
			trees: HashMap::new(),
			gates: HashMap::new(),
			parts: HashMap::new(),
			inputs: HashMap::new(),
			outputs: HashMap::new(),
			loaded: false,
		}
	}
	fn assign(
		&mut self,
		new_map: &mut HashMap<String, BioGate>,
		blacklist: &mut HashSet<String>,
		assigned: &mut HashMap<String, (f64, f64, String)>,
		gates: &HashMap<String, Gate>,
		curr_gate: &str,
		promoter_kind: PromoterKind,
	) -> Result<(f64, f64, String), Error> {
		if assigned.contains_key(curr_gate) {
			return Ok(assigned.get(curr_gate).cloned().unwrap());
		}
		if self.inputs.contains_key(curr_gate) {
			let input = self.inputs.get(curr_gate).unwrap();
			blacklist.insert(input.name.to_owned());
			return Ok((input.rpu_off, input.rpu_on, input.promoter.to_owned()));
		}

		let gate = gates.get(curr_gate).unwrap();
		let new_promoter_kind = get_promoter_kind(&gate.kind);

		let mut input_promoters = Vec::new();
		let (mut x, mut y) = (0.0, MAX);
		for input in &gate.inputs {
			let (in_x, in_y, promoter) = self.assign(
				new_map,
				blacklist,
				assigned,
				gates,
				input,
				new_promoter_kind.clone(),
			)?;
			x = in_x.max(x);
			y = in_y.min(y);
			input_promoters.push(promoter);
		}

		if !self.trees.contains_key(&promoter_kind) {
			return Err(assign_err(
				format!("{} gates are not available.", gate.kind.to_string(),),
				(0, 0),
			));
		}

		let tree = self.trees.get_mut(&promoter_kind).unwrap();
		let ln = tree.search(x, y, blacklist);
		if ln.is_none() {
			return Err(assign_err("Run out of gate options.".to_owned(), (0, 0)));
		}

		let ln = ln.unwrap();
		let mut bio_gate = self.gates.get(&ln.name).cloned().unwrap();
		bio_gate.input_promoters = input_promoters;
		let promoter = bio_gate.promoter.to_owned();
		new_map.insert(ln.name.to_owned(), bio_gate);

		let new_y = self.eval(ln.name.to_owned(), x);
		let new_x = self.eval(ln.name.to_owned(), y);
		assigned.insert(curr_gate.to_owned(), (new_x, new_y, promoter.to_owned()));

		Ok((new_x, new_y, promoter))
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
		let mut new_gates = HashMap::new();
		let (_, _, promoter) = self.assign(
			&mut new_gates,
			&mut HashSet::new(),
			&mut HashMap::new(),
			&lc.gates,
			&lc.output.name,
			PromoterKind::Repressor,
		)?;

		let output = Output {
			inputs: vec![promoter],
			name: lc.output.name.to_owned(),
			seq: self.outputs.get(&lc.output.name).unwrap().to_owned(),
		};

		let mut parts = HashMap::new();
		for (_, gate) in &new_gates {
			for part in &gate.parts {
				let p = self.parts.get(part).cloned().unwrap();
				parts.insert(part.to_owned(), p);
			}
		}

		for input in &lc.inputs {
			let inp = self.inputs.get(&input.name).unwrap();
			let p = self.parts.get(&inp.promoter).cloned().unwrap();
			parts.insert(inp.promoter.to_owned(), p);
		}

		Ok((new_gates, output, parts))
	}

	pub fn eval(&self, name: String, var: f64) -> f64 {
		let g = self.gates.get(&name).unwrap();
		let expr: Expr = g.equation.parse().unwrap();
		let mut ctx = Context::new();
		ctx.var("ymin", g.params["ymin"])
			.var("ymax", g.params["ymax"])
			.var("K", g.params["K"])
			.var("n", g.params["n"]);

		let func = expr.bind_with_context(ctx, "x").unwrap();
		func(var)
	}

	pub fn load(&mut self) {
		let dir = env::current_dir().unwrap();
		let trees_path = format!("{}/static/trees.json", dir.display());
		let gates_path = format!("{}/static/gates.json", dir.display());
		let parts_path = format!("{}/static/parts.json", dir.display());
		let inputs_path = format!("{}/static/inputs.json", dir.display());
		let outputs_path = format!("{}/static/outputs.json", dir.display());

		let trees_f = read_to_string(trees_path).unwrap();
		let gates_f = read_to_string(gates_path).unwrap();
		let parts_f = read_to_string(parts_path).unwrap();
		let inputs_f = read_to_string(inputs_path).unwrap();
		let outputs_f = read_to_string(outputs_path).unwrap();

		let trees: HashMap<PromoterKind, KdTree> = from_str(&trees_f).unwrap();
		let gates: HashMap<String, BioGate> = from_str(&gates_f).unwrap();
		let parts: HashMap<String, Part> = from_str(&parts_f).unwrap();
		let inputs: HashMap<String, Input> = from_str(&inputs_f).unwrap();
		let outputs: HashMap<String, String> = from_str(&outputs_f).unwrap();

		self.trees = trees;
		self.gates = gates;
		self.parts = parts;
		self.inputs = inputs;
		self.outputs = outputs;
		self.loaded = true;
	}
}
