use crate::{
	logic_circuit, parser,
	utils::{data, error},
};
use data::get_data;
use error::Error;
use logic_circuit::{Device, Gate, GateKind, Input, LogicCircuit, Output, Testbench};
use parser::{lexer::Token, Def, Enviroment, Implementation, Module, Operation, ParserIter, Test};
use std::collections::{HashMap, HashSet};

pub struct LogicCircuitBuilder<'a> {
	parse_iter: ParserIter<'a>,
	impl_tree: HashMap<String, Implementation>,
	test_tree: HashMap<String, Test>,
	env_tree: HashMap<String, Enviroment>,
	mod_tree: HashMap<String, Module>,
}

impl<'a> LogicCircuitBuilder<'a> {
	pub fn new(parse_iter: ParserIter<'a>) -> Self {
		Self {
			parse_iter,
			impl_tree: HashMap::new(),
			test_tree: HashMap::new(),
			env_tree: HashMap::new(),
			mod_tree: HashMap::new(),
		}
	}

	pub fn get_gate_kind(token: &Token) -> Result<GateKind, Error> {
		match token.value.as_str() {
			"not" => Ok(GateKind::Not),
			"nor" => Ok(GateKind::Nor),
			_ => Err(Error::UnexpectedToken(
				token.value.to_string(),
				token.pos,
				token.value.len(),
			)),
		}
	}

	fn check_implementation_errors(&mut self, imp: Implementation) -> Result<(), Error> {
		Error::already_exists(self.impl_tree.contains_key(&imp.name.value), &imp.name)?;
		Error::not_found(!self.mod_tree.contains_key(&imp.name.value), &imp.name)?;

		let data = get_data();
		if imp.body.len() > data.genes_len() {
			return Err(Error::NotEnoughGenes);
		}

		let module = self.mod_tree.get(&imp.name.value).unwrap();
		Error::invalid_number_of_args(module.outs.len() != 1, &module.name)?;

		let mut pmap = HashSet::new();
		let mut rmap = HashSet::new();
		let inputs = &module.ins;
		let outputs = &module.outs;
		for inp in inputs {
			pmap.insert(inp.value.to_string());
		}
		for out in outputs {
			rmap.insert(out.value.to_string());
		}

		let mut vmap = HashSet::new();
		let mut vunused = HashMap::new();
		for op in &imp.body {
			match op {
				Operation::Logic(lop) => {
					let kind = Self::get_gate_kind(&lop.symbol)?;
					match kind {
						GateKind::Not => {
							Error::invalid_number_of_args(lop.args.len() != 1, &lop.symbol)?
						}
						GateKind::Nor => {
							Error::invalid_number_of_args(lop.args.len() != 2, &lop.symbol)?
						}
					};
					Error::already_exists(
						vmap.contains(&lop.var.value) || pmap.contains(&lop.var.value),
						&lop.var,
					)?;
					rmap.remove(&lop.var.value);
					for arg in &lop.args {
						vunused.remove(&arg.value);
						Error::not_found(
							!vmap.contains(&arg.value) && !pmap.contains(&arg.value),
							&arg,
						)?;
					}

					vmap.insert(lop.var.value.to_string());
					vunused.insert(lop.var.value.to_string(), lop.var.clone());
				}
			}
		}

		for arg in &module.outs {
			vunused.remove(&arg.value);
		}

		// TODO: add warning instead of error
		for (_, var) in vunused {
			Error::not_used(true, &var)?;
		}

		self.impl_tree.insert(imp.name.value.to_string(), imp);

		Ok(())
	}

	pub fn check_test_errors(&mut self, test: Test) -> Result<(), Error> {
		Error::already_exists(self.test_tree.contains_key(&test.name.value), &test.name)?;
		Error::not_found(
			!self.mod_tree.contains_key(&test.module.value),
			&test.module,
		)?;
		Error::not_found(!self.env_tree.contains_key(&test.name.value), &test.name)?;

		let env = self.env_tree.get(&test.name.value).unwrap();
		let inputs = &env.ins;
		let mut pmap = HashSet::new();
		for inp in inputs {
			pmap.insert(inp.value.to_string());
		}

		let module = self.mod_tree.get(&test.module.value).unwrap();
		let env = self.env_tree.get(&test.name.value).unwrap();

		let same_len = (module.ins.len() == env.ins.len()) && (module.outs.len() == env.outs.len());
		Error::invalid_number_of_args(!same_len, &test.name)?;

		let mut at_set = HashSet::new();
		for bp in &test.body {
			Error::already_exists(at_set.contains(&bp.time), &bp.symbol)?;
			at_set.insert(bp.time);

			let mut assm = HashSet::new();
			for ass in &bp.assignments {
				Error::already_exists(assm.contains(&ass.iden.value), &ass.iden)?;
				Error::not_found(!pmap.contains(&ass.iden.value), &ass.iden)?;
				assm.insert(ass.iden.value.to_string());
			}
		}

		self.test_tree.insert(test.name.value.to_string(), test);
		Ok(())
	}

