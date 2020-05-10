use crate::_utils::{helpers, parser};
use helpers::{
	args_from_to, compile_err, format_args_for_gate, format_ret_for_gate, get_gate_kind, map_hms,
	ret_from_to, uw, Error, Warning,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use parser::{Arg, Def, DefKind, ExpressionKind, Operation, OperationKind, ParserIter};

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum GateKind {
	OR,
	NOT,
	NOR,
	AND,
	NAND,
	XOR,
	Unknown,
}

impl GateKind {
	pub fn to_string(&self) -> String {
		match self {
			GateKind::OR => "OR".to_string(),
			GateKind::NOT => "NOT".to_string(),
			GateKind::NOR => "NOR".to_string(),
			GateKind::AND => "AND".to_string(),
			GateKind::NAND => "NAND".to_string(),
			GateKind::XOR => "XOR".to_string(),
			GateKind::Unknown => "Unknown".to_string(),
		}
	}
}

#[derive(Debug)]
pub struct Gate {
	pub inputs: Vec<String>,
	pub kind: GateKind,
}

#[derive(Debug)]
pub struct LogicCircut {
	pub inputs: Vec<Arg>,
	pub output: Arg,
	pub gates: HashMap<String, Gate>,
}

pub struct LogicCircutBuilder<'a> {
	parse_iter: ParserIter<'a>,
	parse_tree: HashMap<String, Def>,
}

impl<'a> LogicCircutBuilder<'a> {
	pub fn new(parse_iter: ParserIter<'a>) -> Self {
		Self {
			parse_iter: parse_iter,
			parse_tree: HashMap::new(),
		}
	}

