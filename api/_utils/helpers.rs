use crate::_utils::{assembler, builder, parser};

use assembler::{Params, PartKind};
use builder::GateKind;
use chrono::Utc;
use parser::Arg;
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

#[derive(Serialize, Debug)]
pub enum WarningKind {
	UnusedVar,
}

#[derive(Serialize, Debug)]
pub struct Warning {
	pub kind: WarningKind,
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

pub fn gate_logic(inputs: Vec<bool>, rpus: Vec<f64>, kind: &GateKind) -> (bool, f64) {
	match kind {
		GateKind::NOT => (!inputs[0], rpus[0]),
		GateKind::NOR => {
			for inp in inputs {
				if inp {
					return (false, rpus[0].max(rpus[1]));
				}
			}
			(true, rpus[0].max(rpus[1]))
		}
		_ => panic!("wtf"),
	}
}

pub fn get_gate_kind(symbol: &str) -> GateKind {
	match symbol {
		"~" => GateKind::NOT,
		"~|" => GateKind::NOR,
		_ => GateKind::Unknown,
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

pub fn uw(message: String, pos: (usize, usize)) -> Warning {
	Warning {
		kind: WarningKind::UnusedVar,
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

pub fn map(num: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
	(num - in_min) * (out_max - out_min) / (in_max - in_min) + out_min
}

pub fn transfer(x: f64, params: &Params) -> f64 {
	params.ymin + (params.ymax - params.ymin) / (1.0 + (x / params.k).powf(params.n))
}

pub fn get_group(curr: String) -> String {
	let group: Vec<&str> = curr.split("_").collect();
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
				let index_fmt = format!("     {:>9}", (i * 60) + 1);
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
