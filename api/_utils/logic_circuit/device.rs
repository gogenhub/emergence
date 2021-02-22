use super::*;
use crate::_utils::genetic_circuit::Component;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Device {
	Gate(Gate),
}

impl Device {
	pub fn num_biological(&self) -> usize {
		match self {
			Self::Gate(gate) => gate.num_biological(),
		}
	}

	pub fn into_biological(&self, i: usize, cached: &mut HashMap<String, Component>) -> Vec<Component> {
		match self {
			Self::Gate(gate) => gate.into_biological(i, cached),
		}
	}
}
