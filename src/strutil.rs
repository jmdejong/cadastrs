

pub fn to_length(txt: &str, length: usize, padding: char) -> String {
	if txt.len() < length {
		format!("{}{}", txt, String::from(padding).repeat(length - txt.len()))
	} else {
		let mut out: String = txt.to_string();
		out.truncate(length);
		out
	}
}

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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn truncate_long_str() {
		assert_eq!(to_length("hello_world", 8, ' '), "hello_wo".to_string());
	}

	#[test]
	fn pad_short_str() {
		assert_eq!(to_length("hi", 8, ' '), "hi      ".to_string());
	}
}
