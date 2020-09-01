use crate::_utils::{builder, helpers, kd_tree};
use builder::LogicCircuit;
use colors_transform::{Color, Hsl};
use fs_extra::file::read_to_string;
use helpers::{
	assign_err, gate_logic, get_group, make_plasmid_dna, make_plasmid_part, make_plasmid_title,
	map, transfer, Error,
};
use kd_tree::{KdTree, LeafNode};
use serde::{Deserialize, Serialize};
use serde_json::from_str;
use std::collections::{HashMap, HashSet};
use std::env;

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
	kind: PartKind,
	name: String,
	seq: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Input {
	name: String,
	pub promoter: String,
	pub rpu_off: f64,
	pub rpu_on: f64,
}

#[derive(Serialize, Debug)]
struct Gene {
	inputs: Vec<String>,
	promoter: String,
	color: String,
	name: String,
}

#[derive(Serialize, Debug)]
struct OutputGene {
	name: String,
	inputs: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct GeneticCircuit {
	output: OutputGene,
	inputs: Vec<String>,
	genes: Vec<Gene>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Params {
	pub ymax: f64,
	pub ymin: f64,
	#[serde(alias = "K")]
	pub k: f64,
	pub n: f64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct BioGate {
	name: String,
	parts: Vec<String>,
	pub promoter: String,
	pub params: Params,
}

pub struct Assembler {
	kd: KdTree,
	gates: HashMap<String, BioGate>,
	parts: HashMap<String, Part>,
	inputs: HashMap<String, Input>,
	outputs: HashMap<String, String>,
	pub loaded: bool,
}

impl Assembler {
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

	pub fn make_dna(&self, gc: &GeneticCircuit) -> (String, String, String, String) {
		let mut gates_plasmid = String::new();
		let mut promoter_colors = HashMap::new();

		let pre_gates = self.parts.get("gates_pre_backbone").unwrap();
		let mut gates_dna = pre_gates.seq.to_owned();

		gates_plasmid += &make_plasmid_part(
			&pre_gates.kind,
			0,
			gates_dna.len(),
			&pre_gates.name,
			"white",
		);

		for gene in &gc.genes {
			promoter_colors.insert(gene.promoter.to_owned(), gene.color.to_owned());
			for inp in &gene.inputs {
				let part = self.parts.get(inp).unwrap();
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

			let bio_gate = self.gates.get(&gene.name).unwrap();
			for part_name in &bio_gate.parts {
				let part = self.parts.get(part_name).unwrap();
				let start = gates_dna.len();
				let end = start + part.seq.len();

				gates_dna += &part.seq;
				gates_plasmid +=
					&make_plasmid_part(&part.kind, start, end, &part.name, &gene.color);
			}
		}

		let post_gates1 = self.parts.get("gates_post_backbone1").unwrap();
		let post_gates2 = self.parts.get("gates_post_backbone2").unwrap();

		let start1 = gates_dna.len();
		let end1 = start1 + post_gates1.seq.len();

		gates_dna += &post_gates1.seq;

		let start2 = gates_dna.len();
		let end2 = start2 + post_gates2.seq.len();

		gates_dna += &post_gates2.seq;

		gates_plasmid +=
			&make_plasmid_part(&post_gates1.kind, start1, end1, &post_gates1.name, "white");
		gates_plasmid +=
			&make_plasmid_part(&post_gates2.kind, start2, end2, &post_gates2.name, "white");

		let gates_title = make_plasmid_title("gates-plasmid", gates_dna.len());
		let gates_plasmid_dna: String = make_plasmid_dna(&gates_dna);
		let final_gates_plasmid = gates_title + &gates_plasmid + &gates_plasmid_dna;

		let mut output_plasmid = String::new();
		let pre_output = self.parts.get("output_pre_backbone").unwrap();
		let mut output_dna = pre_output.seq.to_owned();

		output_plasmid += &make_plasmid_part(
			&pre_output.kind,
			0,
			output_dna.len(),
			&pre_output.name,
			"white",
		);

		for inp in &gc.output.inputs {
			let part = self.parts.get(inp).unwrap();
			let start = output_dna.len();
			let end = start + part.seq.len();

			output_dna += &part.seq;

			output_plasmid += &make_plasmid_part(
				&part.kind,
				start,
				end,
				&part.name,
				promoter_colors.get(inp).unwrap_or(&"white".to_owned()),
			)
		}
		let out_part = self.outputs.get(&gc.output.name).unwrap();
		let start = output_dna.len();
		let end = start + out_part.len();

		output_plasmid +=
			&make_plasmid_part(&PartKind::Output, start, end, &gc.output.name, "white");

		output_dna += &out_part;

		let post_output = self.parts.get("output_post_backbone").unwrap();
		let start = output_dna.len();
		let end = start + post_output.seq.len();

		output_plasmid +=
			&make_plasmid_part(&post_output.kind, start, end, &post_output.name, "white");
		output_dna += &post_output.seq;

		let output_title = make_plasmid_title("output-plasmid", output_plasmid.len());

		let output_plasmid_dna = make_plasmid_dna(&output_dna);
		let final_output_plasmid = output_title + &output_plasmid + &output_plasmid_dna;

		(
			gates_dna,
			output_dna,
			final_gates_plasmid,
			final_output_plasmid,
		)
	}

	fn walk_simulate(
		&self,
		curr_gate: &str,
		lc: &LogicCircuit,
		assigned_gates: &HashMap<String, String>,
		input_map: &HashMap<String, (bool, f64)>,
	) -> (bool, f64) {
		if input_map.contains_key(curr_gate) {
			return input_map.get(curr_gate).cloned().unwrap();
		}
		let gate = lc.gates.get(curr_gate).unwrap();
		let mut inputs = Vec::new();
		let mut rpus = Vec::new();
		for inp in &gate.inputs {
			let (bit, rpu) = self.walk_simulate(inp, lc, assigned_gates, input_map);
			inputs.push(bit);
			rpus.push(rpu);
		}

		let (out, x) = gate_logic(inputs, rpus, &gate.kind);
		let assigned_gate = assigned_gates.get(curr_gate).unwrap();
		let bio_gate = self.gates.get(assigned_gate).unwrap();
		let y = transfer(x, &bio_gate.params);

		(out, y)
	}

	pub fn simulate(
		&self,
		lc: &LogicCircuit,
		assigned_gates: &HashMap<String, String>,
	) -> Vec<(String, bool, f64)> {
		let mut results = Vec::new();
		let len = lc.inputs.len();
		let permutations = 2u32.pow(len as u32);
		for num in 0..permutations {
			let mut input_map: HashMap<String, (bool, f64)> = HashMap::new();
			let mut signs: Vec<&str> = Vec::new();
			for i in 0..len {
				let name = lc.inputs[i].name.to_owned();
				let input = self.inputs.get(&name).unwrap();
				if num & 2u32.pow(i as u32) > 0 {
					signs.push("+");
					input_map.insert(name, (true, input.rpu_on));
				} else {
					signs.push("-");
					input_map.insert(name, (false, input.rpu_off));
				}
			}
			let (out, rpu) = self.walk_simulate(&lc.output.name, lc, assigned_gates, &input_map);
			results.push((signs.join("/"), out, rpu));
		}
		results
	}

	fn walk_assemble(
		&self,
		curr_gate: &str,
		lc: &LogicCircuit,
		assigned_gates: &HashMap<String, String>,
		genetic_circuit: &mut GeneticCircuit,
		id: &mut u8,
	) -> String {
		if self.inputs.contains_key(curr_gate) {
			return self.inputs.get(curr_gate).unwrap().promoter.to_owned();
		}

		let gate = lc.gates.get(curr_gate).unwrap();
		let mut inputs = Vec::new();
		for inp in &gate.inputs {
			let pro = self.walk_assemble(inp, lc, assigned_gates, genetic_circuit, id);
			inputs.push(pro);
		}
		*id += 1;
		let assigned_gate = assigned_gates.get(curr_gate).unwrap();
		let bio_gate = self.gates.get(assigned_gate).unwrap();

		let color_hex = Hsl::from(
			map(*id as f32, 0.0, assigned_gates.len() as f32, 0.0, 355.0),
			100.0,
			50.0,
		)
		.to_rgb()
		.to_css_hex_string();
		genetic_circuit.genes.push(Gene {
			inputs: inputs,
			color: color_hex,
			promoter: bio_gate.promoter.to_owned(),
			name: assigned_gate.to_owned(),
		});

		bio_gate.promoter.to_owned()
	}

	pub fn assemble(
		&self,
		lc: &LogicCircuit,
		assigned_gates: &HashMap<String, String>,
	) -> GeneticCircuit {
		let mut genetic_circuit = GeneticCircuit {
			output: OutputGene {
				name: lc.output.name.to_owned(),
				inputs: Vec::new(),
			},
			inputs: lc.inputs.iter().map(|x| x.name.to_owned()).collect(),
			genes: Vec::new(),
		};

		let mut id = 0;
		let promoter = self.walk_assemble(
			&lc.output.name,
			lc,
			assigned_gates,
			&mut genetic_circuit,
			&mut id,
		);
		genetic_circuit.output.inputs.push(promoter);
		genetic_circuit
	}

	pub fn assign(&mut self, lc: &LogicCircuit) -> Result<(HashMap<String, String>, f64), Error> {
		if !self.outputs.contains_key(&lc.output.name) {
			return Err(assign_err(
				format!("Output '{}' not found.", lc.output.name),
				(lc.output.pos, lc.output.name.len()),
			));
		}

		let mut inputs = HashMap::new();
		for input in &lc.inputs {
			if !self.inputs.contains_key(&input.name) {
				return Err(assign_err(
					format!("Input '{}' not found.", input.name),
					(input.pos, input.name.len()),
				));
			}
			let inp = self.inputs.get(&input.name).cloned().unwrap();
			inputs.insert(input.name.to_owned(), inp);
		}

		let (assigned_gates, min) = self.try_walk(lc, 1.0)?;

		Ok((assigned_gates, min))
	}

	fn try_walk(
		&self,
		lc: &LogicCircuit,
		min: f64,
	) -> Result<(HashMap<String, String>, f64), Error> {
		let mut assigned_gates = HashMap::new();

		let initial_bl: HashSet<String> = lc.inputs.iter().map(|x| x.name.to_owned()).collect();
		let (_, _, new_min, _, _) = self.walk_back(
			lc.output.name.to_owned(),
			&lc,
			&initial_bl,
			&mut HashMap::new(),
			&mut assigned_gates,
			min,
		)?;

		let chres = self.try_walk(lc, new_min);
		if chres.is_ok() {
			Ok(chres.unwrap())
		} else {
			Ok((assigned_gates, new_min))
		}
	}

	fn walk_back(
		&self,
		curr_gate: String,
		lc: &LogicCircuit,
		ext_bl: &HashSet<String>,
		gate_bl: &mut HashMap<String, HashSet<String>>,
		assigned_gates: &mut HashMap<String, String>,
		min: f64,
	) -> Result<(f64, f64, f64, String, HashSet<String>), Error> {
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
				HashSet::new(),
			));
		}

		let gate = lc.gates.get(&curr_gate).unwrap();
		let mut res_bl = ext_bl.clone();
		let (mut new_on, mut new_off, mut new_min, mut names): (f64, f64, f64, Vec<String>) =
			(0.0, f64::MAX, f64::MAX, vec![]);
		for inp in &gate.inputs {
			let (con, coff, cmin, name, bl) =
				self.walk_back(inp.to_owned(), lc, &res_bl, gate_bl, assigned_gates, min)?;
			res_bl.extend(bl);
			names.push(name);
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
			return self.walk_back(curr_gate, lc, ext_bl, gate_bl, assigned_gates, min);
		}

		res_bl.insert(get_group(name.to_owned()));
		assigned_gates.insert(curr_gate, name.to_owned());

		Ok((max_off, max_on, new_min.min(max_rpu), name, res_bl))
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
