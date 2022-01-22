use super::internal::*;

#[derive(Clone)]
pub enum Material {
	UniformColor(vec3),
	VertexColor,
	FlatTexture(Rc<Texture>),
	MatteTexture(Rc<Texture>),
	Glossy(Rc<Texture>), // TODO: sun_intensity,...
	Lightmap { texture: Rc<Texture>, lightmap: Rc<Texture> },
}
