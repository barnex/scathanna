use super::internal::*;

pub struct Triangle<T> {
	/// vertex0
	pub o: Vector3<T>,
	/// (vertex1 - vertex0)
	pub a: Vector3<T>,
	/// (vertex2 - vertex0)
	pub b: Vector3<T>,
}

impl<T> Triangle<T>
where
	T: Number,
{
	pub fn new(a: Vector3<T>, b: Vector3<T>, c: Vector3<T>) -> Self {
		// Check for degeneracy: vertices should not be near-collinear.
		//debug_assert!((b - a).cross(c - a).len().as_f64() > 1e-6);
		Self { o: a, a: b - a, b: c - a }
	}

	/// Vertex 0.
	pub fn v0(&self) -> Vector3<T> {
		self.o
	}

	/// Vertex 1.
	pub fn v1(&self) -> Vector3<T> {
		self.o + self.a
	}

	/// Vertex 2.
	pub fn v2(&self) -> Vector3<T> {
		self.o + self.b
	}
}

impl<T> Triangle<T>
where
	T: Float,
{
	#[inline]
	pub fn intersects(&self, r: &Ray<T>) -> bool {
		self.intersect_t(r).is_some()
	}
	#[inline]
	pub fn intersect_t(&self, r: &Ray<T>) -> Option<T> {
		let a = self.a;
		let b = self.b;
		let n = a.cross(b);
		let s = r.start - self.o;
		let t = -n.dot(s) / n.dot(r.dir);

		if t < T::ZERO {
			return None
		}

		// TODO: possible early return if t < 0 || t > hitrecord.t

		let p = s + r.dir * t;

		// Barycentric coordinates for 3D triangle, after
		// Peter Shirley, Fundamentals of Computer Graphics, 2nd Edition.
		let nc = a.cross(p);
		let na = (b - a).cross(p - a);
		//let nb = p.cross(b);
		let n2 = n.dot(n);
		let l1 = n.dot(na) / n2;
		let l3 = n.dot(nc) / n2;
		//let l2 = n.dot(nb) / n2;
		let l2 = T::ONE - l1 - l3;

		//if !(l1 >= 0. && l2 >= 0. && l3 >= 0.) {
		//	// Note: `!(x>0)` handles NaN gracefully
		//	return false;
		//}

		//t >= 0. && l1 >= 0. && l2 >= 0. && l3 >= 0.
		if T::partial_min(T::partial_min(l1, l2), l3) < T::ZERO {
			return None
		}

		Some(t)
	}

	pub fn sized_normal(&self) -> Vector3<T> {
		self.a.cross(self.b)
	}
}



#[cfg(test)]
mod test {
	use super::*;

	const EZ: dvec3 = dvec3::EZ;

	fn ray(start: (f64, f64, f64), dir: dvec3) -> Ray64 {
		Ray::new(start.into(), dir)
	}

	/*

			 * (3,4)
			/|
		   / |
		  /  |
	(1,2)*---* (3,2)

	*/
	#[test]
	fn intersects() {
		let t = Triangle::new(dvec3(1., 2., -1.), dvec3(3., 2., -1.), dvec3(3., 4., -1.));

		assert!(!t.intersects(&ray((0., 0., 0.,), -EZ)));
		assert!(!t.intersects(&ray((0., 0., 0.,), EZ)));

		assert!(t.intersects(&ray((2., 3., 0.,), -EZ)));
		assert!(!t.intersects(&ray((2., 3., 0.,), EZ)));

		assert!(!t.intersects(&ray((4., 3., 0.,), -EZ)));
		assert!(!t.intersects(&ray((4., 3., 0.,), EZ)));
		assert!(!t.intersects(&ray((2., -3., 0.,), -EZ)));
		assert!(!t.intersects(&ray((2., -3., 0.,), EZ)));

		assert!(!t.intersects(&ray((0., 0., -2.,), EZ)));
		assert!(!t.intersects(&ray((0., 0., -2.,), -EZ)));

		assert!(t.intersects(&ray((2., 3., -2.,), EZ)));
		assert!(!t.intersects(&ray((2., 3., -2.,), -EZ)));
	}
}
