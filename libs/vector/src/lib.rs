///! Vector types similar to OpenGL's.
pub mod prelude;

mod float;
mod vector2;
mod vector3;

mod dvec_2;
mod dvec_3;
mod ivec_2;
mod ivec_3;
mod uvec_2;
mod uvec_3;
mod vec_2;
mod vec_3;

pub use dvec_2::*;
pub use dvec_3::*;
pub use ivec_2::*;
pub use ivec_3::*;
pub use uvec_2::*;
pub use uvec_3::*;
pub use vec_2::*;
pub use vec_3::*;
pub use vector2::*;
pub use vector3::*;
