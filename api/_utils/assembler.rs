use crate::_utils::{builder, helpers, kd_tree};
use builder::{LogicCircuit, Testbench};
use colors_transform::{Color, Hsl};
use fs_extra::file::read_to_string;
use helpers::{
	assign_err, damp, damp_params, get_group, lerp, make_plasmid_dna, make_plasmid_part,
	make_plasmid_title, map, transfer, Error,
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
	params: Params,
}

#[derive(Serialize, Debug)]
struct OutputGene {
	name: String,
	inputs: Vec<String>,
}

#[derive(Serialize, Debug)]
pub struct GeneticCircuit {
	output: OutputGene,
	inputs: Vec<Input>,
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

#[derive(Clone, Debug)]
pub struct AssignedGate {
	on: f64,
	off: f64,
	min: f64,
	name: String,
	bl: HashSet<String>,
}

#[derive(Deserialize)]
struct Rules {
	gates: HashMap<String, u32>,
	promoters: HashMap<String, u32>,
}

pub struct Assembler {
	kd: KdTree,
	gates: HashMap<String, BioGate>,
	parts: HashMap<String, Part>,
	inputs: HashMap<String, Input>,
	outputs: HashMap<String, String>,
	rules: Rules,
	roadblock: HashSet<String>,
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
			rules: Rules {
				gates: HashMap::new(),
				promoters: HashMap::new(),
			},
			roadblock: HashSet::new(),
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
		let rules_path = format!("{}/static/rules.json", dir.display());
		let roadblock_path = format!("{}/static/roadblock.json", dir.display());

		let trees_f = read_to_string(tree_path).unwrap();
		let gates_f = read_to_string(gates_path).unwrap();
		let parts_f = read_to_string(parts_path).unwrap();
		let inputs_f = read_to_string(inputs_path).unwrap();
		let outputs_f = read_to_string(outputs_path).unwrap();
		let rules_f = read_to_string(rules_path).unwrap();
		let roadblock_f = read_to_string(roadblock_path).unwrap();

		let tree: KdTree = from_str(&trees_f).unwrap();
		let gates: HashMap<String, BioGate> = from_str(&gates_f).unwrap();
		let parts: HashMap<String, Part> = from_str(&parts_f).unwrap();
		let inputs: HashMap<String, Input> = from_str(&inputs_f).unwrap();
		let outputs: HashMap<String, String> = from_str(&outputs_f).unwrap();
		let rules: Rules = from_str(&rules_f).unwrap();
		let roadblock: HashSet<String> = from_str(&roadblock_f).unwrap();

		self.kd = tree;
		self.gates = gates;
		self.parts = parts;
		self.inputs = inputs;
		self.outputs = outputs;
		self.rules = rules;
		self.roadblock = roadblock;
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

