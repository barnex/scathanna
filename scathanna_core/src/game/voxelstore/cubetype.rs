use super::internal::*;

/// Summary of a cube's contents.
/// TODO: private in voxelstore
#[derive(Debug, PartialEq, Eq)]
pub enum CubeType {
	Uniform(VoxelType),
	Mixed,
}
