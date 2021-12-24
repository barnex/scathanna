mod internal;

mod editor;
mod game_state;
mod gl_client;
pub mod net;
mod physics;
mod voxel_world;
mod voxelstore;

pub use editor::EdState;
pub use game_state::*;
pub use gl_client::GLClient;

pub const ASSETS_PATH: &str = "assets/";
pub const MAPS_PATH: &str = "assets/maps/";
pub const TEXTURES_PATH: &str = "assets/textures/";
pub const OBJ_PATH: &str = "assets/obj/";
