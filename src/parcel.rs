
use std::fmt;
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
			art: std::iter::repeat(" ".repeat(24)).take(12).collect(),
			mask: std::iter::repeat(" ".repeat(24)).take(12).collect(),
			links: HashMap::new()
		}
	}

	pub fn from_text(text: &str, owner: Owner) -> Result<Self, ParseError> {
		let mut lines = text.lines().enumerate();
		// first line is the location of the plot: 2 integers separated by whitespace
		let (_, first_line) = lines.next().ok_or(ParseError{ kind: ParseErrorKind::EmptyFile, row: 0, line: "".to_string() })?;
		let location: Pos = Pos::from_space_separated(first_line)
			.ok_or(ParseError{ kind: ParseErrorKind::PosLine, row: 0, line: first_line.to_string() })?;
		// the next 12 lines are the art that is actually drawn
		// if there are less than 12 lines or less than 24 characters per line then the missing area is filled in with whitespace
		// any characters after 24 are ignored
		let art: Vec<String> = read_plot(&mut lines);
		// If the separator line is an empty line, then the 12 lines after that are the mask
		// If the separator line is a single dash then the mask is the same as the art
		// If the end of the file has been reached then it doesn't matter what the mask is since it is not used
		// If the separator line is something else then the user made a mistake
		let mask: Vec<String> =
			if let Some((row, line)) = lines.next() {
				match line.trim() {
					"-" => art.clone(),
					"" => read_plot(&mut lines),
					_ => return Err(ParseError{ kind: ParseErrorKind::SeparatorLine, row, line: line.to_string() })
				}
			} else {
				art.clone()
			};
		// all remaining lines are link definitions
		// they consist of the key (a single non-whitespace character that should occur in the mask), and a link (separated by whitespace)
		let mut links: HashMap<char, String> = HashMap::new();
		for (row, line_raw) in lines {
			let line = line_raw.trim();
			if line.is_empty() { continue; }
			let (charpart, link) = strutil::split_once_whitespace(line)
				.ok_or(ParseError{ kind: ParseErrorKind::LinkLine, row, line: line.to_string() })?;
			let key: char = strutil::to_char(charpart)
				.ok_or(ParseError{ kind: ParseErrorKind::LinkLine, row, line: line.to_string() })?;
			links.insert(key, link.to_string());
		}
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

fn read_plot<'a>(lines: &mut impl Iterator<Item=(usize, &'a str)>) -> Vec<String> {
	(0..PLOT_HEIGHT)
		.map(|_| strutil::to_length(lines.next().unwrap_or((0, "")).1, PLOT_WIDTH, ' '))
		.collect::<Vec<String>>()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
	pub kind: ParseErrorKind,
	pub row: usize,
	pub line: String
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseErrorKind {
	EmptyFile,
	PosLine,
	SeparatorLine,
	LinkLine
}
impl fmt::Display for ParseError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let message = match self.kind {
			ParseErrorKind::EmptyFile => "The file is empty",
			ParseErrorKind::PosLine => "The first line must contain to position of the plot as 2 integers separated by a space",
			ParseErrorKind::SeparatorLine => "After the plot there must be a separator line that's either empty or only contains a '-' character",
			ParseErrorKind::LinkLine => "Each line line must start with a key (single character), followed by a space, followed by the link"
		};
		write!(f, "Parse error: {}\n on line {}: \"{}\"", message, self.row + 1, self.line)
	}
}
impl std::error::Error for ParseError {}

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
		assert_eq!(Parcel::from_text("", Owner::Public).unwrap_err().kind, ParseErrorKind::EmptyFile);
	}

	#[test]
	fn parse_error_when_position_invalid() {
		assert_eq!(Parcel::from_text(" ", Owner::Public).unwrap_err().kind, ParseErrorKind::PosLine);
		assert_eq!(Parcel::from_text("123", Owner::Public).unwrap_err().kind, ParseErrorKind::PosLine);
		assert_eq!(Parcel::from_text("a 3", Owner::Public).unwrap_err().kind, ParseErrorKind::PosLine);
		assert_eq!(Parcel::from_text("5 b", Owner::Public).unwrap_err().kind, ParseErrorKind::PosLine);
		assert_eq!(Parcel::from_text("10 11 12", Owner::Public).unwrap_err().kind, ParseErrorKind::PosLine);
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
