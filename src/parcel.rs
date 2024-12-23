
use std::fmt;
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use lazy_static::lazy_static;
use crate::{
  pos::Pos,
  strutil,
  owner::Owner
};

pub const PLOT_WIDTH: usize = 24;
pub const PLOT_HEIGHT: usize = 12;
lazy_static! {
	static ref allowed_characters: HashSet<char> = " !\"#$%&\'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~¥¨°²´·¿×ōπᓚᗢᘏ†•…‾∞≈≡⊞─│┌┏┐┓└┗┘┛├┣┤┫┬┳┴┻┼╂═║╒╔╕╗╘╚╛╜╝╟╠╢╣╤╥╦╧╩╫╭╮╰╱╲╿▀▁▂▃▄█▉▊▌▎▐░▒▓▔▙▛▜▟▪►◄◊◘◠☆☺♠♥♪♫♯⚵⚶⛭✥✽❀➅➐⠀⠁⠃⠈⠋⠘⠙⠛⠞⠟⠳⠺⠾⡀⡇⡞⡤⢀⢇⢠⢤⢦⢩⢫⢸⢹⢻⢾⢿⣀⣄⣆⣠⣤⣬⣯⣳⣴⣷⣻⣼⣽⣿".chars().collect();
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
		// If the separator line is something else then this and all following lines should be ignored
		let mask: Vec<String> =
			if let Some((_row, line)) = lines.next() {
				match line.trim() {
					"-" => art.clone(),
					"" => read_plot(&mut lines),
					_ => {
						lines = "".lines().enumerate(); // don't read any more lines
						art.clone()
					}
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
				// The first line of a plot should have the username as anchor
				opened = true;
				line.push_str(&format!("<span id=\"{}\">", name));
			}
		}
		let mut active_key: Option<char> = None;
		for (ch, mch) in self.art[y].chars().zip(self.mask[y].chars()) {
			// if the last char had a link and this one does not or has a different link, then close it
			if active_key.is_some_and(|k| k != mch) {
				line.push_str("</a>");
				active_key = None;
			}
			// if no link is active and this char has a link, then open the link
			if let Some(link) = self.links.get(&mch) {
				if active_key.is_none() {
					line.push_str(&format!("<a href=\"{}\">", link.replace('"', "&quot;")));
					active_key = Some(mch);
				}
			}
			// replace html unsafe characters
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
		if active_key.is_some() {
			line.push_str("</a>");
		}
		if opened {
			line.push_str("</span>");
		}
		line
	}
}


fn process_plot_line(txt: &str, length: usize) -> String {
	String::from_iter(
		txt.chars()
			.chain(std::iter::repeat(' '))
			.take(length)
			.map(|ch| if allowed_characters.contains(&ch) { ch } else { '?' })
	)
}

fn read_plot<'a>(lines: &mut impl Iterator<Item=(usize, &'a str)>) -> Vec<String> {
	(0..PLOT_HEIGHT)
		.map(|_| process_plot_line(lines.next().unwrap_or((0, "")).1, PLOT_WIDTH))
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
	fn truncate_long_str() {
		assert_eq!(process_plot_line("hello_world", 8), "hello_wo".to_string());
	}

	#[test]
	fn pad_short_str() {
		assert_eq!(process_plot_line("hi", 8), "hi      ".to_string());
	}

	#[test]
	fn truncate_unicode() {
		assert_eq!(process_plot_line("║╒╔╕╗", 1), "║".to_string());
		assert_eq!(process_plot_line("║╒╔╕╗", 2), "║╒".to_string());
		assert_eq!(process_plot_line("║╒╔╕╗", 3), "║╒╔".to_string());
		assert_eq!(process_plot_line("║╒╔╕╗", 4), "║╒╔╕".to_string());
		assert_eq!(process_plot_line("║╒╔╕╗", 5), "║╒╔╕╗".to_string());
	}

	#[test]
	fn replace_disallowed_characters() {
		assert_eq!(process_plot_line("|..👻.◫.|", 8), "|..?.?.|".to_string());
	}

	#[test]
	fn parse_smaller_parcel() {
		let parceltext = r#" 0  1
1234567890
1234567890123456789012345678901234567890
31
a
z"#;
		let parcel: Parcel = Parcel::from_text(parceltext, Owner::user("troido")).unwrap();
		assert_eq!(parcel.art, vec![
			"1234567890              ",
			"123456789012345678901234",
			"31                      ",
			"a                       ",
			"z                       ",
			"                        ",
			"                        ",
			"                        ",
			"                        ",
			"                        ",
			"                        ",
			"                        "
		]);
		assert_eq!(parcel.links, HashMap::new());
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

	#[test]
	fn parse_larger_parcel() {
		let parceltext = r#" 0  1
 ___________
< Cadastre! >
 -----------
\                             .       .
 \                           / `.   .' "
  \                  .---.  <    > <    >  .---.
   \                 |    \  \ - ~ ~ - /  /    |
         _____          ..-~             ~-..-~
        |     |   \~~~\.'                    `./~~~/
       ---------   \__/                        \__/
      .'  O    \     /               /       \  "
     (_____,    `._.'               |         }  \/~~~/
      `----.          /       }     |        /    \__/
            `-.      |       /      |       /      `. ,~~|
                ~-.__|      /_ - ~ ^|      /- _      `..-'
                     |     /        |     /     ~-.     `-. _  _  _
                     |_____|        |_____|         ~ - . _ _ _ _ _>
"#;
		let parcel: Parcel = Parcel::from_text(parceltext, Owner::user("troido")).unwrap();
		assert_eq!(parcel.art, vec![
			" ___________            ",
			"< Cadastre! >           ",
			" -----------            ",
			"\\                       ",
			" \\                      ",
			"  \\                  .--",
			"   \\                 |  ",
			"         _____          ",
			"        |     |   \\~~~\\.",
			"       ---------   \\__/ ",
			"      .'  O    \\     /  ",
			"     (_____,    `._.'   "
		]);
		assert_eq!(parcel.links, HashMap::new());
	}

	#[test]
	fn parse_js_in_link() {
				let parceltext = r#"2 2
........................
........................
.........?????..........
........................
........................
........................
....!!!!!!!!............
........................
........................
........................
........................
........................
-
 ?   https://en.wikipedia.org
! javascript:(function(){ console.log("<hello> " + '"world"'); })()
"#;
		let parcel: Parcel = Parcel::from_text(parceltext, Owner::user("troido")).unwrap();
		assert_eq!(parcel.links, hashmap!(
			'?' => "https://en.wikipedia.org".to_string(),
			'!' => r#"javascript:(function(){ console.log("<hello> " + '"world"'); })()"#.to_string()
		));
		assert_eq!(parcel.html_line(6), r#"....<a href="javascript:(function(){ console.log(&quot;<hello> &quot; + '&quot;world&quot;'); })()">!!!!!!!!</a>............"#.to_string());
	}
}
