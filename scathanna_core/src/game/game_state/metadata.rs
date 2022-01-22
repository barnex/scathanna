use super::internal::*;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Metadata {
	#[serde(default)]
	pub spawn_points: Vec<SpawnPoint>,

	#[serde(default)]
	pub pickup_points: Vec<PickupPoint>,

	#[serde(default = "default_sun_dir")]
	pub sun_direction: vec3,
}

fn default_sun_dir() -> vec3 {
	// value used for lightmap baking before stored in map metadata.
	vec3(0.304855380424846, 0.609710760849692, 0.731652913019631)
}

impl Metadata {
	pub fn new() -> Self {
		Self::default()
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
