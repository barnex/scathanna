use std::ops::*;
use vector::*;

/// 3x3 matrix intended for linear transformations.
#[derive(Clone, Copy)]
pub struct Matrix3<T> {
	el: [Vector3<T>; 3],
}

impl<T: Copy> Matrix3<T> {
	#[inline]
	pub fn new(a: Vector3<T>, b: Vector3<T>, c: Vector3<T>) -> Self {
		Self { el: [a, b, c] }
	}
}

/*
impl<T: Copy> From<[[T; 3]; 3]> for Matrix3<T> {
	fn from(arr: [[T; 3]; 3]) -> Self {
		Self { el: [arr[0], arr[1], arr[2]] }
	}
}
*/

impl<T> From<[Vector3<T>; 3]> for Matrix3<T>
where
	T: Copy,
{
	fn from(arr: [Vector3<T>; 3]) -> Self {
		Self {
			el: [arr[0].into(), arr[1].into(), arr[2].into()],
		}
	}
}

impl<T> Default for Matrix3<T>
where
	T: Default + Copy,
{
	fn default() -> Self {
		Self {
			el: [Vector3::default(), Vector3::default(), Vector3::default()],
		}
	}
}

/*
impl<T> Matrix3<T>
where
	T: Default + One + Copy,
{
	pub fn unit() -> Self {
		let o = T::default();
		let i = T::one();
		Matrix3::from([[i, o, o], [o, i, o], [o, o, i]])
	}
}
*/

impl<T> Index<usize> for Matrix3<T> {
	type Output = Vector3<T>;

	/// Index returns column i as vector.
	#[inline]
	fn index(&self, i: usize) -> &Self::Output {
		&self.el[i]
	}
}

impl<T> IndexMut<usize> for Matrix3<T> {
	/// Index returns column i as vector.
	#[inline]
	fn index_mut(&mut self, i: usize) -> &mut Self::Output {
		&mut self.el[i]
	}
}

impl<T> Neg for Matrix3<T>
where
	T: Neg<Output = T> + Copy,
{
	type Output = Matrix3<T>;
	fn neg(self) -> Matrix3<T> {
		Matrix3 { el: [-self[0], -self[1], -self[2]] }
	}
}

// TODO: unroll loop, do not require Default
impl<T> Mul<Matrix3<T>> for Matrix3<T>
where
	T: Add<T, Output = T> + Mul<T, Output = T> + Copy + Default,
{
	type Output = Matrix3<T>;

	/// Matrix3-Matrix3 multiplication.
	fn mul(self, rhs: Matrix3<T>) -> Matrix3<T> {
		let mut c = Matrix3::default();
		for i in 0..3 {
			for j in 0..3 {
				for k in 0..3 {
					c[i][j] = c[i][j] + rhs[i][k] * self[k][j]
				}
			}
		}
		c
	}
}

impl<T> Mul<Vector3<T>> for Matrix3<T>
where
	T: Add<T, Output = T> + Mul<T, Output = T> + Copy,
{
	type Output = Vector3<T>;

	/// Matrix3-Vector multiplication.
	fn mul(self, rhs: Vector3<T>) -> Self::Output {
		Vector3::new(
			self[0][0] * rhs[0] + self[1][0] * rhs[1] + self[2][0] * rhs[2],
			self[0][1] * rhs[0] + self[1][1] * rhs[1] + self[2][1] * rhs[2],
			self[0][2] * rhs[0] + self[1][2] * rhs[1] + self[2][2] * rhs[2],
		)
	}
}

impl<T> Mul<T> for Matrix3<T>
where
	T: Add<T, Output = T> + Mul<T, Output = T> + Copy,
{
	type Output = Matrix3<T>;

	/// Matrix-scalar multiplication.
	fn mul(self, rhs: T) -> Matrix3<T> {
		Matrix3 {
			el: [self[0] * rhs, self[1] * rhs, self[2] * rhs],
		}
	}
}

/*
impl<T> Matrix3<T>
where
	T: Add<T, Output = T> + Mul<T, Output = T> + Sub<T, Output = T> + Copy + Recip,
{
	/// Inverse matrix.
	pub fn recip(&self) -> Matrix3<T> {
		let a = self[0][0];
		let b = self[1][0];
		let c = self[2][0];
		let d = self[0][1];
		let e = self[1][1];
		let f = self[2][1];
		let g = self[0][2];
		let h = self[1][2];
		let i = self[2][2];

		let a_: T = e * i - f * h;
		let b_: T = f * g - d * i;
		let c_: T = d * h - e * g;
		let inv: Matrix3<T> = Self {
			el: [
				gvec3(e * i - f * h, f * g - d * i, d * h - e * g),
				gvec3(c * h - b * i, a * i - c * g, b * g - a * h),
				gvec3(b * f - c * e, c * d - a * f, a * e - b * d),
			],
		};
		let det: T = a * a_ + b * b_ + c * c_;
		inv * (det.recip())
	}
}
*/

