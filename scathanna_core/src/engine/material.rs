use super::internal::*;

#[derive(Clone)]
pub enum Material {
	UniformColor(vec3),
	VertexColor,
	FlatTexture(Rc<Texture>),
	MatteTexture(Rc<Texture>),
	Lightmap { texture: Rc<Texture>, lightmap: Rc<Texture> },
}
