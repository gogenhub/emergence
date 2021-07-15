use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Actuator {
	pub name: String,
	pub input: String,
}

impl Actuator {
	pub fn name(&self) -> String {
		self.name.to_string()
	}
}