/*
impl<T> fmt::Display for Matrix3<T>
where
	T: fmt::Display,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "[{}, {}, {}]", self[0], self[1], self[2])
	}
}
*/

/*
impl<T> fmt::Debug for mat3<T>
where
	T: fmt::Display,
{
	fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
		write!(f, "[{}, {}, {}]", self[0], self[1], self[2])
	}
}
*/

//#[cfg(test)]
//mod tests {
//	use super::*;
//
//	#[test]
//	fn mul_matrix_matrix() {
//		let theta = 45.0 * DEG;
//		let c = theta.cos();
//		let s = theta.sin();
//		let a = Matrix::from([[c, s, 0.], [-s, c, 0.], [0., 0., 1.]]);
//		//assert_eq!(
//		//	a * a,
//		//	Matrix::from([[0., 1., 0.], [-1., 0., 0.], [0., 0., 1.]])
//		//);
//	}
//
//	#[test]
//	fn mul_matrix_vector() {
//		let theta = 30.0 * DEG;
//		let c = theta.cos();
//		let s = theta.sin();
//
//		let m = Matrix::from([[c, s, 0.], [-s, c, 0.], [0., 0., 1.]]);
//
//		//assert_eq!(m * Vector(1., 0., 0.), Vector(0.866025, 0.500000, 0.000000));
//		//assert_eq!(m * Vector(0., 1., 0.), Vector(-0.50000, 0.866025, 0.000000));
//		//assert_eq!(m * Vector(0., 0., 1.), Vector(0.000000, 0.000000, 1.000000));
//	}
//}

//  func ExampleMatrix_Mul() {
//  	theta := 45 * math.Pi / 180
//  	c := math.Cos(theta)
//  	s := math.Sin(theta)
//  	a := Matrix{{c, s, 0}, {-s, c, 0}, {0, 0, 1}}
//  	fmt.Printf("% 4.1f", a.Mul(&a))
//
//  	//Output:
//  	// [[ 0.0  1.0  0.0] [-1.0  0.0  0.0] [ 0.0  0.0  1.0]]
//  }
//
//  func ExampleMatrix_Mul_2() {
//  	R := Matrix{{0, 1, 0}, {-1, 0, 0}, {0, 0, 1}}
//  	F := Matrix{{-1, 0, 0}, {0, 1, 0}, {0, 0, 1}}
//  	fmt.Printf("% 4.1f\n", R.Mul(&F))
//  	fmt.Printf("% 4.1f\n", F.Mul(&R))
//
//  	//Output:
//  	// [[ 0.0 -1.0  0.0] [-1.0  0.0  0.0] [ 0.0  0.0  1.0]]
//  	// [[ 0.0  1.0  0.0] [ 1.0  0.0  0.0] [ 0.0  0.0  1.0]]
//  }
//
//  func ExampleMatrix_MulVec() {
//  	theta := 30 * math.Pi / 180
//  	c := math.Cos(theta)
//  	s := math.Sin(theta)
//
//  	m := Matrix{{c, s, 0}, {-s, c, 0}, {0, 0, 1}}
//  	fmt.Printf("% 3f\n", m.MulVec(Vec{1, 0, 0}))
//  	fmt.Printf("% 3f\n", m.MulVec(Vec{0, 1, 0}))
//  	fmt.Printf("% 3f\n", m.MulVec(Vec{0, 0, 1}))
//
//  	//Output:
//  	// [ 0.866025  0.500000  0.000000]
//  	// [-0.500000  0.866025  0.000000]
//  	// [ 0.000000  0.000000  1.000000]
//  }
//
//  func ExampleMatrix_Inverse() {
//  	m := Matrix{{1, 2, 3}, {3, -1, 2}, {2, 3, -1}}
//  	inv := m.Inverse()
//  	check := inv.Mul(&m)
//
//  	for i := range check {
//  		for j, v := range check[i] {
//  			if math.Abs(v) < 1e-9 {
//  				check[i][j] = 0
//  			}
//  		}
//  	}
//  	fmt.Printf("% 4.3f", check)
//
//  	//Output:
//  	// [[ 1.000  0.000  0.000] [ 0.000  1.000  0.000] [ 0.000  0.000  1.000]]
//  }
//
