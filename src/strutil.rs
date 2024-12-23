


pub fn split_once_whitespace(txt: &str) -> Option<(&str, &str)> {
	let (head, rest) = txt.trim().split_once(" ")?;
	Some((head.trim(), rest.trim()))
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
