use super::*;
use crate::genetic_circuit::Component;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Device {
	Gate(Gate),
	Input(Input),
}

impl Device {
	pub fn num_biological(&self) -> usize {
		match self {
			Self::Gate(gate) => gate.num_biological(),
			Self::Input(input) => input.num_biological(),
		}
	}

	pub fn into_biological(
		&self,
		i: usize,
		cached: &mut HashMap<String, Component>,
	) -> Vec<Component> {
		match self {
			Self::Gate(gate) => gate.into_biological(i, cached),
			Self::Input(input) => input.into_biological(cached),
		}
	}
}
