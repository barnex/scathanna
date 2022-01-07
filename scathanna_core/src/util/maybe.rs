pub type Maybe = Option<()>;

#[allow(non_snake_case)]
pub fn Maybe(x: ()) -> Maybe {
	Some(x)
}
