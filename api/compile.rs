extern crate base64;
extern crate chrono;
extern crate fs_extra;
extern crate regex;
extern crate serde;
extern crate serde_json;

mod _utils;

use _utils::{assembler, builder, helpers, lexer, parser};
use assembler::{Assembler, GeneticCircuit};
use helpers::Error;
use lambda_runtime::{error::HandlerError, start, Context};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::{collections::HashMap, error::Error as StdError, str, sync::Mutex};

static ASSIGNER: Lazy<Mutex<Assembler>> = Lazy::new(|| Mutex::new(Assembler::new()));

#[derive(Serialize, Debug)]
struct CompileResult {
	score: f64,
	gc: GeneticCircuit,
	gates_dna: String,
	out_dna: String,
	gates_plasmid: String,
	out_plasmid: String,
	simulation: HashMap<String, Vec<f64>>,
}

#[derive(Deserialize)]
struct NowEvent {
	#[serde(rename = "Action")]
	action: String,
	body: String,
}

#[derive(Deserialize, PartialEq)]
enum Method {
	POST,
	GET,
	OPTIONS,
	PUT,
	DELETE,
	PATCH,
}

#[derive(Deserialize)]
struct Request {
	host: String,
	path: String,
	method: Method,
	headers: HashMap<String, String>,
	#[serde(default = "String::new")]
	body: String,
	encoding: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Response {
	status_code: u16,
	headers: HashMap<String, String>,
	body: String,
	encoding: Option<String>,
}

fn compile(emergence: String) -> Result<CompileResult, Error> {
	let lx = lexer::LexerIter::new(emergence.chars());
	let prs = parser::ParserIter::new(lx);
	let mut bld = builder::LogicCircuitBuilder::new(prs);
	bld.build_parse_tree()?;
	let lc = bld.build_logic_circut();
	let tb = bld.build_testbench();
	let mut ass = ASSIGNER.lock().unwrap();
	if !ass.loaded {
		ass.load();
	}
	let (ass_gates, score) = ass.assign(&lc)?;
	let mut gc = ass.assemble(&lc, &ass_gates);
	let simulation = ass.simulate(&tb, &gc);
	ass.apply_rules(&mut gc);
	let (gates_dna, out_dna, gates_plasmid, out_plasmid) = ass.make_dna(&gc);
	Ok(CompileResult {
		score,
		gc,
		gates_dna,
		out_dna,
		gates_plasmid,
		out_plasmid,
		simulation,
	})
}

fn handler(e: NowEvent, _: Context) -> Result<Response, HandlerError> {
	let req: Request = serde_json::from_str(&e.body)?;

	let mut headers = HashMap::new();
	headers.insert("Access-Control-Allow-Origin".to_owned(), "*".to_owned());
	if req.method == Method::OPTIONS {
		headers.insert(
			"Access-Control-Request-Method".to_owned(),
			"POST, OPTIONS, GET".to_owned(),
		);
		headers.insert("Access-Control-Request-Headers".to_owned(), "*".to_owned());
		return Ok(Response {
			status_code: 200,
			headers: headers,
			body: "".to_owned(),
			encoding: None,
		});
	}
	let req_body = if req.encoding.is_some() && req.encoding.unwrap() == "base64" {
		str::from_utf8(&base64::decode(&req.body).unwrap_or_default())
			.unwrap()
			.to_owned()
	} else {
		req.body
	};

	let res = compile(req_body);

	let status_code: u16;
	let res_body;
	if res.is_err() {
		let err = res.unwrap_err();
		res_body = to_string(&err).unwrap();
		status_code = 400;
	} else {
		let result = res.unwrap();
		res_body = to_string(&result).unwrap();
		status_code = 200;
	}

	Ok(Response {
		status_code: status_code,
		headers: headers,
		body: res_body,
		encoding: None,
	})
}

fn main() -> Result<(), Box<dyn StdError>> {
	Ok(start(handler, None))
}
