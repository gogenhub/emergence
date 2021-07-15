use crate::{genetic_circuit, utils::data};
use data::get_data;
use genetic_circuit::{Component, Signal};
use std::collections::hash_map::HashMap;

#[derive(Clone, Debug)]
pub struct Input {
	pub name: String,
	pub value: String,
}

impl Input {
	pub fn num_biological(&self) -> usize {
		let data = get_data();
		data.signals_len()
	}

	pub fn into_biological(&self, cached: &mut HashMap<String, Component>) -> Vec<Component> {
		let data = get_data();
		let signal = data.get_signal(&self.value);
		cached.insert(self.name.to_string(), Component::Signal(signal.clone()));
		vec![Component::Signal(signal.clone())]
	}
}
