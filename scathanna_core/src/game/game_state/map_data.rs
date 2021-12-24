use super::internal::*;

/// The immutable portion of a GameState.
pub struct MapData {
	pub name: String,
	pub voxels: Voxels,
	pub metadata: Metadata,
}

impl MapData {
	pub const VOXEL_FILE: &'static str = "voxels.bincode.gz";
	pub const METADATA_FILE: &'static str = "metadata.json";

	pub fn load(map_name: &str) -> Result<Self> {
		let dir = map_directory(map_name);
		let voxels = Voxels::load(dir.join(Self::VOXEL_FILE))?;
		let metadata = Metadata::load(dir.join(Self::METADATA_FILE))?;
		Ok(Self {
			voxels,
			metadata,
			name: map_name.to_owned(),
		})
	}

	pub fn save(&self, dir: &Path) -> Result<()> {
		self.voxels.save(dir.join(Self::VOXEL_FILE))?;
		self.metadata.save(&dir.join(Self::METADATA_FILE))?;
		Ok(())
	}

	/// Where does a ray intersect the map, if any.
	pub fn intersect(&self, ray: &DRay) -> Option<f64> {
		self.voxels.intersect(ray).map(|(_, t)| t)
	}
}

/// Turn a map name (e.g. "fun_map") into the corresponding directory
/// (e.g. "assets/maps/fun_map.sc").
pub fn map_directory(map_name: &str) -> PathBuf {
	abs_path(&Path::new(MAPS_PATH).join(map_name).with_extension("sc"))
}
