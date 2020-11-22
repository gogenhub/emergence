use crate::_utils::{builder, data, helpers};
use builder::Testbench;
use data::{get_gate, get_input, GeneticCircuit};
use helpers::transfer;
use std::collections::HashMap;

pub fn simulate(
	testbench: &Testbench,
	gc: &GeneticCircuit,
) -> (HashMap<String, Vec<f64>>, HashMap<String, (f64, f64)>) {
	let mut states = HashMap::new();
	let mut history: HashMap<String, Vec<f64>> = HashMap::new();
	let mut steady_states: HashMap<String, (f64, f64)> = HashMap::new();
	for inp in &gc.inputs {
		states.insert(inp.promoter.to_owned(), 0.0);
		history.insert(inp.promoter.to_owned(), Vec::new());
		steady_states.insert(inp.promoter.to_owned(), (inp.rpu_on, inp.rpu_off));
	}
	for gene in &gc.genes {
		states.insert(gene.promoter.to_owned(), 0.0);
		history.insert(gene.promoter.to_owned(), Vec::new());

		let (mut sum_off, mut sum_on) = (0.0, 0.0);
		for inp in &gene.inputs {
			let (on, off) = steady_states.get(inp).unwrap();
			sum_on += on;
			sum_off += off;
		}
		let gate = get_gate(&gene.name);

		let steady_on = transfer(sum_off, &gate.params) / gate.params.decay;
		let steady_off = transfer(sum_on, &gate.params) / gate.params.decay;

		steady_states.insert(gene.promoter.to_owned(), (steady_on, steady_off));
	}
	for i in 0..1000 {
		if testbench.at_breakpoints.contains_key(&i) {
			let bp = testbench.at_breakpoints.get(&i).unwrap();
			for (name, val) in bp {
				let inp = get_input(name);
				states.insert(
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
			let state = states.get(&inp.promoter).unwrap();
			let hist = history.get_mut(&inp.promoter).unwrap();
			hist.push(*state);
		}

		for gene in &gc.genes {
			let gate = get_gate(&gene.name);
			let state = states.get(&gene.promoter).unwrap();
			let sum: f64 = gene
				.inputs
				.iter()
				.map(|name| states.get(name).unwrap())
				.sum();

			let flux = transfer(sum, &gate.params) - gate.params.decay * state;
			let new_state = state + flux;
			states.insert(gene.promoter.to_owned(), new_state);
			let hist = history.get_mut(&gene.promoter).unwrap();
			hist.push(new_state);
		}
	}
	(history, steady_states)
}
