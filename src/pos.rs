
use std::ops::{Add, Sub, Neg, Mul, Div, Rem, AddAssign};
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use crate::strutil;


#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Default)]
pub struct Pos {
	pub x: i64,
	pub y: i64
}

#[allow(dead_code)]
impl Pos {

	pub const fn new(x: i64, y: i64) -> Pos {
		Pos {x, y}
	}

	pub fn zero() -> Pos {
		Pos {x: 0, y: 0}
	}

	pub fn from_tuple(p: (i64, i64)) -> Pos {
		let (x, y) = p;
		Pos {x, y}
	}

	pub fn from_space_separated(input: &str) -> Option<Self> {
		let (xs, ys) = strutil::split_once_whitespace(input)?;
		Some(Pos{
			x: xs.parse::<i64>().ok()?,
			y: ys.parse::<i64>().ok()?
		})
	}

	pub fn abs(&self) -> Pos {
		Pos{x: self.x.abs(), y: self.y.abs()}
	}

	pub fn max(&self) -> i64 {
		if self.x > self.y {
			self.x
		} else {
			self.y
		}
	}

	pub fn min(&self) -> i64 {
		if self.x < self.y {
			self.x
		} else {
			self.y
		}
	}

	pub fn size(&self) -> i64{
		self.x.abs() + self.y.abs()
	}

	#[allow(dead_code)]
	pub fn is_zero(&self) -> bool {
		self.x == 0 && self.y == 0
	}

	pub fn distance_to(&self, other: Pos) -> i64 {
		(other - *self).size()
	}

	pub fn normalize(&self) -> Self {
		Self { x: self.x.signum(), y: self.y.signum() }
	}
}

impl Add<Pos> for Pos {
	type Output = Pos;
	fn add(self, other: Pos) -> Pos {
		Pos {
			x: self.x + other.x,
			y: self.y + other.y
		}
	}
}

impl Add<(i64, i64)> for Pos {
	type Output = Pos;
	fn add(self, other: (i64, i64)) -> Pos {
		Pos {
			x: self.x + other.0,
			y: self.y + other.1
		}
	}
}

impl Sub<Pos> for Pos {
	type Output = Pos;
	fn sub(self, other: Pos) -> Pos {
		Pos {
			x: self.x - other.x,
			y: self.y - other.y
		}
	}
}

impl Neg for Pos {
    type Output = Pos;
    fn neg(self) -> Pos {
		Pos {x: -self.x, y: -self.y}
    }
}

impl Mul<i64> for Pos {
	type Output = Pos;
	fn mul(self, n: i64) -> Pos {
		Pos {
			x: self.x * n,
			y: self.y * n
		}
	}
}

impl Div<i64> for Pos {
	type Output = Pos;
	fn div(self, n: i64) -> Pos {
		Pos {
			x: self.x.div_euclid(n),
			y: self.y.div_euclid(n)
		}
	}
}


impl Rem<i64> for Pos {
	type Output = Pos;
	fn rem(self, n: i64) -> Pos {
		Pos {
			x: self.x.rem_euclid(n),
			y: self.y.rem_euclid(n)
		}
	}
}

impl AddAssign for Pos {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            x: self.x + other.x,
            y: self.y + other.y,
        };
    }
}


impl Serialize for Pos {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		(self.x, self.y).serialize(serializer)
	}
}
impl<'de> Deserialize<'de> for Pos {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where D: Deserializer<'de> {
		let (x, y) = <(i64, i64)>::deserialize(deserializer)?;
		Ok(Self{x, y})
	}
}




#[cfg(test)]
mod tests {
	use super::*;
	#[test]
	fn division_rounds_to_negative_infinity() {
		assert_eq!(Pos::new(-3, -3) / 2, Pos::new(-2, -2));
	}
}
