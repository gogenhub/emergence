use crate::_utils::{helpers, parser};
use helpers::{
	args_from_to, compile_err, format_args_for_gate, format_ret_for_gate, get_gate_kind, map_hms,
	ret_from_to, Error,
};
use parser::{
	Arg, BreakpointKind, Def, Expression, Function, Operation, OperationKind, ParserIter, Test,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum GateKind {
	NOT,
	NOR,
	Unknown,
}

#[derive(Serialize, Debug)]
pub struct Gate {
	pub inputs: Vec<String>,
	pub kind: GateKind,
}

#[derive(Debug, Serialize)]
pub struct LogicCircuit {
	pub inputs: Vec<Arg>,
	pub output: Arg,
	pub gates: HashMap<String, Gate>,
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

	fn check_op_errors(
		&self,
		op: &Operation,
		fn_map: &mut HashMap<String, (usize, usize, bool)>,
		param_map: &mut HashMap<&String, (usize, bool)>,
		local_vars_map: &mut HashMap<&String, (usize, bool)>,
		retr_map: &mut HashMap<&String, (usize, bool)>,
	) -> Result<(), Error> {
		if op.kind == OperationKind::Call && !fn_map.contains_key(&op.name) {
			return Err(compile_err(
				format!("Function '{}' not found.", op.name),
				(op.pos, op.name.len()),
			));
		}

		if op.kind == OperationKind::Call {
			let func = fn_map.get_mut(&op.name).unwrap();
			if op.args.len() != func.1 {
				return Err(compile_err(
					format!(
						"Function '{}' accepts {} arguments, found {}.",
						op.name,
						func.1,
						op.args.len()
					),
					(op.pos, op.name.len()),
				));
			}
			func.2 = true;
		}

		if op.kind == OperationKind::Inline {
			let kind = get_gate_kind(&op.name);
			if kind == GateKind::Unknown {
				return Err(compile_err(
					format!("Invalid operation '{}'.", op.name),
					(op.pos, op.name.len()),
				));
			}
		}

		for arg in &op.args {
			if !param_map.contains_key(&arg.name) && !local_vars_map.contains_key(&arg.name) {
				return Err(compile_err(
					format!("Argument '{}' not found.", arg.name),
					(arg.pos, arg.name.len()),
				));
			}

			if retr_map.contains_key(&arg.name) {
				return Err(compile_err(
					format!("Can't pass return variable '{}'.", arg.name),
					(arg.pos, arg.name.len()),
				));
			}

			(param_map
				.get_mut(&arg.name)
				.or(local_vars_map.get_mut(&arg.name))
				.unwrap())
			.1 = true;
		}

		Ok(())
	}

	fn check_function_errors(
		&self,
		func: &Function,
		fn_map: &mut HashMap<String, (usize, usize, bool)>,
	) -> Result<(), Error> {
		let mut params_map = HashMap::new();
		for param in &func.params {
			if params_map.contains_key(&param.name) {
				return Err(compile_err(
					format!("Parameter with name '{}' already exists.", param.name),
					(param.pos, param.name.len()),
				));
			}
			params_map.insert(&param.name, (param.pos, false));
		}
		if params_map.contains_key(&func.ret.name) {
			return Err(compile_err(
				format!(
					"Return variable has the same name as one of the args: {}",
					func.ret.name
				),
				(func.ret.pos, func.ret.name.len()),
			));
		}

		let mut local_vars_map = HashMap::new();
		let mut ret_map = HashMap::new();
		ret_map.insert(&func.ret.name, (func.ret.pos, false));
		for exp in &func.body {
			match exp {
				Expression::Declare(declare) => {
					for var in &declare.vars {
						if local_vars_map.contains_key(&var.name) {
							return Err(compile_err(
								format!("Variable '{}' already exists.", var.name),
								(var.pos, var.name.len()),
							));
						}

						if params_map.contains_key(&var.name) {
							return Err(compile_err(
								format!(
									"Variable can't have the same name as parameter: '{}'.",
									var.name
								),
								(var.pos, var.name.len()),
							));
						}

						if ret_map.contains_key(&var.name) {
							return Err(compile_err(
								format!(
									"Variable can't have the same name as return variable: '{}'.",
									var.name
								),
								(var.pos, var.name.len()),
							));
						}

						local_vars_map.insert(&var.name, (var.pos, false));
					}
				}
				Expression::Assign(assign) => {
					if !local_vars_map.contains_key(&assign.var.name)
						&& !ret_map.contains_key(&assign.var.name)
					{
						return Err(compile_err(
							format!("Variable '{}' not found.", assign.var.name),
							(assign.var.pos, assign.var.name.len()),
						));
					}
					(local_vars_map
						.get_mut(&assign.var.name)
						.or(ret_map.get_mut(&assign.var.name))
						.unwrap())
					.1 = true;
					self.check_op_errors(
						&assign.op,
						fn_map,
						&mut params_map,
						&mut local_vars_map,
						&mut ret_map,
					)?;
				}
			}
		}

		for (key, (pos, used)) in params_map {
			if !used {
				return Err(compile_err(
					format!("Parametar '{}' is never used.", key),
					(pos, key.len()),
				));
			}
		}

		for (key, (pos, used)) in local_vars_map {
			if !used {
				return Err(compile_err(
					format!("Variable '{}' never used.", key),
					(pos, key.len()),
				));
			}
		}

		for (key, (pos, used)) in ret_map {
			if !used {
				return Err(compile_err(
					format!("Return variable '{}' never used.", key),
					(pos, key.len()),
				));
			}
		}

		Ok(())
	}

	pub fn check_test_errors(
		&self,
		test: &Test,
		fn_map: &mut HashMap<String, (usize, usize, bool)>,
	) -> Result<(), Error> {
		if !fn_map.contains_key(&test.name) {
			return Err(compile_err(
				format!("Function with name '{}' not defined.", test.name),
				(test.pos, test.name.len()),
			));
		}
		let test_fn = fn_map.get_mut(&test.name).unwrap();
		test_fn.2 = true;
		let mut at_set = HashSet::new();
		let mut params_map = HashMap::new();
		for param in &test.params {
			if params_map.contains_key(&param.name) {
				return Err(compile_err(
					format!("Parameter with name '{}' already exists.", param.name),
					(param.pos, param.name.len()),
				));
			}
			params_map.insert(param.name.to_owned(), param);
		}
		if params_map.contains_key(&test.ret.name) {
			return Err(compile_err(
				format!("Parameter with name '{}' already exists.", test.ret.name),
				(test.ret.pos, test.ret.name.len()),
			));
		}

		for bp in &test.body {
			match bp.kind {
				BreakpointKind::At => {
					if at_set.contains(&bp.time) {
						return Err(compile_err(
							format!("Exact breakpoint at '{}' already exists.", bp.time),
							(bp.pos, bp.name.len()),
						));
					}
					at_set.insert(bp.time);
				}
				BreakpointKind::Unknown => {
					return Err(compile_err(
						format!("Unknown breakpoint type '{}'.", bp.time),
						(test.ret.pos, test.ret.name.len()),
					));
				}
			};

			let mut assignments = HashSet::new();
			for ass in &bp.assignments {
				if assignments.contains(&ass.name) {
					return Err(compile_err(
						format!(
							"Assignment for '{}' already exist for '{}' breakpoint.",
							ass.name, bp.time
						),
						(ass.pos, ass.name.len()),
					));
				}

				if !params_map.contains_key(&ass.name) {
					return Err(compile_err(
						format!("Parameter '{}' not found.", ass.name),
						(ass.pos, ass.name.len()),
					));
				}

				assignments.insert(ass.name.to_owned());
			}
		}
		Ok(())
	}

	pub fn build_parse_tree(&mut self) -> Result<(), Error> {
		let mut fn_map = HashMap::new();
		let mut test_map = HashSet::new();
		while let Some(res) = self.parse_iter.next() {
			if res.is_err() {
				return Err(res.unwrap_err());
			}

			let def = res.unwrap();
			match def {
				Def::Function(func) => {
					if fn_map.contains_key(&func.name) {
						return Err(compile_err(
							format!("Function with name '{}' already defined.", func.name),
							(func.pos, func.name.len()),
						));
					}
					self.check_function_errors(&func, &mut fn_map)?;
					fn_map.insert(func.name.to_owned(), (func.pos, func.params.len(), false));
					self.function_tree.insert(func.name.to_owned(), func);
				}
				Def::Test(test) => {
					if test_map.contains(&test.name) {
						return Err(compile_err(
							format!("Test with name '{}' already defined.", test.name),
							(test.pos, test.name.len()),
						));
					}
					self.check_test_errors(&test, &mut fn_map)?;
					test_map.insert(test.name.to_owned());
					self.test_tree.insert(test.name.to_owned(), test);
				}
			}
		}

		for (name, (pos, _, used)) in fn_map {
			if !used {
				return Err(compile_err(
					format!("Function '{}' not used.", name),
					(pos, name.len()),
				));
			}
		}

		if test_map.len() > 1 {
			return Err(compile_err(
				"Only one test is supported.".to_owned(),
				(0, 0),
			));
		}

		if !test_map.contains("main") {
			return Err(compile_err("Test 'main' is required.".to_owned(), (0, 0)));
		}

		Ok(())
	}

	fn build_gates(
		&self,
		func: &Function,
		id: &str,
		args_map: &HashMap<String, String>,
		rets_map: &HashMap<String, String>,
		gates: &mut HashMap<String, Gate>,
	) {
		for (i, exp) in func.body.iter().enumerate() {
			if exp.is_declare() {
				continue;
			}
			let assign = exp.assign();
			let op = &assign.op;
			match op.kind {
				// normal operation
				OperationKind::Inline => {
					let kind = get_gate_kind(&op.name);
					let ins = format_args_for_gate(&op.args, &args_map, id);
					let out = format_ret_for_gate(&assign.var, &rets_map, id);
					gates.insert(
						out.to_owned(),
						Gate {
							kind: kind,
							inputs: ins,
						},
					);
				}
				// function call
				OperationKind::Call => {
					let call_func = self.function_tree.get(&op.name).unwrap();

					let new_id = format!("{}{}{}", id, op.name, i);

					let arg_map_to_current = args_from_to(&call_func.params, &op.args);
					let ret_map_to_current = ret_from_to(&call_func.ret, &assign.var);

					let arg_map_new = map_hms(&arg_map_to_current, args_map, &id);
					let ret_map_new = map_hms(&ret_map_to_current, rets_map, &id);

					self.build_gates(call_func, &new_id, &arg_map_new, &ret_map_new, gates);
				}
			}
		}
	}

	pub fn build_logic_circut(&mut self) -> LogicCircuit {
		let main_func = self.function_tree.get("main").unwrap();
		let main_test = self.test_tree.get("main").unwrap();
		let mut gates = HashMap::new();
		let arg_map = args_from_to(&main_func.params, &main_test.params);
		let ret_map = ret_from_to(&main_func.ret, &main_test.ret);
		self.build_gates(main_func, "", &arg_map, &ret_map, &mut gates);

		let ins = main_test.params.clone();
		let out = main_test.ret.clone();
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
			match bp.kind {
				BreakpointKind::At => {
					at_bp.insert(bp.time, assigns);
				}
				_ => (),
			}
		}

		Testbench {
			at_breakpoints: at_bp,
		}
	}
}
