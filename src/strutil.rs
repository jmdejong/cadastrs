


pub fn split_once_whitespace(txt: &str) -> Option<(&str, &str)> {
	let parts: Vec<&str> = txt.split_whitespace().collect();
	if parts.len() == 2 {
		Some((parts[0], parts[1]))
	} else {
		None
	}
}

pub fn to_char(txt: &str) -> Option<char> {
	let mut chars = txt.chars();
	let ch = chars.next()?;
	if chars.next() == None {
		Some(ch)
	} else {
		None
	}
}
