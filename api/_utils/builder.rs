use crate::_utils::{helpers, parser};
use helpers::{args_from_to, err, exp, Error};
use parser::{Arg, BreakpointKind, Function, OperationKind, ParserIter, Test};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct Gate {
	pub name: String,
	pub inputs: Vec<String>,
	pub kind: OperationKind,
}

#[derive(Debug, Clone)]
pub struct LogicCircuit {
	pub inputs: Vec<Arg>,
	pub output: Arg,
	pub gates: Vec<Gate>,
}

pub struct Testbench {
	pub at_breakpoints: HashMap<u32, HashMap<String, bool>>,
}

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
			exp(!pmap.contains(&param.name), "Parameter already exists: ", param)?;
			pmap.insert(param.name.to_owned());
		}
		exp(!pmap.contains(&func.out.name), "Return variable can't be an arg: ", &func.out)?;

		let mut vmap = HashSet::new();
		let mut vunused = HashMap::new();
		for op in &func.body {
			let var = &op.var;
			exp(!vmap.contains(&var.name) && !pmap.contains(&var.name), "Variable already exists: ", var)?;
			for arg in &op.args {
				vunused.remove(&arg.name);
				exp(vmap.contains(&arg.name) || pmap.contains(&arg.name), "Argument not found: ", arg)?;
			}

			vmap.insert(var.name.to_owned());
			vunused.insert(var.name.to_owned(), var);
		}
		vunused.remove(&func.out.name);

		for (_, var) in vunused {
			exp(false, "Variable not used: ", var)?;
		}

		exp(vmap.contains(&func.out.name), "Variable not found: ", &func.out)?;

		Ok(())
	}

	pub fn check_test_errors(&self, test: &Test) -> Result<(), Error> {
		let mut at_set = HashSet::new();
		let mut pmap = HashMap::new();
		for param in &test.params {
			exp(!pmap.contains_key(&param.name), "Parameter with name already exists: ", param)?;
			pmap.insert(param.name.to_owned(), param);
		}

		for bp in &test.body {
			exp(!at_set.contains(&bp.time), "Breakpoint already exists: ", bp)?;
			at_set.insert(bp.time);

			let mut assm = HashSet::new();
			for ass in &bp.assignments {
				exp(!assm.contains(&ass.name), "Assignment already exists: ", ass)?;
				exp(pmap.contains_key(&ass.name), "Parameter not found: ", ass)?;
				assm.insert(ass.name.to_owned());
			}
		}
		Ok(())
	}

	pub fn build_parse_tree(&mut self) -> Result<(), Error> {
		while let Some(res) = self.parse_iter.next() {
			let res = res?;
			if res.is_func() {
				let func = res.func();
				exp(!self.function_tree.contains_key(&func.name), "Function already exists: ", &func)?;
				self.check_function_errors(&func)?;
				self.function_tree.insert(func.name.to_owned(), func);
			} else {
				let test = res.test();
				exp(self.function_tree.contains_key(&test.name), "Function not found: ", &test)?;
				exp(!self.test_tree.contains_key(&test.name), "Test already exists: ", &test)?;
				self.check_test_errors(&test)?;
				self.test_tree.insert(test.name.to_owned(), test);
			}
		}

		if self.function_tree.len() > 1 || !self.function_tree.contains_key("main") {
			return Err(err("Only 'main' function allowed.".to_owned(), (0, 0)));
		}

		Ok(())
	}

	fn build_gates(&self, func: &Function, pmap: &HashMap<String, String>) -> Vec<Gate> {
		let mut gates = Vec::new();
		for op in &func.body {
			let inputs: Vec<String> = op
				.args
				.iter()
				.map(|v| {
					if pmap.contains_key(&v.name) {
						return pmap[&v.name].to_owned();
					}
					v.name.to_owned()
				})
				.collect();
			gates.push(Gate {
				name: op.var.name.to_owned(),
				kind: op.kind.clone(),
				inputs,
			});
		}

		gates
	}

	pub fn build_logic_circut(&mut self) -> LogicCircuit {
		let main_func = self.function_tree.get("main").unwrap();
		let main_test = self.test_tree.get("main").unwrap();
		let pmap = args_from_to(&main_func.params, &main_test.params);
		let gates = self.build_gates(main_func, &pmap);

		let ins = main_test.params.clone();
		let out = main_func.out.clone();
		let lc = LogicCircuit {
			gates: gates,
			inputs: ins,
			output: out,
		};
		lc
	}

	pub fn build_testbench(&mut self) -> Testbench {
		let main_test = self.test_tree.get("main").unwrap();
		let mut at_bp = HashMap::new();
		for bp in &main_test.body {
			let mut assigns = HashMap::new();
			for ass in &bp.assignments {
				assigns.insert(ass.name.to_owned(), ass.value);
			}
			if bp.kind == BreakpointKind::At {
				at_bp.insert(bp.time, assigns);
			}
		}

		Testbench { at_breakpoints: at_bp }
	}
}
