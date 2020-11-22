use crate::_utils::{builder, data, helpers};
use builder::{Gate, LogicCircuit};
use colors_transform::{Color, Hsl};
use data::{
	gates_len, get_gate_at, get_input, get_rules, has_input, has_output, Gene, GeneticCircuit,
	OutputGene,
};
use helpers::{
	assign_err, gen_matrix, get_group, inv_out_error, lrate, map, out_error, transfer, Error,
};
use rand::distributions::{Distribution, Uniform};
use rand::prelude::ThreadRng;
use std::collections::{HashMap, HashSet};

pub struct Assembler {
	lc: LogicCircuit,
}

impl Assembler {
	pub fn new(lc: LogicCircuit) -> Self {
		Self { lc }
	}

	fn walk_assemble(
		&self,
		curr_gate: &str,
		assigned_gates: &HashMap<String, usize>,
		added_gates: &mut HashSet<String>,
		genetic_circuit: &mut GeneticCircuit,
		id: &mut u8,
	) -> String {
		if has_input(curr_gate) {
			return get_input(curr_gate).promoter.to_owned();
		}

		if added_gates.contains(curr_gate) {
			let assigned_gate = assigned_gates.get(curr_gate).unwrap();
			let bio_gate = get_gate_at(*assigned_gate);
			return bio_gate.promoter.to_owned();
		}

		added_gates.insert(curr_gate.to_owned());
		let gate = self.lc.gates.get(curr_gate).unwrap();
		let mut inputs = Vec::new();
		for inp in &gate.inputs {
			let pro = self.walk_assemble(inp, assigned_gates, added_gates, genetic_circuit, id);
			inputs.push(pro);
		}
		let assigned_gate = assigned_gates.get(curr_gate).unwrap();
		let bio_gate = get_gate_at(*assigned_gate);
		*id += 1;

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
			name: bio_gate.name.to_owned(),
		});

		bio_gate.promoter.to_owned()
	}

	pub fn assemble(&self, assigned_gates: &HashMap<String, usize>) -> GeneticCircuit {
		let mut genetic_circuit = GeneticCircuit {
			output: OutputGene {
				name: self.lc.output.name.to_owned(),
				inputs: Vec::new(),
			},
			inputs: self
				.lc
				.inputs
				.iter()
				.map(|x| get_input(&x.name).clone())
				.collect(),
			genes: Vec::new(),
		};

		let mut id = 0;
		let mut added_gates = HashSet::new();
		let promoter = self.walk_assemble(
			&self.lc.output.name,
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
			let a_index = get_rules().gates.get(&get_group(&a.name)).unwrap();
			let b_index = get_rules().gates.get(&get_group(&b.name)).unwrap();
			a_index.cmp(b_index)
		});

		for gene in &mut gc.genes {
			gene.inputs.sort_by(|a, b| {
				let a_index = get_rules().promoters.get(a).unwrap();
				let b_index = get_rules().promoters.get(b).unwrap();
				a_index.cmp(b_index)
			});
		}
	}

	pub fn assign(&mut self) -> Result<(HashMap<String, usize>, f64), Error> {
		if !has_output(&self.lc.output.name) {
			return Err(assign_err(
				format!("Output '{}' not found.", self.lc.output.name),
				(self.lc.output.pos, self.lc.output.name.len()),
			));
		}

		for input in &self.lc.inputs {
			if !has_input(&input.name) {
				return Err(assign_err(
					format!("Input '{}' not found.", input.name),
					(input.pos, input.name.len()),
				));
			}
		}

		let (assigned_gates, score) = self.search()?;

		Ok((assigned_gates, score))
	}

	fn search(&self) -> Result<(HashMap<String, usize>, f64), Error> {
		if self.lc.gates.len() > gates_len() {
			return Err(assign_err(
				format!("Number of gates exceeds maximum: '{}'", gates_len()),
				(0, 0),
			));
		}
		let mut layers = HashMap::new();
		self.init(
			&self.lc.output.name,
			&self.lc.gates,
			&mut layers,
			1,
			gates_len(),
		);

		let mut rng = rand::thread_rng();
		let uni = Uniform::new_inclusive(0.0f64, 1.0);

		let len = 6000;
		let mut best_score = 0.0;
		let mut best_ass = HashMap::new();
		for i in 0..len {
			let lr = lrate(i as f64, len as f64);
			let mut selected_gates = HashMap::new();
			let mut bl = self
				.lc
				.inputs
				.iter()
				.map(|arg| arg.name.to_owned())
				.collect();
			self.walk(
				&self.lc.output.name,
				0,
				&self.lc.gates,
				&mut selected_gates,
				&layers,
				&mut bl,
				uni,
				&mut rng,
			);

			let mut cached = HashMap::new();
			self.test(
				&self.lc.output.name,
				&self.lc.gates,
				&selected_gates,
				&mut HashSet::new(),
				&mut cached,
			);
			let (off, on, diff) = self.test(
				&self.lc.output.name,
				&self.lc.gates,
				&selected_gates,
				&mut HashSet::new(),
				&mut cached,
			);

			let diff_err = inv_out_error(diff);

			let score = diff_err * (on / off);
			if score > best_score {
				best_score = score;
				best_ass = selected_gates.clone();
			}
			let out = out_error(score);
			self.update_weights(
				lr,
				out * diff_err,
				&self.lc.output.name,
				0,
				&selected_gates,
				&mut layers,
				&self.lc.gates,
				&mut HashSet::new(),
			);
		}
		Ok((best_ass, best_score))
	}

	pub fn init(
		&self,
		name: &str,
		gates: &HashMap<String, Gate>,
		layers: &mut HashMap<String, Vec<Vec<f64>>>,
		num_inputs: usize,
		num_nodes: usize,
	) {
		if layers.contains_key(name) || !gates.contains_key(name) {
			return;
		}
		layers.insert(name.to_owned(), gen_matrix(num_inputs, num_nodes));
		let gate = gates.get(name).unwrap();
		for inp in &gate.inputs {
			self.init(inp, gates, layers, num_nodes, num_nodes);
		}
	}

	pub fn walk(
		&self,
		name: &str,
		selected: usize,
		gates: &HashMap<String, Gate>,
		selected_gates: &mut HashMap<String, usize>,
		layers: &HashMap<String, Vec<Vec<f64>>>,
		bl: &mut HashSet<String>,
		uni: Uniform<f64>,
		rng: &mut ThreadRng,
	) {
		if has_input(name) || selected_gates.contains_key(name) {
			return;
		}
		let gate = gates.get(name).unwrap();
		let layer = layers.get(name).unwrap();
		let ch = uni.sample(rng);
		let sel = self.choose_node(ch, &layer[selected], bl);
		let node = &get_gate_at(sel);
		selected_gates.insert(name.to_owned(), sel);
		bl.insert(get_group(&node.name));
		for inp in &gate.inputs {
			self.walk(&inp, sel, gates, selected_gates, layers, bl, uni, rng);
		}
	}

	pub fn test(
		&self,
		name: &str,
		gates: &HashMap<String, Gate>,
		selected_gates: &HashMap<String, usize>,
		visited: &mut HashSet<String>,
		cached: &mut HashMap<String, (f64, f64, f64)>,
	) -> (f64, f64, f64) {
		if visited.contains(name) {
			let cach = cached.get(name).unwrap_or(&(0.0, 0.0, 0.0));
			return *cach;
		}
		if has_input(name) {
			let inp = get_input(name);
			return (inp.rpu_off, inp.rpu_on, 0.0);
		}
		visited.insert(name.to_owned());
		let gate = gates.get(name).unwrap();
		let (off, on, diff): (f64, f64, f64);
		if gate.inputs.len() == 1 {
			let (coff, con, cdiff) =
				self.test(&gate.inputs[0], gates, selected_gates, visited, cached);
			off = coff;
			on = con;
			diff = cdiff;
		} else {
			let (coff0, con0, diff0) =
				self.test(&gate.inputs[0], gates, selected_gates, visited, cached);
			let (coff1, con1, diff1) =
				self.test(&gate.inputs[1], gates, selected_gates, visited, cached);
			on = con0 + con1;
			off = coff0 + coff1;
			diff = (con0 - con1).abs() + (coff0 - coff1).abs() + diff0 + diff1;
		}

		let selected = selected_gates.get(name).unwrap();
		let gate = get_gate_at(*selected);
		let new_off = transfer(on, &gate.params) / gate.params.decay;
		let new_on = transfer(off, &gate.params) / gate.params.decay;
		cached.insert(name.to_owned(), (new_off, new_on, diff));

		(new_off, new_on, diff)
	}

	pub fn choose_node(&self, ch: f64, weights: &Vec<f64>, bl: &HashSet<String>) -> usize {
		let mut sum = 0.0;
		for (i, w) in weights.iter().enumerate() {
			let node = &get_gate_at(i);
			if bl.contains(&get_group(&node.name)) {
				continue;
			}
			sum += w;
		}
		let mut acc = 0.0;
		for (i, w) in weights.iter().enumerate() {
			let node = &get_gate_at(i);
			if bl.contains(&get_group(&node.name)) {
				continue;
			}
			acc += w / sum;
			if ch <= acc {
				return i;
			}
		}
		weights.len() - 1
	}

	pub fn update_weights(
		&self,
		lr: f64,
		pr: f64,
		name: &str,
		prev: usize,
		selected_gates: &HashMap<String, usize>,
		layers: &mut HashMap<String, Vec<Vec<f64>>>,
		gates: &HashMap<String, Gate>,
		visited: &mut HashSet<String>,
	) {
		if visited.contains(name) {
			return;
		}
		if has_input(name) {
			return;
		}
		visited.insert(name.to_owned());
		let layer = layers.get_mut(name).unwrap();
		let id = selected_gates.get(name).unwrap();
		let weights = layer.get_mut(prev).unwrap();
		let target = pr - weights[*id];
		weights[*id] += lr * target;

		let sum: f64 = weights.iter().sum();
		for w in weights.iter_mut() {
			*w = *w / sum;
		}
		let gate = gates.get(name).unwrap();
		for inp in &gate.inputs {
			self.update_weights(lr, pr, inp, *id, selected_gates, layers, gates, visited);
		}
	}
}