	pub fn simulate(
		&self,
		testbench: &Testbench,
		gc: &GeneticCircuit,
	) -> HashMap<String, Vec<f64>> {
		let delay = 50;
		let growth_rate = 0.1;
		let params = damp_params(10.0, 0.01);

		let mut input_targets: HashMap<String, f64> = HashMap::new();
		let mut history: HashMap<String, Vec<f64>> = HashMap::new();
		let mut lerp_states: HashMap<String, f64> = HashMap::new();
		let mut damp_states: HashMap<String, (f64, f64)> = HashMap::new();
		let mut final_hist: HashMap<String, Vec<f64>> = HashMap::new();

		for inp in &gc.inputs {
			input_targets.insert(inp.promoter.to_owned(), inp.rpu_off);
			history.insert(inp.promoter.to_owned(), vec![inp.rpu_off]);
			lerp_states.insert(inp.promoter.to_owned(), inp.rpu_off);
			damp_states.insert(inp.promoter.to_owned(), (inp.rpu_off, 0.0));
			final_hist.insert(inp.promoter.to_owned(), vec![inp.rpu_off]);
		}
		for gene in &gc.genes {
			let mut max = 0.0f64;
			for inp in &gene.inputs {
				let inp_state = input_targets.get(inp).unwrap();
				max = max.max(*inp_state);
			}
			let rpu = transfer(max, &gene.params);
			input_targets.insert(gene.promoter.to_owned(), rpu);
			history.insert(gene.promoter.to_owned(), vec![rpu]);
			lerp_states.insert(gene.promoter.to_owned(), rpu);
			damp_states.insert(gene.promoter.to_owned(), (rpu, 0.0));
			final_hist.insert(gene.promoter.to_owned(), vec![rpu]);
		}
		for i in 0..1000 {
			if testbench.at_breakpoints.contains_key(&i) {
				let bp = testbench.at_breakpoints.get(&i).unwrap();
				for (name, val) in bp {
					let inp = self.inputs.get(name).unwrap();
					input_targets.insert(
						inp.promoter.to_owned(),
						if *val == true {
							inp.rpu_on
						} else {
							inp.rpu_off
						},
					);
				}
			}

			for inp in &gc.inputs {
				let target = input_targets.get(&inp.promoter).unwrap();
				let input_history = history.get_mut(&inp.promoter).unwrap();
				input_history.push(*target);

				let p = lerp_states.get(&inp.promoter).unwrap();
				let (q, vel) = damp_states.get(&inp.promoter).unwrap();
				let new_p = lerp(*p, *target, growth_rate);
				let (new_q, new_vel) = damp(*q, *vel, *p, &params);
				lerp_states.insert(inp.promoter.to_owned(), new_p);
				damp_states.insert(inp.promoter.to_owned(), (new_q, new_vel));

				let final_inp_hist = final_hist.get_mut(&inp.promoter).unwrap();
				final_inp_hist.push(new_q);
			}

			for gene in &gc.genes {
				let mut target = 0.0f64;
				for inp in &gene.inputs {
					let input_history = history.get(inp).unwrap();
					let rpu = input_history
						.get((input_history.len() as i32 - delay).max(0) as usize)
						.unwrap();
					target = target.max(*rpu);
				}

				let gene_history = history.get_mut(&gene.promoter).unwrap();
				gene_history.push(target);

				let p = lerp_states.get(&gene.promoter).unwrap();
				let (q, vel) = damp_states.get(&gene.promoter).unwrap();
				let new_p = lerp(*p, target, growth_rate);
				let (new_q, new_vel) = damp(*q, *vel, *p, &params);
				lerp_states.insert(gene.promoter.to_owned(), new_p);
				damp_states.insert(gene.promoter.to_owned(), (new_q, new_vel));

				let final_inp_hist = final_hist.get_mut(&gene.promoter).unwrap();
				final_inp_hist.push(new_q);
			}
		}

		final_hist
	}

	fn walk_assemble(
		&self,
		curr_gate: &str,
		lc: &LogicCircuit,
		assigned_gates: &HashMap<String, String>,
		added_gates: &mut HashSet<String>,
		genetic_circuit: &mut GeneticCircuit,
		id: &mut u8,
	) -> String {
		if self.inputs.contains_key(curr_gate) {
			return self.inputs.get(curr_gate).unwrap().promoter.to_owned();
		}

		if added_gates.contains(curr_gate) {
			let assigned_gate = assigned_gates.get(curr_gate).unwrap();
			let bio_gate = self.gates.get(assigned_gate).unwrap();
			return bio_gate.promoter.to_owned();
		}

		let gate = lc.gates.get(curr_gate).unwrap();
		let mut inputs = Vec::new();
		for inp in &gate.inputs {
			let pro = self.walk_assemble(inp, lc, assigned_gates, added_gates, genetic_circuit, id);
			inputs.push(pro);
		}
		*id += 1;
		let assigned_gate = assigned_gates.get(curr_gate).unwrap();
		let bio_gate = self.gates.get(assigned_gate).unwrap();

		let color_hex = Hsl::from(
			map(*id as f64, 0.0, assigned_gates.len() as f64, 0.0, 355.0) as f32,
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
			params: bio_gate.params.clone(),
		});
		added_gates.insert(curr_gate.to_owned());

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
			inputs: lc
				.inputs
				.iter()
				.map(|x| self.inputs.get(&x.name).cloned().unwrap())
				.collect(),
			genes: Vec::new(),
		};

