use crate::_utils::{data, helpers};
use data::{get_gate, get_output, get_part, GeneticCircuit, PartKind};
use helpers::{make_plasmid_dna, make_plasmid_part, make_plasmid_title};
use std::collections::HashMap;

pub fn make_dna(gc: &GeneticCircuit) -> (String, String, String, String) {
	let mut gates_plasmid = String::new();
	let mut promoter_colors = HashMap::new();

	let pre_gates = get_part("gates_pre_backbone");
	let mut gates_dna = pre_gates.seq.to_owned();

	gates_plasmid += &make_plasmid_part(
		&pre_gates.kind,
		0,
		gates_dna.len(),
		&pre_gates.name,
		"white",
	);

	for gene in &gc.genes {
		promoter_colors.insert(gene.promoter.to_owned(), gene.color.to_owned());
		for inp in &gene.inputs {
			let part = get_part(inp);
			let start = gates_dna.len();
			let end = start + part.seq.len();

			gates_dna += &part.seq;
			gates_plasmid += &make_plasmid_part(
				&part.kind,
				start,
				end,
				&part.name,
				promoter_colors.get(inp).unwrap_or(&"white".to_owned()),
			);
		}

		let bio_gate = get_gate(&gene.name);
		for part_name in &bio_gate.parts {
			let part = get_part(part_name);
			let start = gates_dna.len();
			let end = start + part.seq.len();

			gates_dna += &part.seq;
			gates_plasmid += &make_plasmid_part(&part.kind, start, end, &part.name, &gene.color);
		}
	}

	let post_gates1 = get_part("gates_post_backbone1");
	let post_gates2 = get_part("gates_post_backbone2");

	let start1 = gates_dna.len();
	let end1 = start1 + post_gates1.seq.len();

	gates_dna += &post_gates1.seq;

	let start2 = gates_dna.len();
	let end2 = start2 + post_gates2.seq.len();

	gates_dna += &post_gates2.seq;

	gates_plasmid +=
		&make_plasmid_part(&post_gates1.kind, start1, end1, &post_gates1.name, "white");
	gates_plasmid +=
		&make_plasmid_part(&post_gates2.kind, start2, end2, &post_gates2.name, "white");

	let gates_title = make_plasmid_title("gates-plasmid", gates_dna.len());
	let gates_plasmid_dna: String = make_plasmid_dna(&gates_dna);
	let final_gates_plasmid = gates_title + &gates_plasmid + &gates_plasmid_dna;

	let mut output_plasmid = String::new();
	let pre_output = get_part("output_pre_backbone");
	let mut output_dna = pre_output.seq.to_owned();

	output_plasmid += &make_plasmid_part(
		&pre_output.kind,
		0,
		output_dna.len(),
		&pre_output.name,
		"white",
	);

	for inp in &gc.output.inputs {
		let part = get_part(inp);
		let start = output_dna.len();
		let end = start + part.seq.len();

		output_dna += &part.seq;

		output_plasmid += &make_plasmid_part(
			&part.kind,
			start,
			end,
			&part.name,
			promoter_colors.get(inp).unwrap_or(&"white".to_owned()),
		)
	}
	let out_part = get_output(&gc.output.name);
	let start = output_dna.len();
	let end = start + out_part.len();

	output_plasmid += &make_plasmid_part(&PartKind::Output, start, end, &gc.output.name, "white");

	output_dna += &out_part;

	let post_output = get_part("output_post_backbone");
	let start = output_dna.len();
	let end = start + post_output.seq.len();

	output_plasmid += &make_plasmid_part(&post_output.kind, start, end, &post_output.name, "white");
	output_dna += &post_output.seq;

	let output_title = make_plasmid_title("output-plasmid", output_plasmid.len());

	let output_plasmid_dna = make_plasmid_dna(&output_dna);
	let final_output_plasmid = output_title + &output_plasmid + &output_plasmid_dna;

	(
		gates_dna,
		output_dna,
		final_gates_plasmid,
		final_output_plasmid,
	)
}
