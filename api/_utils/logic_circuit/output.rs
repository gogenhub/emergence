use crate::_utils::genetic_circuit;
use genetic_circuit::{Actuator, Component};
use std::collections::HashMap;

#[derive(Clone)]
pub struct Output {
	pub name: String,
	pub value: String,
}

impl Output {
	pub fn into_biological(&self, cached: &HashMap<String, Component>) -> Actuator {
		let input = cached.get(&self.name).unwrap().promoter().to_string();
		Actuator {
			name: self.value.to_string(),
			input,
		}
	}
}
