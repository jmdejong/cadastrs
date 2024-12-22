
#[macro_export]
macro_rules! hashmap {
	( $($key:expr => $value:expr ),* ) => {{
		#[allow(unused_mut)]
		let mut h = std::collections::HashMap::new();
		$(
			h.insert($key, $value);
		)*
		h
	}}
}

