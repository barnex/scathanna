use super::internal::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
	pub spawn_points: Vec<SpawnPoint>,
}

impl Metadata {
	pub fn new() -> Self {
		Self { spawn_points: vec![] }
	}

	pub fn save(&self, file: &Path) -> Result<()> {
		Ok(serde_json::to_writer_pretty(create(file)?, self)?)
	}

	pub fn load<P: AsRef<Path>>(file: P) -> Result<Self> {
		let file = file.as_ref();
		let r = open(file)?;
		Ok(serde_json::from_reader(r)?)
	}
}

impl Default for Metadata {
	fn default() -> Self {
		Self {
			spawn_points: vec![SpawnPoint { pos: ivec3(0, 0, 0) }],
		}
	}
}
