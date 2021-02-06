use crate::_utils::{builder, data, helpers};
use builder::{Device, GateKind, LogicCircuit};
use data::get_data;
use helpers::{inv_out_error, lrate, out_error, transfer, Error};
use rand::distributions::{Distribution, Uniform};
use rand::prelude::ThreadRng;
use std::collections::{HashMap, HashSet};

pub struct Layer {
	nodes: Vec<f64>,
	rng: ThreadRng,
	uni: Uniform<f64>,
	device: Device,
}

impl Layer {
	pub fn init(device: Device) -> Self {
		let data = get_data();
		let len = match device {
			Device::Gate(_) => data.genes_len(),
		};
		let mut rng = rand::thread_rng();
		let uni = Uniform::new_inclusive(0.0f64, 1.0);
		let nodes = vec![uni.sample(&mut rng); len];
		Self {
			nodes,
			rng,
			uni,
			device,
		}
	}

	pub fn choose_node(&mut self, bl: &mut HashSet<String>) -> usize {
		let ch = self.uni.sample(&mut self.rng);
		let sel = self.get_node_from_prob(ch, bl);
		self.insert_bl(sel, bl);
		sel
	}

	pub fn update_weight(&mut self, lr: f64, pr: f64, node_id: usize) {
		let weight = self.nodes.get_mut(node_id).unwrap();
		let target = pr - *weight;
		let change = lr * target;
		*weight += change;
	}

	pub fn insert_bl(&self, i: usize, bl: &mut HashSet<String>) {
		let data = get_data();
		match self.device {
			Device::Gate(_) => {
				let gene = data.get_gene_at(i);
				bl.insert(gene.group());
			}
		}
	}

	pub fn in_bl(&self, i: usize, bl: &HashSet<String>) -> bool {
		let data = get_data();
		match self.device {
			Device::Gate(_) => {
				let gene = data.get_gene_at(i);
				if bl.contains(&gene.group()) {
					return true;
				}
			}
		}
		false
	}

	pub fn get_node_from_prob(&self, ch: f64, bl: &HashSet<String>) -> usize {
		let mut acc = 0.0;
		let mut sum: f64 = 0.0;
		for (i, w) in self.nodes.iter().enumerate() {
			if self.in_bl(i, &bl) {
				continue;
			}
			sum += w;
		}
		for (i, w) in self.nodes.iter().enumerate() {
			if self.in_bl(i, &bl) {
				continue;
			}
			acc += w / sum;
			if ch <= acc {
				return i;
			}
		}
		self.nodes.len() - 1
	}
}

pub struct GeneNetwork {
	layers: Vec<Layer>,
	lc: LogicCircuit,
}

impl GeneNetwork {
	pub fn init(lc: LogicCircuit) -> Result<Self, Error> {
		let data = get_data();
		for input in &lc.inputs {
			if !data.has_input(input) {
				return Err(Error::NotFound(0, 0));
			}
		}
		if lc.devices.len() > data.genes_len() {
			return Err(Error::NotEnoughGates);
		}
		let mut layers = Vec::new();
		for device in lc.devices.iter().rev() {
			let layer = Layer::init(device.clone());
			layers.push(layer);
		}
		Ok(Self { layers, lc })
	}

	pub fn fit(&mut self) -> Result<(Vec<usize>, f64), Error> {
		let len = 6000;
		let mut best_score = 0.0;
		let mut best_org_score = 0.0;
		let mut best_sel = Vec::new();
		for i in 0..len {
			let lr = lrate(i as f64, len as f64);
			let sel_genes = self.walk();
			let (off, on, diff) = self.test(&sel_genes);

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

	pub fn walk(&mut self) -> Vec<usize> {
		let mut bl: HashSet<String> = self.lc.inputs.iter().map(|x| x.to_string()).collect();
		let mut selected = Vec::new();
		for layer in &mut self.layers {
			let sel = layer.choose_node(&mut bl);
			selected.push(sel);
		}
		selected
	}

	pub fn test(&self, selected_gates: &Vec<usize>) -> (f64, f64, f64) {
		let data = get_data();
		let mut cached = HashMap::new();
		for arg in &self.lc.inputs {
			let sensor = data.get_input(&arg);
			cached.insert(arg.to_owned(), (sensor.rpu_off, sensor.rpu_on, 0.0));
		}
		for (i, selected) in selected_gates.iter().rev().enumerate() {
			let device = self.lc.devices.get(i).unwrap();
			match device {
				Device::Gate(gate) => {
					let (off, on, diff) = if gate.kind == GateKind::Not {
						let (coff, con, cdiff) = cached[&gate.inputs[0]];
						(coff, con, cdiff)
					} else {
						let (coff0, con0, diff0) = cached[&gate.inputs[0]];
						let (coff1, con1, diff1) = cached[&gate.inputs[1]];
						let diff = (con0 - con1).abs() + (coff0 - coff1).abs() + diff0 + diff1;
						(coff0 + coff1, con0.min(con1), diff)
					};

					let gene = data.get_gene_at(*selected);
					let new_off = transfer(on, &gene.params) / gene.params.decay;
					let new_on = transfer(off, &gene.params) / gene.params.decay;
					cached.insert(gate.output.to_owned(), (new_off, new_on, diff));
				}
			}
		}

		cached[&self.lc.output]
	}

	pub fn update_weights(&mut self, lr: f64, pr: f64, selected: Vec<usize>) {
		for (layer, curr_node_id) in self.layers.iter_mut().zip(selected.iter()) {
			layer.update_weight(lr, pr, *curr_node_id);
		}
	}
}
