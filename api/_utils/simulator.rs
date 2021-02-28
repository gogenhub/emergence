use crate::_utils::{assembler, builder, data};
use assembler::GeneticCircuit;
use builder::Testbench;
use data::get_data;
use std::collections::HashMap;

pub fn simulate(
	testbench: &Testbench,
	gc: &GeneticCircuit,
) -> (HashMap<String, Vec<f64>>, HashMap<String, (f64, f64)>) {
	let data = get_data();
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
		gene.simulation_steady_state(&mut steady_states);
	}
	for i in 0..1000 {
		if testbench.breakpoints.contains_key(&i) {
			let bp = testbench.breakpoints.get(&i).unwrap();
			for (name, val) in bp {
				let inp = data.get_input(name);
				states.insert(
					inp.promoter.to_owned(),
					if *val == true { inp.rpu_on } else { inp.rpu_off },
				);
			}
		}

		for inp in &gc.inputs {
			let state = states.get(&inp.promoter).unwrap();
			let hist = history.get_mut(&inp.promoter).unwrap();
			hist.push(*state);
		}

		for gene in &gc.genes {
			gene.model_and_save(&mut states, &mut history);
		}
	}
	(history, steady_states)
}