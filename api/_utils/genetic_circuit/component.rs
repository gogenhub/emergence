use super::{gene::Gene, signal::Signal};
use core::panic;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub enum Component {
	Gene(Gene),
	Signal(Signal),
}

impl Component {
	pub fn signal(&self) -> Signal {
		match self {
			Component::Signal(sig) => sig.clone(),
			_ => panic!("Component is not a Signal"),
		}
	}

	pub fn group(&self) -> String {
		match self {
			Component::Gene(gene) => gene.group(),
			Component::Signal(sig) => sig.group(),
		}
	}

	pub fn name(&self) -> String {
		match self {
			Component::Gene(gene) => gene.name(),
			Component::Signal(sig) => sig.name(),
		}
	}

	pub fn promoter(&self) -> String {
		match self {
			Component::Gene(gene) => gene.promoter(),
			Component::Signal(sig) => sig.promoter(),
		}
	}

	pub fn apply_rules(&mut self) {
		match self {
			Component::Gene(gene) => gene.apply_rules(),
			Component::Signal(sig) => {}
		}
	}

	pub fn into_dna(
		&self,
		dna: &mut String,
		plasmid: &mut String,
		promoter_colors: &mut HashMap<String, String>,
	) {
		match self {
			Component::Gene(gene) => gene.into_dna(dna, plasmid, promoter_colors),
			Component::Signal(sig) => {}
		}
	}

	pub fn test_steady_state(&self, cached: &mut HashMap<String, (f64, f64, f64, f64)>) {
		match self {
			Component::Gene(gene) => gene.test_steady_state(cached),
			Component::Signal(sig) => {}
		}
	}

	pub fn simulation_steady_state(&self, cached: &mut HashMap<String, (f64, f64)>) {
		match self {
			Component::Gene(gene) => gene.simulation_steady_state(cached),
			Component::Signal(sig) => {}
		}
	}

	pub fn model_and_save(
		&self,
		states: &mut HashMap<String, f64>,
		history: &mut HashMap<String, Vec<f64>>,
	) {
		match self {
			Component::Gene(gene) => gene.model_and_save(states, history),
			Component::Signal(sig) => {}
		}
	}
}