	fn check_op_errors(
		&self,
		op: &Operation,
		fn_map: &mut HashMap<String, (usize, usize, bool)>,
		param_map: &mut HashMap<&String, (usize, bool)>,
		retr_map: &mut HashMap<&String, (usize, bool)>,
		local_vars_map: &mut HashMap<&String, (usize, bool)>,
	) -> Result<(), Error> {
		if op.kind == OperationKind::Call && !fn_map.contains_key(&op.name) {
			return Err(compile_err(
				format!("Function '{}' not found.", op.name),
				(op.pos, op.name.len()),
			));
		}

		if op.kind == OperationKind::Call {
			let def = fn_map.get_mut(&op.name).unwrap();
			if op.args.len() != def.1 {
				return Err(compile_err(
					format!(
						"Function '{}' accepts {} arguments, found {}.",
						op.name,
						def.1,
						op.args.len()
					),
					(op.pos, op.name.len()),
				));
			}
			def.2 = true;
		}

		if op.kind == OperationKind::Operation {
			let kind = get_gate_kind(&op.name);
			if kind == GateKind::Unknown {
				return Err(compile_err(
					format!("Invalid operation '{}'.", op.name),
					(op.pos, op.name.len()),
				));
			}
		}

		for arg in &op.args {
			if retr_map.contains_key(&arg.name) {
				return Err(compile_err(
					format!(
						"Return variable '{}' can't be passed to a operation.",
						arg.name
					),
					(arg.pos, arg.name.len()),
				));
			}

			if !param_map.contains_key(&arg.name) && !local_vars_map.contains_key(&arg.name) {
				return Err(compile_err(
					format!("Argument '{}' not found.", arg.name),
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

	fn check_def_errors(
		&self,
		def: &Def,
		defs_map: &mut HashMap<String, (usize, usize, bool)>,
	) -> Result<Vec<Warning>, Error> {
		let mut params_map = HashMap::new();
		for param in &def.params {
			if params_map.contains_key(&param.name) {
				return Err(compile_err(
					format!("Parameter with name '{}' already exists.", param.name),
					(param.pos, param.name.len()),
				));
			}
			params_map.insert(&param.name, (param.pos, false));
		}
		if params_map.contains_key(&def.ret.name) {
			return Err(compile_err(
				format!(
					"Return variable has the same name as one of the args: {}",
					def.ret.name
				),
				(def.ret.pos, def.ret.name.len()),
			));
		}
		let mut retr_map = HashMap::new();
		retr_map.insert(&def.ret.name, (def.ret.pos, false));

		let mut local_vars_map = HashMap::new();
		for exp in &def.body {
			if local_vars_map.contains_key(&exp.var.name) || params_map.contains_key(&exp.var.name)
			{
				return Err(compile_err(
					format!("Variable '{}' already exists.", exp.var.name),
					(exp.var.pos, exp.var.name.len()),
				));
			}

			if exp.kind == ExpressionKind::Assign && retr_map.contains_key(&exp.var.name) {
				return Err(compile_err(
					"Return variable don't need 'let' keyword.".to_owned(),
					(exp.var.pos, exp.var.name.len()),
				));
			}

			if exp.kind == ExpressionKind::Return && !retr_map.contains_key(&exp.var.name) {
				return Err(compile_err(
					format!("Return variable '{}' not found.", exp.var.name),
					(exp.var.pos, exp.var.name.len()),
				));
			}

			if exp.kind == ExpressionKind::Return {
				(retr_map.get_mut(&exp.var.name).unwrap()).1 = true;
			}

			self.check_op_errors(
				&exp.op,
				defs_map,
				&mut params_map,
				&mut retr_map,
				&mut local_vars_map,
			)?;

			if exp.kind == ExpressionKind::Assign {
				local_vars_map.insert(&exp.var.name, (exp.var.pos, false));
			}
		}

		let mut warnings = Vec::new();
		for (key, (pos, used)) in params_map {
			if !used {
				warnings.push(uw(
					format!("Parametar '{}' is never used.", key),
					(pos, key.len()),
				));
			}
		}

		for (key, (pos, used)) in retr_map {
			if !used {
				warnings.push(uw(
					format!("Return variable '{}' never assigned.", key),
					(pos, key.len()),
				));
			}
		}

		for (key, (pos, used)) in local_vars_map {
			if !used {
				warnings.push(uw(
					format!("Variable '{}' never assigned.", key),
					(pos, key.len()),
				));
			}
		}

		Ok(warnings)
	}

	fn build_gates(
		&self,
		def: &Def,
		id: &str,
		args_map: &HashMap<String, String>,
		rets_map: &HashMap<String, String>,
		gates: &mut HashMap<String, Gate>,
	) {
		for (i, exp) in def.body.iter().enumerate() {
			let op = &exp.op;
			match op.kind {
				// normal operation
				OperationKind::Operation => {
					let kind = get_gate_kind(&exp.op.name);
					let ins = format_args_for_gate(&op.args, &args_map, id);
					let out = format_ret_for_gate(&exp.var, &rets_map, id);
					gates.insert(
						out,
						Gate {
							kind: kind,
							inputs: ins,
						},
					);
				}
				// function call
				OperationKind::Call => {
					let call_def = self.parse_tree.get(&op.name).unwrap();

					let new_id = format!("{}{}{}", id, op.name, i);

					let arg_map_to_current = args_from_to(&call_def.params, &op.args);
					let ret_map_to_current = ret_from_to(&call_def.ret, &exp.var);

					let arg_map_new = map_hms(&arg_map_to_current, args_map, &id);
					let ret_map_new = map_hms(&ret_map_to_current, rets_map, &id);

					self.build_gates(call_def, &new_id, &arg_map_new, &ret_map_new, gates);
				}
			}
		}
	}

	fn build_parse_tree(&mut self) -> Result<Vec<Warning>, Error> {
		let mut parse_tree = HashMap::new();
		let mut fn_map = HashMap::new();
		let mut gene_set = HashSet::new();
		let mut warnings = Vec::new();
		while let Some(res) = self.parse_iter.next() {
			if res.is_err() {
				return Err(res.unwrap_err());
			}

			let (name, def) = res.unwrap();
			if fn_map.contains_key(&name) || gene_set.contains(&name) {
				return Err(compile_err(
					format!("Function or gene with name '{}' already defined.", name),
					(def.pos, name.len()),
				));
			}

			let warns = self.check_def_errors(&def, &mut fn_map)?;
			warnings.extend(warns);

			if def.kind == DefKind::Function {
				fn_map.insert(name.to_owned(), (def.pos, def.params.len(), false));
			} else {
				gene_set.insert(name.to_owned());
			}

			parse_tree.insert(name, def);
		}

		if !gene_set.contains("main") {
			return Err(compile_err("Gene 'main' is required.".to_owned(), (0, 0)));
		}

		for (key, (pos, _, used)) in fn_map {
			if !used {
				warnings.push(uw(
					format!("Function '{}' never used.", key),
					(pos, key.len()),
				));
			}
		}
		self.parse_tree = parse_tree;

		Ok(warnings)
	}

	pub fn build_logic_circut(&mut self) -> Result<(LogicCircut, Vec<Warning>), Error> {
		let warnings = self.build_parse_tree()?;
		let main_def = self.parse_tree.get("main").unwrap();
		let mut gates = HashMap::new();
		let arg_map = args_from_to(&main_def.params, &main_def.params);
		let ret_map = ret_from_to(&main_def.ret, &main_def.ret);
		self.build_gates(main_def, "", &arg_map, &ret_map, &mut gates);

		let ins = main_def.params.clone();
		let out = main_def.ret.clone();
		let lc = LogicCircut {
			gates: gates,
			inputs: ins,
			output: out,
		};
		Ok((lc, warnings))
	}
}