	pub fn check_module_error(&mut self, module: Module) -> Result<(), Error> {
		Error::already_exists(self.mod_tree.contains_key(&module.name.value), &module.name)?;
		let ins = &module.ins;
		let outs = &module.outs;
		let mut arg_map = HashSet::new();
		let mut ret_map = HashSet::new();
		for arg in ins {
			Error::already_exists(arg_map.contains(&arg.value), &arg)?;
			arg_map.insert(arg.value.to_string());
		}

		for arg in outs {
			Error::already_exists(ret_map.contains(&arg.value), &arg)?;
			ret_map.insert(arg.value.to_string());
		}

		self.mod_tree.insert(module.name.value.to_string(), module);
		Ok(())
	}

	pub fn check_enviroment_error(&mut self, env: Enviroment) -> Result<(), Error> {
		Error::already_exists(self.env_tree.contains_key(&env.name.value), &env.name)?;

		let data = get_data();

		let ins = &env.ins;
		let outs = &env.outs;
		let mut arg_map = HashSet::new();
		let mut ret_map = HashSet::new();
		for arg in ins {
			Error::already_exists(arg_map.contains(&arg.value), &arg)?;
			Error::not_found(!data.has_signal(&arg.value), &arg)?;
			arg_map.insert(arg.value.to_string());
		}

		for arg in outs {
			Error::already_exists(ret_map.contains(&arg.value), &arg)?;
			Error::not_found(!data.has_actuator(&arg.value), &arg)?;
			ret_map.insert(arg.value.to_string());
		}
		self.env_tree.insert(env.name.value.to_string(), env);
		Ok(())
	}

	pub fn build_parse_tree(&mut self) -> Result<(), Error> {
		while let Some(res) = self.parse_iter.next() {
			let res = res?;
			match res {
				Def::Implementation(imp) => self.check_implementation_errors(imp)?,
				Def::Test(test) => self.check_test_errors(test)?,
				Def::Module(module) => self.check_module_error(module)?,
				Def::Enviroment(env) => self.check_enviroment_error(env)?,
			}
		}

		Ok(())
	}

	fn build_devices(&self, imp: &Implementation) -> Vec<Device> {
		let mut devices = Vec::new();
		for op in &imp.body {
			match op {
				Operation::Logic(gop) => {
					let inputs: Vec<String> =
						gop.args.iter().map(|v| v.value.to_string()).collect();
					let kind = Self::get_gate_kind(&gop.symbol).unwrap();
					devices.push(Device::Gate(Gate {
						output: gop.var.value.to_string(),
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
				assigns.insert(ass.iden.value.to_string(), ass.value);
			}
			at_bp.insert(bp.time, assigns);
		}

		Testbench { breakpoints: at_bp }
	}

	pub fn build_logic_circut(&mut self) -> LogicCircuit {
		let main_mod = self.mod_tree.get("main").unwrap();
		let main_env = self.env_tree.get("main").unwrap();
		let main_impl = self.impl_tree.get("main").unwrap();
		let devices = self.build_devices(&main_impl);

		let mut inputs = Vec::new();
		let mut outputs = Vec::new();
		for i in 0..main_mod.ins.len() {
			let mod_inp = &main_mod.ins[i];
			let env_inp = &main_env.ins[i];
			inputs.push(Input {
				name: mod_inp.value.to_string(),
				value: env_inp.value.to_string(),
			});
		}
		for i in 0..main_mod.outs.len() {
			let mod_out = &main_mod.outs[i];
			let env_out = &main_env.outs[i];
			outputs.push(Output {
				name: mod_out.value.to_string(),
				value: env_out.value.to_string(),
			});
		}
		let testbench = self.build_testbench();
		LogicCircuit {
			devices,
			inputs,
			outputs,
			testbench,
		}
	}
}
