use crate::_utils::data::PartKind;
use chrono::Utc;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct Dna {
	pub raw: String,
	pub plasmid: String,
}

impl Dna {
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
            "LOCUS      {}      {} bp ds-Dna      circular      {}\nFEATURES             Location/Qualifiers\n",
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
}
