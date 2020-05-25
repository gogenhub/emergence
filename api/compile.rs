extern crate base64;
extern crate fs_extra;
extern crate meval;
extern crate regex;
extern crate serde;
extern crate serde_json;

mod _utils;

use _utils::{assigner, builder, helpers, lexer, parser};
use assigner::{BioGate, Output, Part};
use helpers::{Error, Warning};
use lambda_runtime::{error::HandlerError, start, Context};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use std::{collections::HashMap, error::Error as StdError, str, sync::Mutex};

static ASSIGNER: Lazy<Mutex<assigner::Assigner>> =
	Lazy::new(|| Mutex::new(assigner::Assigner::new()));

#[derive(Serialize, Debug)]
struct CompileResult {
	warnings: Vec<Warning>,
	gates: HashMap<String, BioGate>,
	output: Output,
	parts: HashMap<String, Part>,
}

fn compile(emergence: String) -> Result<CompileResult, Error> {
	let lx = lexer::LexerIter::new(emergence.chars());
	let prs = parser::ParserIter::new(lx);
	let mut bld = builder::LogicCircutBuilder::new(prs);
	let (lc, warnings) = bld.build_logic_circut()?;
	let mut ass = ASSIGNER.lock().unwrap();
	if !ass.loaded {
		ass.load();
	}
	let (gates, output, parts) = ass.assign_gates(lc)?;

	Ok(CompileResult {
		gates: gates,
		output: output,
		parts: parts,
		warnings: warnings,
	})
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
	let body;
	if res.is_err() {
		let err = res.unwrap_err();
		body = to_string(&err).unwrap();
		status_code = 400;
	} else {
		let result = res.unwrap();
		body = to_string(&result).unwrap();
		status_code = 200;
	}

	Ok(Response {
		status_code: status_code,
		headers: headers,
		body: body,
		encoding: None,
	})
}

fn main() -> Result<(), Box<dyn StdError>> {
	Ok(start(handler, None))
}
