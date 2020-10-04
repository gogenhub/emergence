use crate::_utils::{assembler, builder, parser};

use assembler::{Params, PartKind};
use builder::GateKind;
use chrono::Utc;
use parser::{Arg, BreakpointKind};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Debug)]
pub enum ErrorKind {
	SyntaxError,
	CompileError,
	AssignError,
}

#[derive(Serialize, Debug)]
pub struct Error {
	pub kind: ErrorKind,
	pub message: String,
	pub pos: (usize, usize),
}

pub fn args_from_to(from: &Vec<Arg>, to: &Vec<Arg>) -> HashMap<String, String> {
	let map: HashMap<String, String> = from
		.iter()
		.zip(to.iter())
		.map(|(x, y)| (x.name.to_owned(), y.name.to_owned()))
		.collect();

	map
}

pub fn ret_from_to(from: &Arg, to: &Arg) -> HashMap<String, String> {
	let mut map = HashMap::new();
	map.insert(from.name.to_owned(), to.name.to_owned());
	map
}

pub fn map_hms(
	from: &HashMap<String, String>,
	to: &HashMap<String, String>,
	prefix: &str,
) -> HashMap<String, String> {
	let new_hm = from
		.iter()
		.map(|(k, v)| {
			(
				k.to_owned(),
				to.get(v).unwrap_or(&format!("{}{}", prefix, v)).to_owned(),
			)
		})
		.collect();
	new_hm
}

pub fn format_args_for_gate(
	from: &Vec<Arg>,
	args_map: &HashMap<String, String>,
	id: &str,
) -> Vec<String> {
	from.iter()
		.map(|x| {
			args_map
				.get(&x.name)
				.unwrap_or(&format!("{}{}", id, x.name))
				.to_owned()
		})
		.collect()
}

pub fn format_ret_for_gate(from: &Arg, rets_map: &HashMap<String, String>, id: &str) -> String {
	rets_map
		.get(&from.name)
		.unwrap_or(&format!("{}{}", id, from.name))
		.to_owned()
}

pub fn get_gate_kind(symbol: &str) -> GateKind {
	match symbol {
		"~" => GateKind::NOT,
		"~|" => GateKind::NOR,
		_ => GateKind::Unknown,
	}
}

pub fn get_breakpoint_kind(symbol: &str) -> BreakpointKind {
	match symbol {
		"@" => BreakpointKind::At,
		_ => BreakpointKind::Unknown,
	}
}

pub fn compile_err(message: String, pos: (usize, usize)) -> Error {
	Error {
		kind: ErrorKind::CompileError,
		message: message,
		pos: pos,
	}
}

pub fn assign_err(message: String, pos: (usize, usize)) -> Error {
	Error {
		kind: ErrorKind::AssignError,
		message: message,
		pos: pos,
	}
}

pub fn eof() -> Error {
	Error {
		kind: ErrorKind::SyntaxError,
		message: "Unexpected end of file.".to_owned(),
		pos: (0, 0),
	}
}

pub fn syntax_err(message: String, from: usize, len: usize) -> Error {
	Error {
		kind: ErrorKind::SyntaxError,
		message: message,
		pos: (from, len),
	}
}

pub fn map(num: f64, in_min: f64, in_max: f64, out_min: f64, out_max: f64) -> f64 {
	(num - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}

pub fn transfer(x: f64, params: &Params) -> f64 {
	params.ymin + (params.ymax - params.ymin) / (1.0 + (x / params.k).powf(params.n))
}

pub fn lerp(curr: f64, target: f64, step: f64) -> f64 {
	step * (target - curr) + curr
}

pub struct MotionParams {
	pos_pos_coef: f64,
	pos_vel_coef: f64,
	vel_pos_coef: f64,
	vel_vel_coef: f64,
}

pub fn damp(state: f64, velocity: f64, target_state: f64, params: &MotionParams) -> (f64, f64) {
	let old_pos = state - target_state;
	let old_vel = velocity;

	let new_state = old_pos * params.pos_pos_coef + old_vel * params.pos_vel_coef + target_state;
	let new_velocity = old_pos * params.vel_pos_coef + old_vel * params.vel_vel_coef;

	(new_state, new_velocity)
}

pub fn damp_params(angular_freq: f64, delta_time: f64) -> MotionParams {
	let exp_term = (-angular_freq * delta_time).exp();
	let time_exp = delta_time * exp_term;
	let time_exp_freq = time_exp * angular_freq;

	MotionParams {
		pos_pos_coef: time_exp_freq + exp_term,
		pos_vel_coef: time_exp,
		vel_pos_coef: -angular_freq * time_exp_freq,
		vel_vel_coef: -time_exp_freq + exp_term,
	}
}

pub fn get_group(curr: &str) -> String {
	let group: Vec<&str> = curr.split("_").collect();
	if group.len() < 2 {
		return "none".to_owned();
	}
	group[1].to_owned()
}

pub fn make_plasmid_dna(seq: &str) -> String {
	return "ORIGIN\n".to_owned()
		+ &seq
			.as_bytes()
			.chunks(60)
			.enumerate()
			.map(|(i, chunk)| {
				let ch: Vec<String> = chunk
					.chunks(10)
					.map(|x| {
						let parsed: String = std::str::from_utf8(x).unwrap().to_owned();
						parsed
					})
					.collect();
				let index_fmt = format!("{:>9}", (i * 60) + 1);
				format!("{} {}", index_fmt, ch.join(" "))
			})
			.collect::<Vec<String>>()
			.join("\n");
}

pub fn make_plasmid_title(name: &str, len: usize) -> String {
	format!(
		"LOCUS      {}      {} bp ds-DNA      circular      {}\nFEATURES             Location/Qualifiers\n",
		name,
		len,
		Utc::today().format("%e-%b-%Y")
	)
}

pub fn make_plasmid_part(
	kind: &PartKind,
	start: usize,
	end: usize,
	label: &str,
	color: &str,
) -> String {
	return format!("     {:<16}{}..{}\n", format!("{:?}", kind), start + 1, end)
		+ &format!("                     /label={}\n", label)
		+ &format!("                     /ApEinfo_fwdcolor={}\n", color);
}
