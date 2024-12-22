
use std::collections::HashMap;
use std::path::Path;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use crate::{
  pos::{Pos},
  strutil
};

pub const PLOT_WIDTH: usize = 24;
pub const PLOT_HEIGHT: usize = 12;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Parcel {
	pub owner: Owner,
	pub location: Pos,
	pub art: Vec<String>,
	#[serde(default, rename="linkmask")]
	pub mask: Vec<String>,
	pub links: HashMap<char, String>
}

impl Parcel {
	#[allow(dead_code)]
	pub fn empty(owner: Owner, location: Pos) -> Self {
		Self {
			owner,
			location,
			art: std::iter::repeat_n(" ".repeat(24), 12).collect(),
			mask: std::iter::repeat_n(" ".repeat(24), 12).collect(),
			links: HashMap::new()
		}
	}

	pub fn from_text(text: &str, owner: Owner) -> Result<Self, ParseError> {
		let mut lines = text.lines();
		// first line is the location of the plot: 2 integers separated by whitespace
		let location: Pos = Pos::from_space_separated(lines.next().ok_or(ParseError::EmptyFile)?)
			.ok_or(ParseError::PosLine)?;
		// the next 12 lines are the art that is actually drawn
		// if there are less than 12 lines or less than 24 characters per line then the missing area is filled in with whitespace
		// any characters after 24 are ignored
		let art: Vec<String> = read_plot(&mut lines);
		// If the next line is an empty line, then the 12 lines after that are the mask
		// If the next line is a single dash then the mask is the same as the art
		// If there is no next line it doesn't matter what the mask is since it is not used
		// If the next line is something else then the user made a mistake
		let mask: Vec<String> = match lines.next().map(str::trim) {
			None | Some("-") => art.clone(),
			Some("") => read_plot(&mut lines),
			Some(x) => return Err(ParseError::SeparatorLine(x.to_string()))
		};
		// all remaining lines are link definitions
		// they consist of the key (a single non-whitespace character that should occur in the mask), and a link (separated by whitespace)
		let links = lines
			.map(str::trim)
			.filter(|line| !line.is_empty())
			.map(|line| {
				let (charpart, link) = strutil::split_once_whitespace(line)
					.ok_or(ParseError::LinkLine(line.to_string()))?;
				Ok((
					strutil::to_char(charpart)
						.ok_or(ParseError::LinkLine(charpart.to_string()))?,
					link.to_string()
				))
			})
			.collect::<Result<HashMap<char, String>, ParseError>>()?;
		Ok(Self {owner, location, art, mask, links})
	}

	pub fn text_line(&self, y: usize) -> &str {
		&self.art[y]
	}

	pub fn html_line(&self, y: usize) -> String {
		let mut line = String::with_capacity(PLOT_WIDTH);
		let mut opened = false;
		if y == 0 {
			if let Owner::User(name) = &self.owner {
				opened = true;
				line.push_str(&format!("<span id=\"{}\">", name));
			}
		}
		let mut last_key: Option<char> = None;
		for (ch, mch) in self.art[y].chars().zip(self.mask[y].chars()) {

			if last_key.is_some_and(|k| k != mch) {
				line.push_str("</a>");
				last_key = None;
			}
			if let Some(link) = self.links.get(&mch) {
				if last_key.is_none() {
					line.push_str(&format!("<a href=\"{}\">", link));
					last_key = Some(mch);
				}
			}
			if ch == '<' {
				line.push_str("&lt;");
			} else if ch == '>' {
				line.push_str("&gt;");
			} else if ch == '&' {
				line.push_str("&amp;");
			} else {
				line.push(ch);
			}
		}
		if last_key.is_some() {
			line.push_str("</a>");
		}
		if opened {
			line.push_str("</span>");
		}
		line
	}
}

