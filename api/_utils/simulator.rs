use crate::_utils::{assembler, builder, data, helpers};
use assembler::GeneticCircuit;
use builder::Testbench;
use data::get_input;
use helpers::transfer;
use std::collections::HashMap;

pub fn simulate(testbench: &Testbench, gc: &GeneticCircuit) -> (HashMap<String, Vec<f64>>, HashMap<String, (f64, f64)>) {
	let mut states = HashMap::new();
	let mut history: HashMap<String, Vec<f64>> = HashMap::new();
	let mut steady_states: HashMap<String, (f64, f64)> = HashMap::new();
	for inp in &gc.inputs {
		states.insert(inp.promoter.to_owned(), inp.rpu_off);
		history.insert(inp.promoter.to_owned(), Vec::new());
		steady_states.insert(inp.promoter.to_owned(), (inp.rpu_off, inp.rpu_on));
	}
	for gene in &gc.genes {
		states.insert(gene.promoter.to_owned(), 0.0);
		history.insert(gene.promoter.to_owned(), Vec::new());

		let (mut sum_off, mut sum_on) = (0.0, 0.0);
		for inp in &gene.inputs {
			let (off, on) = steady_states.get(inp).unwrap();
			sum_on += on;
			sum_off += off;
		}

		let steady_off = transfer(sum_on, &gene.params) / gene.params.decay;
		let steady_on = transfer(sum_off, &gene.params) / gene.params.decay;

		steady_states.insert(gene.promoter.to_owned(), (steady_off, steady_on));
	}
	for i in 0..1000 {
		if testbench.at_breakpoints.contains_key(&i) {
			let bp = testbench.at_breakpoints.get(&i).unwrap();
			for (name, val) in bp {
				let inp = get_input(name);
				states.insert(inp.promoter.to_owned(), if *val == true { inp.rpu_on } else { inp.rpu_off });
			}
		}

		for inp in &gc.inputs {
			let state = states.get(&inp.promoter).unwrap();
			let hist = history.get_mut(&inp.promoter).unwrap();
			hist.push(*state);
		}

		for gene in &gc.genes {
			let state = states.get(&gene.promoter).unwrap();
			let sum: f64 = gene.inputs.iter().map(|pro| states.get(pro).unwrap()).sum();

			let flux = transfer(sum, &gene.params) - gene.params.decay * state;
			let new_state = state + flux;
			states.insert(gene.promoter.to_owned(), new_state);
			let hist = history.get_mut(&gene.promoter).unwrap();
			hist.push(new_state);
		}
	}
	(history, steady_states)
}
