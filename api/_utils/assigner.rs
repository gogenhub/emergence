use crate::_utils::{data, devices, helpers, logic_circuit};
use data::get_data;
use devices::Device;
use helpers::Error;
use logic_circuit::LogicCircuit;
use rand::distributions::{Distribution, Uniform};
use rand::prelude::ThreadRng;
use std::collections::HashSet;

pub struct Layer {
	nodes: Vec<f64>,
	rng: ThreadRng,
	uni: Uniform<f64>,
	device: Device,
}

impl Layer {
	pub fn init(device: Device) -> Self {
		let len = match &device {
			Device::Gate(gate) => gate.num_components(),
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
		match &self.device {
			Device::Gate(gate) => gate.blacklist(i, bl),
		}
	}

	pub fn in_bl(&self, i: usize, bl: &HashSet<String>) -> bool {
		match &self.device {
			Device::Gate(gate) => gate.is_blacklisted(i, &bl),
		}
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
	num_iterations: usize,
}

impl GeneNetwork {
	pub fn out_error(x: f64) -> f64 {
		1.0 - (-x / 200.0).exp()
	}

	pub fn inv_out_error(x: f64) -> f64 {
		(-x / 10.0).exp()
	}

	pub fn lrate(&self, i: f64) -> f64 {
		let len = self.num_iterations as f64;
		(-i / len).exp()
	}

	pub fn init(lc: LogicCircuit, num_iterations: usize) -> Result<Self, Error> {
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
		Ok(Self {
			layers,
			lc,
			num_iterations,
		})
	}

	pub fn fit(&mut self) -> Result<Vec<usize>, Error> {
		let mut best_score = 0.0;
		let mut best_sel = Vec::new();
		for i in 0..self.num_iterations {
			let lr = self.lrate(i as f64);
			let sel_genes = self.walk();
			let (_, _, diff, score) = self.lc.into_biological(&sel_genes).test();

			let diff_err = Self::inv_out_error(diff);
			let diff_core = diff_err * score;
			if diff_core > best_score {
				best_score = diff_core;
				best_sel = sel_genes.clone();
			}
			let out = Self::out_error(diff_core);
			self.update_weights(lr, out, sel_genes);
		}
		Ok(best_sel)
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

	pub fn update_weights(&mut self, lr: f64, pr: f64, selected: Vec<usize>) {
		for (layer, curr_node_id) in self.layers.iter_mut().zip(selected.iter()) {
			layer.update_weight(lr, pr, *curr_node_id);
		}
	}
}