		let mut id = 0;
		let mut added_gates = HashSet::new();
		let promoter = self.walk_assemble(
			&lc.output.name,
			lc,
			assigned_gates,
			&mut added_gates,
			&mut genetic_circuit,
			&mut id,
		);
		genetic_circuit.output.inputs.push(promoter);
		genetic_circuit
	}

	pub fn apply_rules(&self, gc: &mut GeneticCircuit) {
		gc.genes.sort_by(|a, b| {
			let a_index = self.rules.gates.get(&get_group(&a.name)).unwrap();
			let b_index = self.rules.gates.get(&get_group(&b.name)).unwrap();
			a_index.cmp(b_index)
		});

		for gene in &mut gc.genes {
			gene.inputs.sort_by(|a, b| {
				let a_index = self.rules.promoters.get(a).unwrap();
				let b_index = self.rules.promoters.get(b).unwrap();
				a_index.cmp(b_index)
			});
		}
	}

	pub fn assign(&mut self, lc: &LogicCircuit) -> Result<(HashMap<String, String>, f64), Error> {
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
		let ass_gate = self.walk_back(
			lc.output.name.to_owned(),
			&lc,
			&initial_bl,
			&mut HashMap::new(),
			&mut assigned_gates,
			min,
		)?;

		let chres = self.try_walk(lc, ass_gate.min);
		if chres.is_ok() {
			Ok(chres.unwrap())
		} else {
			Ok((assigned_gates, ass_gate.min))
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
	) -> Result<AssignedGate, Error> {
		if !gate_bl.contains_key(&curr_gate) {
			gate_bl.insert(curr_gate.to_owned(), HashSet::new());
		}

		if self.inputs.contains_key(&curr_gate) {
			let in_rpus = self.inputs.get(&curr_gate).unwrap();
			return Ok(AssignedGate {
				on: in_rpus.rpu_off,
				off: in_rpus.rpu_on,
				min: in_rpus.rpu_on / in_rpus.rpu_off,
				name: "0_0".to_string(),
				bl: HashSet::new(),
			});
		}

		let gate = lc.gates.get(&curr_gate).unwrap();
		let mut res_bl = ext_bl.clone();
		let (mut new_on, mut new_off, mut new_min, mut names): (f64, f64, f64, Vec<String>) =
			(0.0, f64::MAX, f64::MAX, vec![]);
		for inp in &gate.inputs {
			let ass_gate =
				self.walk_back(inp.to_owned(), lc, &res_bl, gate_bl, assigned_gates, min)?;
			res_bl.extend(ass_gate.bl);
			names.push(ass_gate.name);
			new_on = ass_gate.on.max(new_on);
			new_off = ass_gate.off.min(new_off);
			new_min = ass_gate.min.min(new_min);
		}

		let mut gbl = gate_bl.get(&curr_gate).cloned().unwrap();
		gbl.extend(res_bl.clone());
		let node = self.kd.search(vec![new_on, new_off, 1000.0], &gbl);

		let (name, max_on, max_off, max_rpu) = self.get_on_off(new_on, new_off, node);
		let num_r = self.get_num_roadblocks(&names);
		if max_rpu <= min || num_r > 1 {
			if self.inputs.contains_key(&gate.inputs[0]) {
				return Err(assign_err(
					"Failed to find optimal genetic circuit!".to_owned(),
					(0, 0),
				));
			}
			let parentgbl = gate_bl.get_mut(&gate.inputs[0].to_owned()).unwrap();
			parentgbl.insert(names[0].to_owned());
			return self.walk_back(curr_gate, lc, ext_bl, gate_bl, assigned_gates, min);
		}

		res_bl.insert(get_group(&name));
		let ng = AssignedGate {
			name: name.to_owned(),
			on: max_off,
			off: max_on,
			min: new_min.min(max_rpu),
			bl: res_bl,
		};
		assigned_gates.insert(curr_gate, name);

		Ok(ng)
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

	fn get_num_roadblocks(&self, names: &Vec<String>) -> u8 {
		let mut num = 0;
		for name in names {
			if name == "0_0" {
				continue;
			}
			let gate = self.gates.get(name).unwrap();
			if self.roadblock.contains(&gate.promoter) {
				num += 1;
			}
		}
		num
	}
}
