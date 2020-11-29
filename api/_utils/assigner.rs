use crate::_utils::{builder, data, helpers};
use builder::LogicCircuit;
use data::{genes_len, get_gene_at, get_input, has_input};
use helpers::{err, exp, gen_matrix, get_group, inv_out_error, lrate, out_error, transfer, Error};
use rand::distributions::{Distribution, Uniform};
use rand::prelude::ThreadRng;
use std::collections::{HashMap, HashSet};

pub struct Assigner {
	layers: Vec<Vec<Vec<f64>>>,
}

impl Assigner {
	pub fn init(lc: &LogicCircuit) -> Result<Self, Error> {
		for input in &lc.inputs {
			exp(has_input(&input.name), "Input not found: ", input)?;
		}
		if lc.gates.len() > genes_len() {
			return Err(err(format!("Number of gates exceeds maximum: '{}'", genes_len()), (0, 0)));
		}
		let mut layers = Vec::new();
		let genes_len = genes_len();

		for gate in lc.gates.iter().rev() {
			let num_nodes = if lc.output.name == gate.name { 1 } else { genes_len };
			let nodes = gen_matrix(genes_len, num_nodes);
			layers.push(nodes);
		}
		Ok(Self { layers })
	}

	pub fn fit(&mut self, lc: &LogicCircuit) -> Result<(Vec<usize>, f64), Error> {
		let mut rng = rand::thread_rng();
		let uni = Uniform::new_inclusive(0.0f64, 1.0);

		let len = 6000;
		let mut best_score = 0.0;
		let mut best_org_score = 0.0;
		let mut best_sel = Vec::new();
		for i in 0..len {
			let lr = lrate(i as f64, len as f64);
			let sel_genes = self.walk(&lc, &mut rng, &uni);
			let (off, on, diff) = self.test(&sel_genes, &lc);

			let diff_err = inv_out_error(diff);
			let org_score = on / off;
			let diff_core = diff_err * org_score;
			if diff_core > best_score {
				best_org_score = org_score;
				best_score = diff_core;
				best_sel = sel_genes.clone();
			}
			let out = out_error(diff_core);
			self.update_weights(lr, out, sel_genes);
		}
		Ok((best_sel, best_org_score))
	}

	pub fn walk(&self, lc: &LogicCircuit, rng: &mut ThreadRng, uni: &Uniform<f64>) -> Vec<usize> {
		let mut bl: HashSet<String> = lc.inputs.iter().map(|x| x.name.to_owned()).collect();
		let mut sel = 0;
		let mut selected = Vec::new();
		for layer in self.layers.iter() {
			let weights = &layer[sel];
			let ch = uni.sample(rng);
			sel = self.choose_node(ch, weights, &bl);
			selected.push(sel);
			let node = get_gene_at(sel);
			bl.insert(get_group(&node.name));
		}
		selected
	}

	pub fn test(&self, selected_gates: &Vec<usize>, lc: &LogicCircuit) -> (f64, f64, f64) {
		let mut cached = HashMap::new();
		for arg in &lc.inputs {
			let sensor = get_input(&arg.name);
			cached.insert(arg.name.to_owned(), (sensor.rpu_off, sensor.rpu_on, 0.0));
		}
		for (i, selected) in selected_gates.iter().rev().enumerate() {
			let gate = lc.gates.get(i).unwrap();

			let (off, on, diff): (f64, f64, f64);
			if gate.inputs.len() == 1 {
				let (coff, con, cdiff) = cached[&gate.inputs[0]];
				off = coff;
				on = con;
				diff = cdiff;
			} else {
				let (coff0, con0, diff0) = cached[&gate.inputs[0]];
				let (coff1, con1, diff1) = cached[&gate.inputs[1]];
				on = con0.min(con1);
				off = coff0.max(coff1);
				diff = (con0 - con1).abs() + (coff0 - coff1).abs() + diff0 + diff1;
			}

			let gene = get_gene_at(*selected);
			let new_off = transfer(on, &gene.params) / gene.params.decay;
			let new_on = transfer(off, &gene.params) / gene.params.decay;
			cached.insert(gate.name.to_owned(), (new_off, new_on, diff));
		}

		cached[&lc.output.name]
	}

	pub fn choose_node(&self, ch: f64, weights: &Vec<f64>, bl: &HashSet<String>) -> usize {
		let mut acc = 0.0;
		let mut sum: f64 = 0.0;
		for (i, w) in weights.iter().enumerate() {
			let node = &get_gene_at(i);
			if bl.contains(&get_group(&node.name)) {
				continue;
			}
			sum += w;
		}
		for (i, w) in weights.iter().enumerate() {
			let node = &get_gene_at(i);
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

	pub fn update_weights(&mut self, lr: f64, pr: f64, selected: Vec<usize>) {
		let mut prev_node_id = 0;
		for (layer, curr_node_id) in self.layers.iter_mut().zip(selected.iter()) {
			let weights = layer.get_mut(prev_node_id).unwrap();
			let target = pr - weights[*curr_node_id];
			let change = lr * target;
			weights[*curr_node_id] += change;
			prev_node_id = *curr_node_id;
		}
	}
}
