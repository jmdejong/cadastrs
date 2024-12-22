
use serde::{Serialize, Deserialize};
use crate::pos::Pos;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Background(pub u64);


fn hash(num: u64) -> u64{
	num.wrapping_mul(104399)
		.wrapping_add(617)
}

impl Background {
	pub fn next(&self) -> Self {
		Self(self.0.wrapping_mul(211).wrapping_add(53) & 0xffffffff)
	}

	pub fn char_at(&self, pos: Pos) -> &str {
		let chars = "'',,..``\"";
		let h = ((hash(hash(hash(self.0) ^ pos.x as u64) ^ pos.y as u64) >> 8) % 128) as usize;
		if h < chars.len() {
			&chars[h..h+1]
		} else {
			" "
		}
	}
}
