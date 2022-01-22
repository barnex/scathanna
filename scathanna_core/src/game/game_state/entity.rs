use super::internal::*;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Entity {
	id: EID,
	pub position: vec3,
	pub kind: EKind,
}

pub type Entities = HashMap<EID, Entity>;

pub type EID = u64;
static NEXT_ENTITY_ID: AtomicU64 = AtomicU64::new(1);

// A fresh, unique entity number.
fn new_entity_id() -> EID {
	NEXT_ENTITY_ID.fetch_add(1, Ordering::SeqCst)
}

impl Entity {
	pub fn new(position: vec3, kind: EKind) -> Self {
		Self { id: new_entity_id(), position, kind }
	}

	pub fn id(&self) -> EID {
		self.id
	}

	pub fn bounds(&self) -> BoundingBox<f32> {
		// TODO.
		let pos = self.position;
		let hsize = 3.0;
		let vsize = 3.0; // TODO
		let min = pos - vec3(hsize / 2.0, 0.0, hsize / 2.0);
		let max = pos + vec3(hsize / 2.0, vsize, hsize / 2.0);
		BoundingBox::new(min, max)
	}
}

#[derive(Serialize, Deserialize, Clone, Debug, Copy, PartialEq, Eq)]
pub enum EKind {
	GiftBox { pickup_point_id: Option<usize> },
	CowboyHat,
	BerserkerHelmet,
	PartyHat,
	XMasHat,
}

use EKind::*;

impl EKind {
	pub const ALL_POWERUPS: [EKind; 5] = [
		GiftBox { pickup_point_id: None },
		CowboyHat, //
		BerserkerHelmet,
		PartyHat,
		XMasHat,
	];

	pub const ALL_EKINDS: [EKind; 5] = [
		GiftBox { pickup_point_id: None },
		CowboyHat, //
		BerserkerHelmet,
		PartyHat,
		XMasHat,
	];

	pub fn as_str(self) -> &'static str {
		use EKind::*;
		match self {
			GiftBox { .. } => "gift_box",
			CowboyHat => "cowboy_hat",
			BerserkerHelmet => "berserker_helmet",
			PartyHat => "party_hat",
			XMasHat => "xmas_hat",
			//_ => "thingamabob",
		}
	}

	pub fn description(self) -> &'static str {
		use EKind::*;
		match self {
			CowboyHat => "Shoot as fast as you can pull the trigger",
			BerserkerHelmet => "It's self-explanatory",
			PartyHat => "One extra life, love lava",
			XMasHat => "Unlimited jumping",
			GiftBox { .. } => "There's a prize on your head",
		}
	}

	// pick a random powerup, except the one that the player already has.
	pub fn random_powerup_except(current: Option<EKind>) -> EKind {
		match current {
			None => Self::random_powerup(),
			Some(current) => loop {
				let rand = Self::random_powerup();
				if rand != current {
					return rand;
				}
			},
		}
	}

	// pick a random powerup.
	pub fn random_powerup() -> EKind {
		Self::ALL_POWERUPS[rand::thread_rng().gen_range(0..Self::ALL_POWERUPS.len())]
	}

	/// Check if `self` and `other` have the same enum discriminant.
	/// E.g., check if are both of kind `GiftBox`, regardless of inner data.
	pub fn is_kind(&self, other: &Self) -> bool {
		mem::discriminant(self) == mem::discriminant(other)
	}
}

impl FromStr for EKind {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self> {
		Self::ALL_EKINDS //
			.into_iter()
			.filter(|k| k.as_str() == s)
			.next()
			.ok_or(anyhow!("unknown entity: {}", s))
	}
}

pub fn touches(bounds: &BoundingBox<f32>, other: BoundingBox<f32>) -> bool {
	// TODO: could be refined
	bounds.contains(other.center())
}
