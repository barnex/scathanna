use super::internal::*;
use smallvec::*;
use std::str::FromStr;

#[test]
fn test_statement_ok() {
	use ParsedLine::*;

	let parse = |l| ParsedLine::from_str(l).unwrap();

	assert_eq!(parse(""), Unknown("".into()));
	assert_eq!(parse("  "), Unknown("  ".into()));
	assert_eq!(parse("#  a comment"), Comment("a comment".into()));
	assert_eq!(parse("mtllib lib.mtl"), Mtllib("lib.mtl".into()));
	assert_eq!(parse("o Cube"), O("Cube".into()));
	assert_eq!(parse("s off"), S(false));
	assert_eq!(parse("s 1"), S(true));
	assert_eq!(parse("v 1 2 3"), V(vec3(1.0, 2.0, 3.0)));
	assert_eq!(parse("v  1   2  3  "), V(vec3(1.0, 2.0, 3.0)));
	assert_eq!(parse("vt 1 2"), Vt(vec2(1.0, 2.0)));
	assert_eq!(parse("usemtl Material.001"), Usemtl("Material.001".into()));
	assert_eq!(parse("f 1 2 3"), F(smallvec![VIndex(1, None, None), VIndex(2, None, None), VIndex(3, None, None)]));
	assert_eq!(parse("f 1/2 3/4/5 6//7"), F(smallvec![VIndex(1, Some(2), None), VIndex(3, Some(4), Some(5)), VIndex(6, None, Some(7))]));

	assert_eq!(
		parse("f 1/1/1 2/2/2 3/3/3"),
		F(smallvec![VIndex(1, Some(1), Some(1)), VIndex(2, Some(2), Some(2)), VIndex(3, Some(3), Some(3)),])
	);
}

#[test]
fn test_statement_err() {
	let parse = ParsedLine::from_str;
	assert!(parse("v").is_err());
	assert!(parse("v 1").is_err());
	assert!(parse("v 1 2").is_err());
	assert!(parse("v bla bla bla").is_err());
	assert!(parse("v 1 2 3 4").is_err());
	assert!(parse("vt").is_err());
	assert!(parse("vt 1").is_err());
	assert!(parse("vt 1 2 3").is_err());
	assert!(parse("f").is_err());
	assert!(parse("f 1").is_err());
	assert!(parse("f 1 2").is_err());
	assert!(parse("f 1 2 bad").is_err());
}

#[test]
fn test_parse() {
	let input = r"
# Test OBJ file
mtllib test.mtl
o Triangle1
v 1.0 2.0 3.0
v 4.0 5.0 6.0
v 7.0 8.0 9.0
vt 0.1 0.2
vt 0.3 0.4
vt 0.5 0.6
vn 1.0 0.0 0.0
vn 0.0 1.0 0.0
vn 0.0 0.0 1.0
usemtl Material.001
s 1
f 1/1/1 2/2/2 3/3/3
o Quad
v 10.0 11.0 12.0
vt 0.7 0.8
vn 1.0 1.0 1.0
usemtl Material.002
s off
f 1/1/1 2/2/2 3/3/3 4/4/4
 "
	.as_bytes();
	let got = parse(input).unwrap(); // test passes if it does not error out

	let want = ObjSet {
		mtllib: Some(String::from("test.mtl")),
		objects: vec![
			//
			Object {
				name: "Triangle1".into(),
				mtl: Some("Material.001".into()),
				faces: vec![smallvec![
					Vertex {
						position: vec3(1.0, 2.0, 3.0),
						texture: vec2(0.1, 0.2),
						normal: vec3(1.0, 0.0, 0.0),
					},
					Vertex {
						position: vec3(4.0, 5.0, 6.0),
						texture: vec2(0.3, 0.4),
						normal: vec3(0.0, 1.0, 0.0),
					},
					Vertex {
						position: vec3(7.0, 8.0, 9.0),
						texture: vec2(0.5, 0.6),
						normal: vec3(0.0, 0.0, 1.0),
					},
				]],
			},
			Object {
				name: "Quad".into(),
				mtl: Some("Material.002".into()),
				faces: vec![smallvec![
					Vertex {
						position: vec3(1.0, 2.0, 3.0),
						texture: vec2(0.1, 0.2),
						normal: vec3(1.0, 0.0, 0.0),
					},
					Vertex {
						position: vec3(4.0, 5.0, 6.0),
						texture: vec2(0.3, 0.4),
						normal: vec3(0.0, 1.0, 0.0),
					},
					Vertex {
						position: vec3(7.0, 8.0, 9.0),
						texture: vec2(0.5, 0.6),
						normal: vec3(0.0, 0.0, 1.0),
					},
					Vertex {
						position: vec3(10.0, 11.0, 12.0),
						texture: vec2(0.7, 0.8),
						normal: vec3(1.0, 1.0, 1.0),
					},
				]],
			},
		],
	};

	assert_eq!(got, want);
}
