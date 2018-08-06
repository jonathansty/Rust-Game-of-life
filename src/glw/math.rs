use std::ops::{Mul, Div, Add};

pub trait DotProduct<RHS=Self>{
    type Output;

	#[must_use]
	fn dot(&self, rhs : RHS ) -> Self::Output;
}
#[derive(Default,Clone, PartialEq, Debug)]
pub struct Vec2<T>{
    pub x : T,
    pub y : T,
}

impl<T> Vec2<T>
{
	#[allow(dead_code)]
	pub fn new(x: T, y: T) -> Self{
		Vec2{
			x,
			y,
		}
	}
}

impl<T> Mul for Vec2<T>
	where T: Mul<Output=T> 
{
	type Output = Self;
	fn mul(self : Self, rhs: Self) -> Self {

		Vec2{
			x: self.x * rhs.x,
			y: self.y * rhs.y,
		}
	}
}

impl<T> Add for Vec2<T>
	where T: Add<Output=T> 
{
	type Output = Self;
	fn add(self : Self, rhs: Self) -> Self {
		Vec2{
			x: self.x + rhs.x,
			y: self.y + rhs.y,
		}
	}
}

impl<T> Div for Vec2<T>
	where T: Div<Output=T>
{
	type Output = Self;
	fn div(self: Self, rhs: Self) -> Self{
		Vec2{
			x: self.x / rhs.x,
			y: self.y / rhs.y,
		}
	}
}


#[derive(Default, PartialEq, Clone, Debug)]
pub struct Vec3<T>{
	pub x : T,
	pub y : T,
	pub z : T,
}

impl<T>  Vec3<T> {
	#[allow(dead_code)]
	pub fn new(x: T, y: T, z: T) -> Self{
		Vec3{
			x,
			y,
			z,
		}
	}
}
impl<T> DotProduct for Vec3<T>
	where T: Mul<Output=T> + Add<Output=T> + Copy
{
	type Output = T;
	fn dot(&self, rhs : Self) -> T{
		self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
	}
}

#[cfg(test)]
mod vec2_tests
{
	use super::*;

	#[test]
	fn create_default(){
		let a : Vec2<f32> = Vec2::default();
		assert_eq!(a, Vec2{x:0.0,y:0.0})
	}

	#[test]
	fn create_vector_init_to_zero(){
		let a = Vec2::new(0,0);
		assert_eq!(a.x,0);
		assert_eq!(a.y,0);
	}

	#[test]
	fn vec_multiply_to_zero(){
		let a = Vec2::new(1, 15);
		let b = Vec2::new(0, 0);

		let result = a * b;
		assert_eq!(result, Vec2{x: 0, y: 0});
	}
}

#[cfg(test)]
mod vec3_tests
{
	use super::*;

	#[test]
	fn create_default(){
		let a : Vec3<f32> = Vec3::default();
		assert_eq!(a,Vec3{x: 0.0, y:0.0, z:0.0});
	}

	#[test]
	fn create_zero_vector(){
		let a = Vec3::new(0,0,0);

		assert_eq!(a.x,0);
		assert_eq!(a.y,0);
		assert_eq!(a.z,0);
	}

	#[test]
	fn dot_product_ortho(){
		let a = Vec3::new(1.0,0.0,0.0);
		let b = Vec3::new(0.0,1.0,0.0);

		let dot_product = a.dot(b);
		assert_eq!(dot_product,0.0);
	}

	#[test]
	fn dot_product_same(){
		let a = Vec3::new(2.0,0.0,0.0);
		let b = Vec3::new(2.0,0.0,0.0);

		let dot_product = a.dot(b);
		assert_eq!(dot_product,4.0);
	}
}