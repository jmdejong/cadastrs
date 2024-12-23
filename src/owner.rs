
use std::path::Path;
use serde::{Serialize, Deserialize, Serializer, Deserializer};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Owner {
	Admin,
	User(String),
	Public
}

impl Owner {
	pub fn priority(&self) -> i32 {
		match self {
			Self::Admin => 3,
			Self::User(_) => 2,
			Self::Public => 1
		}
	}
	pub fn user(name: &str) -> Self {
		Self::User(name.to_string())
	}
	pub fn from_homedir(homedir: &Path) -> Option<Self> {
		Some(Self::user(homedir.file_name()?.to_str()?))
	}
}


impl Serialize for Owner {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		match self {
			Self::Admin => "@_admin".serialize(serializer),
			Self::User(name) => name.serialize(serializer),
			Self::Public => ().serialize(serializer)
		}
	}
}
impl<'de> Deserialize<'de> for Owner {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where D: Deserializer<'de> {
		Ok(match <Option<&str>>::deserialize(deserializer)? {
			None => Self::Public,
			Some("@_admin") => Self::Admin,
			Some(name) => Self::user(name)
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn serialize_owner(){
		assert_eq!(serde_json::json!(Owner::Admin).to_string(), "\"@_admin\"");
		assert_eq!(serde_json::json!(Owner::user("troido")).to_string(), "\"troido\"");
		assert_eq!(serde_json::json!(Owner::Public).to_string(), "null");
	}

	#[test]
	fn deserialize_owner(){
		assert!(serde_json::from_str::<Owner>("{}").is_err());
		assert!(serde_json::from_str::<Owner>("3").is_err());
		assert!(serde_json::from_str::<Owner>("\"").is_err());
		assert_eq!(serde_json::from_str::<Owner>("\"@_admin\"").unwrap(), Owner::Admin);
		assert_eq!(serde_json::from_str::<Owner>("\"troido\"").unwrap(), Owner::user("troido"));
		assert_eq!(serde_json::from_str::<Owner>("null").unwrap(), Owner::Public);
	}
}
