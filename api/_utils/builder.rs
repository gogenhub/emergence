use crate::_utils::{devices, helpers, logic_circuit, parser};
use devices::{Device, Gate, GateKind};
use helpers::{args_from_to, get_gate_kind, Error};
use logic_circuit::{LogicCircuit, Testbench};
use parser::{Def, Function, Operation, ParserIter, Test};
use std::collections::{HashMap, HashSet};

pub struct LogicCircuitBuilder<'a> {
	parse_iter: ParserIter<'a>,
	function_tree: HashMap<String, Function>,
	test_tree: HashMap<String, Test>,
}

impl<'a> LogicCircuitBuilder<'a> {
	pub fn new(parse_iter: ParserIter<'a>) -> Self {
		Self {
			parse_iter: parse_iter,
			function_tree: HashMap::new(),
			test_tree: HashMap::new(),
		}
	}

	fn check_function_errors(&self, func: &Function) -> Result<(), Error> {
		let mut pmap = HashSet::new();
		for param in &func.params {
			Error::already_exists(pmap.contains(&param.value), param)?;
			pmap.insert(param.value.to_owned());
		}

		let mut vmap = HashSet::new();
		let mut vunused = HashMap::new();
		for op in &func.body {
			match op {
				Operation::Logic(lop) => {
					let kind = get_gate_kind(&lop.symbol)?;
					match kind {
						GateKind::Not => Error::invalid_number_of_args(lop.args.len() != 1, &lop.symbol)?,
						GateKind::Nor => Error::invalid_number_of_args(lop.args.len() != 2, &lop.symbol)?,
					};
					Error::already_exists(vmap.contains(&lop.var.value) || pmap.contains(&lop.var.value), &lop.var)?;
					for arg in &lop.args {
						vunused.remove(&arg.value);
						Error::not_found(!vmap.contains(&arg.value) && !pmap.contains(&arg.value), &arg)?;
					}

					vmap.insert(lop.var.value.to_owned());
					vunused.insert(lop.var.value.to_owned(), lop.var.clone());
				}
			}
		}

		Error::not_found(!vmap.contains(&func.out.value), &func.out)?;
		vunused.remove(&func.out.value);

		// TODO: add warning instead of error
		for (_, var) in vunused {
			Error::not_used(true, &var)?;
		}

		Ok(())
	}

	pub fn check_test_errors(&self, test: &Test) -> Result<(), Error> {
		let mut at_set = HashSet::new();
		let mut pmap = HashMap::new();
		for param in &test.params {
			Error::already_exists(pmap.contains_key(&param.value), &param)?;
			pmap.insert(param.value.to_owned(), param);
		}

		for bp in &test.body {
			Error::already_exists(at_set.contains(&bp.time), &bp.symbol)?;
			at_set.insert(bp.time);

			let mut assm = HashSet::new();
			for ass in &bp.assignments {
				Error::already_exists(assm.contains(&ass.iden.value), &ass.iden)?;
				Error::not_found(!pmap.contains_key(&ass.iden.value), &ass.iden)?;
				assm.insert(ass.iden.value.to_owned());
			}
		}
		Ok(())
	}

	pub fn build_parse_tree(&mut self) -> Result<(), Error> {
		while let Some(res) = self.parse_iter.next() {
			let res = res?;
			match res {
				Def::Function(func) => {
					Error::already_exists(self.function_tree.contains_key(&func.iden.value), &func.iden)?;
					self.check_function_errors(&func)?;
					self.function_tree.insert(func.iden.value.to_owned(), func);
				}
				Def::Test(test) => {
					Error::not_found(!self.function_tree.contains_key(&test.iden.value), &test.iden)?;
					Error::already_exists(self.test_tree.contains_key(&test.iden.value), &test.iden)?;
					self.check_test_errors(&test)?;
					self.test_tree.insert(test.iden.value.to_owned(), test);
				}
			}
		}

		Ok(())
	}

	fn build_devices(&self, func: &Function, pmap: &HashMap<String, String>) -> Vec<Device> {
		let mut devices = Vec::new();
		for op in &func.body {
			match op {
				Operation::Logic(gop) => {
					let inputs: Vec<String> = gop
						.args
						.iter()
						.map(|v| {
							if pmap.contains_key(&v.value) {
								return pmap[&v.value].to_owned();
							}
							v.value.to_owned()
						})
						.collect();
					let kind = get_gate_kind(&gop.symbol).unwrap();
					devices.push(Device::Gate(Gate {
						output: gop.var.value.to_owned(),
						kind,
						inputs,
					}));
				}
			}
		}

		devices
	}

	pub fn build_testbench(&mut self) -> Testbench {
		let main_test = self.test_tree.get("main").unwrap();
		let mut at_bp = HashMap::new();
		for bp in &main_test.body {
			let mut assigns = HashMap::new();
			for ass in &bp.assignments {
				assigns.insert(ass.iden.value.to_owned(), ass.value);
			}
			at_bp.insert(bp.time, assigns);
		}

		Testbench { breakpoints: at_bp }
	}

	pub fn build_logic_circut(&mut self) -> LogicCircuit {
		let main_func = self.function_tree.get("main").unwrap();
		let main_test = self.test_tree.get("main").unwrap();
		let pmap = args_from_to(&main_func.params, &main_test.params);
		let devices = self.build_devices(main_func, &pmap);

		let inputs = main_test.params.iter().map(|x| x.value.to_string()).collect();
		let output = main_func.out.value.to_string();
		let testbench = self.build_testbench();
		LogicCircuit {
			devices,
			inputs,
			output,
			testbench,
		}
	}
}
