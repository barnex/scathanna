use super::internal::*;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SoundEffect {
	pub clip_name: CowStr,
	pub volume: f32,
	pub spatial: Option<Spatial>,
}

/// Information needed to create spatial sound.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Spatial {
	/// Where the sound originates.
	pub location: vec3,
	// Distance at which the sound has unit volume.
	//pub unit_distance: f32,
}

impl SoundEffect {
	/// Construct a SoundEffect message without spatial audio.
	/// Used e.g. for the announcer's voice.
	pub fn raw(clip_name: &'static str) -> Self {
		Self {
			clip_name: clip_name.into(),
			volume: 1.0,
			spatial: None,
		}
	}

	pub fn spatial(clip_name: &'static str, location: vec3, volume: f32) -> Self {
		Self {
			clip_name: clip_name.into(),
			volume,
			spatial: Some(Spatial { location }),
		}
	}
}