fn read_plot<'a>(lines: &mut impl Iterator<Item=&'a str>) -> Vec<String> {
	(0..PLOT_HEIGHT)
		.map(|_| strutil::to_length(lines.next().unwrap_or(""), PLOT_WIDTH, ' '))
		.collect::<Vec<String>>()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
	EmptyFile,
	PosLine,
	SeparatorLine(String),
	LinkLine(String)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::hashmap;

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

	#[test]
	fn parse_error_when_empty() {
		assert_eq!(Parcel::from_text("", Owner::Public), Err(ParseError::EmptyFile));
	}

	#[test]
	fn parse_error_when_position_invalid() {
		assert_eq!(Parcel::from_text(" ", Owner::Public), Err(ParseError::PosLine));
		assert_eq!(Parcel::from_text("123", Owner::Public), Err(ParseError::PosLine));
		assert_eq!(Parcel::from_text("a 3", Owner::Public), Err(ParseError::PosLine));
		assert_eq!(Parcel::from_text("5 b", Owner::Public), Err(ParseError::PosLine));
		assert_eq!(Parcel::from_text("10 11 12", Owner::Public), Err(ParseError::PosLine));
	}

	#[test]
	fn parse_parcel_with_mask() {
		let parceltext = r#"0 1
+==()=================+.
| (%&8)  /\       _,__|.
|(&(%)%)/  \    . __,_|.
| (%8%)/_##_\   .     |.
|  ||/ |    |   . @   |.
|  ||  | /\ | * . @   |.
|  ||  |_||_|   .     |.
|        ..  *  .     |.
| (%) O  ........     |.
|        ..    ~troido|.
+=======#  #==========+.
........................

111()111111111111111111.
1 (%&8)  33       _,__1.
1(&(%)%)3333    . __,_1.
1 (%8%)333333   .     1.
1  ||/ 333333   . @   1.
1  ||  333333 * . @   1.
1  ||  333333   .     1.
1        ..  *  . "'` 1.
1 (%) O  ........     1.
1        ..    22222221.
11111111111111111111111.
........................
1 https://tilde.town/~troido/cadastre/
2 https://tilde.town/~troido/index.html
3 https://tilde.town/~troido/entrance.html
		"#;
		let expected = Parcel {
			owner: Owner::user("troido"),
			location: Pos::new(0, 1),
			art: [
				"+==()=================+.",
				"| (%&8)  /\\       _,__|.",
				"|(&(%)%)/  \\    . __,_|.",
				"| (%8%)/_##_\\   .     |.",
				"|  ||/ |    |   . @   |.",
				"|  ||  | /\\ | * . @   |.",
				"|  ||  |_||_|   .     |.",
				"|        ..  *  .     |.",
				"| (%) O  ........     |.",
				"|        ..    ~troido|.",
				"+=======#  #==========+.",
				"........................"
			].map(String::from).to_vec(),
			mask: [
				"111()111111111111111111.",
				"1 (%&8)  33       _,__1.",
				"1(&(%)%)3333    . __,_1.",
				"1 (%8%)333333   .     1.",
				"1  ||/ 333333   . @   1.",
				"1  ||  333333 * . @   1.",
				"1  ||  333333   .     1.",
				"1        ..  *  . \"'` 1.",
				"1 (%) O  ........     1.",
				"1        ..    22222221.",
				"11111111111111111111111.",
				"........................"
			].map(String::from).to_vec(),
			links: hashmap!(
				'1' => "https://tilde.town/~troido/cadastre/".to_string(),
				'2' => "https://tilde.town/~troido/index.html".to_string(),
				'3' => "https://tilde.town/~troido/entrance.html".to_string()
			)
		};
		assert_eq!(Parcel::from_text(parceltext, Owner::user("troido")).unwrap(), expected);
	}

	#[test]
	fn parse_parcel_without_mask() {
		let parceltext = r#"5 1
####################
#  ____            #
# / \__\           #
# |_|__|    ,,,,,  #
#           ,,,,,  #
#    -----  ,,,,,  #
#    -----     _   #
#    -----    (*)  #
# ~johndoe     |   #
##########(%)#######
...........|........
....................
-
# https://example.com
		"#;
		let expected = Parcel {
			owner: Owner::user("johndoe"),
			location: Pos::new(5, 1),
			art: [
				"####################    ",
				"#  ____            #    ",
				"# / \\__\\           #    ",
				"# |_|__|    ,,,,,  #    ",
				"#           ,,,,,  #    ",
				"#    -----  ,,,,,  #    ",
				"#    -----     _   #    ",
				"#    -----    (*)  #    ",
				"# ~johndoe     |   #    ",
				"##########(%)#######    ",
				"...........|........    ",
				"....................    "
			].map(String::from).to_vec(),
			mask: [
				"####################    ",
				"#  ____            #    ",
				"# / \\__\\           #    ",
				"# |_|__|    ,,,,,  #    ",
				"#           ,,,,,  #    ",
				"#    -----  ,,,,,  #    ",
				"#    -----     _   #    ",
				"#    -----    (*)  #    ",
				"# ~johndoe     |   #    ",
				"##########(%)#######    ",
				"...........|........    ",
				"....................    "
			].map(String::from).to_vec(),
			links: hashmap!(
				'#' => "https://example.com".to_string()
			)
		};
		assert_eq!(Parcel::from_text(parceltext, Owner::user("johndoe")).unwrap(), expected);
	}
}
