
use std::cmp::Ordering;
use std::collections::HashMap;
use serde::{de, Serialize, Deserialize, Serializer, Deserializer};
use crate::{
  pos::Pos,
  parcel::{Parcel, Owner, PLOT_WIDTH, PLOT_HEIGHT},
  background::Background
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cadastre {
	// seed: u64,
	places: HashMap<PosKey, Parcel>,
	#[serde(rename="seed")]
	background: Background
}

impl Cadastre {
	pub fn empty() -> Self {
		Self { places: HashMap::new(), background: Background(1) }
	}

	pub fn build(old: &Self, parcels: impl Iterator<Item=Parcel>) -> Self {
		let mut places: HashMap<PosKey, Parcel> = HashMap::new();
		for parcel in parcels {
			// When multiple plots are trying to claim the same space, the owner with the highest priority should win
			// Admins have highest priority, then users, then public plots
			// If the priority is equal, then the one who held the plot previously will get this
			// If neither owner held the plot previously, then it doesn't matter
			// If both the existing parcel and the new parcel have the same owner (eg. public), then it doesn't matter either
			let key = PosKey::from_pos(parcel.location);
			let can_claim: bool =
				if let Some(conflict) = places.get(&key) {
					match parcel.owner.priority().cmp(&conflict.owner.priority()) {
						Ordering::Greater => true,
						Ordering::Equal => old.owner_of(parcel.location).is_some_and(|owner| owner == parcel.owner),
						Ordering::Less => false
					}
				} else {
					true
				};
			if can_claim {
				places.insert(key, parcel);
			}
		}
		Self {
			places,
			background: old.background.next()
		}
	}

	fn parcel(&self, pos: Pos) -> Option<&Parcel> {
		self.places.get(&PosKey::from_pos(pos))
	}

	fn owner_of(&self, pos: Pos) -> Option<Owner> {
		self.parcel(pos).map(|parcel| parcel.owner.clone())
	}

	pub fn render_text<F>(&self, width: usize, height: usize, mut writer: F) //-> impl Iterator<Item = String> + use<'_>{
			where F: FnMut(&str) {
		for y in 0..(height * PLOT_HEIGHT) {
			let plot_y = y as i64 / PLOT_HEIGHT as i64;
			let inner_y = y as usize % PLOT_HEIGHT;
			for plot_x in 0..width {
				if let Some(parcel) = self.parcel(Pos::new(plot_x as i64, plot_y)) {
					writer(parcel.text_line(inner_y));
				} else {
					for x in (PLOT_WIDTH*plot_x)..(PLOT_WIDTH*(plot_x+1)) {
						writer(self.background.char_at(Pos::new(x as i64, y as i64)));
					}
				}
			}
			writer("\n")
		}
	}

	pub fn render_html<F>(&self, width: usize, height: usize, mut writer: F) //-> impl Iterator<Item = String> + use<'_>{
			where F: FnMut(&str) {
		writer("<!DOCTYPE html>\n<html>\n<!-- See tilde.town/~troido/cadastre for instructions -->\n<head>\n<meta charset='utf-8'>\n<style>\na {text-decoration: none}\n</style>\n</head>\n<body><pre>\n");
		for y in 0..(height * PLOT_HEIGHT) {
			let plot_y = y as i64 / PLOT_HEIGHT as i64;
			let inner_y = y as usize % PLOT_HEIGHT;
			for plot_x in 0..width {
				if inner_y == 0 {
					writer(&format!("<span id=\"{},{}\"></span>", plot_x, plot_y));
				}
				if let Some(parcel) = self.parcel(Pos::new(plot_x as i64, plot_y)) {
					writer(&parcel.html_line(inner_y));
				} else {
					for x in (PLOT_WIDTH*plot_x)..(PLOT_WIDTH*(plot_x+1)) {
						writer(self.background.char_at(Pos::new(x as i64, y as i64)));
					}
				}
			}
			writer("\n");
		}
		writer("</pre></body>\n<!-- Cadastre made by ~troido; art by tilde.town users -->\n</html>\n");
	}
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct PosKey(Pos);

impl PosKey {
	#[allow(dead_code)]
	pub fn new(x: i64, y: i64) -> Self {
		Self(Pos::new(x, y))
	}

	pub fn from_pos(pos: Pos) -> Self {
		Self(pos)
	}
}

impl Serialize for PosKey {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: Serializer {
		format!("{},{}", self.0.x, self.0.y).serialize(serializer)
	}
}
impl<'de> Deserialize<'de> for PosKey {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where D: Deserializer<'de> {
		let s = <String>::deserialize(deserializer)?;
		let (x, y) = s.split_once(",").ok_or(de::Error::custom("Missing comma"))?;
		Ok(Self(Pos::new(
			x.parse().map_err(de::Error::custom)?,
			y.parse().map_err(de::Error::custom)?
		)))
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		hashmap,
		parcel::Owner
	};

	#[test]
	fn serialize_poskey_to_and_from_string() {
		assert_eq!(serde_json::json!(PosKey::new(5, -3)).to_string(), "\"5,-3\"");
		assert_eq!(serde_json::from_str::<PosKey>("\"-412,800\"").unwrap(), PosKey::new(-412,800));
	}

	fn some_cadastre() -> Cadastre {
		Cadastre::build(&Cadastre::empty(), vec![
			Parcel::empty(Owner::user("troido"), Pos::new(2, 3)),
			Parcel::empty(Owner::user("odiort"), Pos::new(3, 2)),
			Parcel::empty(Owner::Public, Pos::new(3, 3)),
			Parcel::empty(Owner::Admin, Pos::new(2, 2)),
		].into_iter())
	}

	#[test]
	fn can_reclaim_unclaimed_plots() {
		let cadastre = Cadastre::build(&some_cadastre(), vec![
			Parcel::empty(Owner::user("troido"), Pos::new(3, 2)),
			Parcel::empty(Owner::user("odiort"), Pos::new(2, 3)),
			Parcel::empty(Owner::Public, Pos::new(2, 2)),
		].into_iter());
		assert_eq!(cadastre.owner_of(Pos::new(3, 2)), Some(Owner::user("troido")));
		assert_eq!(cadastre.owner_of(Pos::new(2, 3)), Some(Owner::user("odiort")));
		assert_eq!(cadastre.owner_of(Pos::new(2, 2)), Some(Owner::Public));
	}

	#[test]
	fn tenancy_decides_between_users() {
		let cadastre = Cadastre::build(&some_cadastre(), vec![
			Parcel::empty(Owner::user("john"), Pos::new(2, 3)),
			Parcel::empty(Owner::user("jack"), Pos::new(3, 2)),
			Parcel::empty(Owner::user("troido"), Pos::new(2, 3)),
			Parcel::empty(Owner::user("odiort"), Pos::new(3, 2)),
			Parcel::empty(Owner::user("joe"), Pos::new(2, 3)),
			Parcel::empty(Owner::user("josh"), Pos::new(3, 2)),
		].into_iter());
		assert_eq!(cadastre.owner_of(Pos::new(2, 3)), Some(Owner::user("troido")));
		assert_eq!(cadastre.owner_of(Pos::new(3, 2)), Some(Owner::user("odiort")));
	}


	#[test]
	fn priority_overrides_all() {
		let cadastre = Cadastre::build(&some_cadastre(), vec![
			Parcel::empty(Owner::user("troido"), Pos::new(2, 3)),
			Parcel::empty(Owner::Public, Pos::new(3, 3)),
			Parcel::empty(Owner::user("odiort"), Pos::new(3, 3)),
			Parcel::empty(Owner::Admin, Pos::new(2, 3)),
			Parcel::empty(Owner::user("troido"), Pos::new(2, 3)),
			Parcel::empty(Owner::Public, Pos::new(3, 3)),
		].into_iter());
		assert_eq!(cadastre.owner_of(Pos::new(2, 3)), Some(Owner::Admin));
		assert_eq!(cadastre.owner_of(Pos::new(3, 3)), Some(Owner::user("odiort")));
	}

	#[test]
	fn render_text() {
		let mut text = String::new();
		let cadastre = little_town();
		cadastre.render_text(2, 2, |line| text.push_str(line));
		// println!("{}", text);
		let expected = r#"+------.................                       .
|      |               .                       .
 . |      |           . __                     .
..|          |         . ~\________            .
|              |       ._   ~  ~   \_ {%%}     .
|                 |   .  \_______~<><{%%%%}    .
|     feels         |.           \   ~{%%}     .
|       must          |           \><>!||      .
|         flow         |          |~  !||      .
|         _            |           \  ~ `\     .
|      ---  -_         |            \__   \    .
+------ .......--------π               \~  |   .
+==()=================+.╔══════════════════════╗
| (%&8)  /\       _,__|.║ Tilde.town Cadastre  ║
|(&(%)%)/  \    . __,_|.║                      ║
| (%8%)/_##_\   .     |.║ Any tilde.town user  ║
|  ||/ |    |   . @   |.║ can claim a parcel   ║
|  ||  | /\ | * . @   |.║ of land to show some ║
|  ||  |_||_|   .     |.║ awesome ascii art    ║
|        ..  *  . "'` |.║                      ║
| (%) O  ........     |.║ * Instructions       ║
|        ..    ~troido|.║ * source (github)    ║
+=======#  #==========+.║      Made by ~troido ║
........................╚══════════════════════╝
"#;
		compare_text(&text, expected);
	}

	#[test]
	fn render_html() {
		let mut text = String::new();
		let cadastre = little_town();
		cadastre.render_html(2, 2, |line| text.push_str(line));
		// println!("{}", text);
		let expected = r#"<!DOCTYPE html>
<html>
<!-- See tilde.town/~troido/cadastre for instructions -->
<head>
<meta charset='utf-8'>
<style>
a {text-decoration: none}
</style>
</head>
<body><pre>
<span id="0,0"></span><span id="vilmibm">+------.................</span><span id="1,0"></span>                       .
|      |               .                       .
 . |      |           . __                     .
..|          |         . ~\________            .
|              |       ._   ~  ~   \_ {%%}     .
|                 |   .  \_______~&lt;&gt;&lt;{%%%%}    .
|     <a href="https://tilde.town/~vilmibm">feels</a>         |.           \   ~{%%}     .
|       <a href="https://tilde.town/~vilmibm">must</a>          |           \&gt;&lt;&gt;!||      .
|         <a href="https://tilde.town/~vilmibm">flow</a>         |          |~  !||      .
|         _            |           \  ~ `\     .
|      ---  -_         |            \__   \    .
+------ .......--------<a href="https://libraryofbabel.info/random.cgi">π</a>               \~  |   .
<span id="0,1"></span><span id="troido"><a href="https://tilde.town/~troido/cadastre/">+==</a>()<a href="https://tilde.town/~troido/cadastre/">=================+</a>.</span><span id="1,1"></span>╔══════════════════════╗
<a href="https://tilde.town/~troido/cadastre/">|</a> (%&amp;8)  <a href="https://tilde.town/~troido/entrance.html">/\</a>       _,__<a href="https://tilde.town/~troido/cadastre/">|</a>.║ Tilde.town Cadastre  ║
<a href="https://tilde.town/~troido/cadastre/">|</a>(&amp;(%)%)<a href="https://tilde.town/~troido/entrance.html">/  \</a>    . __,_<a href="https://tilde.town/~troido/cadastre/">|</a>.║                      ║
<a href="https://tilde.town/~troido/cadastre/">|</a> (%8%)<a href="https://tilde.town/~troido/entrance.html">/_##_\</a>   .     <a href="https://tilde.town/~troido/cadastre/">|</a>.║ Any tilde.town user  ║
<a href="https://tilde.town/~troido/cadastre/">|</a>  ||/ <a href="https://tilde.town/~troido/entrance.html">|    |</a>   . @   <a href="https://tilde.town/~troido/cadastre/">|</a>.║ can claim a parcel   ║
<a href="https://tilde.town/~troido/cadastre/">|</a>  ||  <a href="https://tilde.town/~troido/entrance.html">| /\ |</a> * . @   <a href="https://tilde.town/~troido/cadastre/">|</a>.║ of land to show some ║
<a href="https://tilde.town/~troido/cadastre/">|</a>  ||  <a href="https://tilde.town/~troido/entrance.html">|_||_|</a>   .     <a href="https://tilde.town/~troido/cadastre/">|</a>.║ awesome ascii art    ║
<a href="https://tilde.town/~troido/cadastre/">|</a>        ..  *  . "'` <a href="https://tilde.town/~troido/cadastre/">|</a>.║                      ║
<a href="https://tilde.town/~troido/cadastre/">|</a> (%) O  ........     <a href="https://tilde.town/~troido/cadastre/">|</a>.║ * <a href="https://tilde.town/~troido/cadastre">Instructions</a>       ║
<a href="https://tilde.town/~troido/cadastre/">|</a>        ..    <a href="https://tilde.town/~troido/index.html">~troido</a><a href="https://tilde.town/~troido/cadastre/">|</a>.║ * <a href="https://github.com/jmdejong/cadastre">source (github)</a>    ║
<a href="https://tilde.town/~troido/cadastre/">+=======#  #==========+</a>.║      Made by <a href="https://tilde.town/~troido/index.html">~troido</a> ║
........................╚══════════════════════╝
</pre></body>
<!-- Cadastre made by ~troido; art by tilde.town users -->
</html>
"#;
		compare_text(&text, expected);
	}

	fn little_town() -> Cadastre {
		Cadastre {
			background: Background(8138474425133413201),
			places: hashmap!(
				PosKey::new(0, 0) => Parcel {
					owner: Owner::user("vilmibm"),
					location: Pos::new(0, 0),
					art: [
						"+------.................",
						"|      |               .",
						" . |      |           . ",
						"..|          |         .",
						"|              |       .",
						"|                 |   . ",
						"|     feels         |.  ",
						"|       must          | ",
						"|         flow         |",
						"|         _            |",
						"|      ---  -_         |",
						"+------ .......--------π"
					].map(String::from).to_vec(),
					mask: [
						"+------.................",
						"|      |               .",
						" . |      |           . ",
						"..|          |         .",
						"|              |       .",
						"|                 |   . ",
						"|     11111         |.  ",
						"|       1111          | ",
						"|         1111         |",
						"|         _            |",
						"|      ---  -_         |",
						"+------ .......--------2"
					].map(String::from).to_vec(),
					links: hashmap!(
						'1' => "https://tilde.town/~vilmibm".to_string(),
						'2' => "https://libraryofbabel.info/random.cgi".to_string()
					)
				},
				PosKey::new(0, 1) => Parcel {
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
						"|        ..  *  . \"'` |.",
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
				},
				PosKey::new(1, 1) => Parcel {
					owner: Owner::Admin,
					location: Pos::new(1, 1),
					art: [
						"╔══════════════════════╗",
						"║ Tilde.town Cadastre  ║",
						"║                      ║",
						"║ Any tilde.town user  ║",
						"║ can claim a parcel   ║",
						"║ of land to show some ║",
						"║ awesome ascii art    ║",
						"║                      ║",
						"║ * Instructions       ║",
						"║ * source (github)    ║",
						"║      Made by ~troido ║",
						"╚══════════════════════╝"
					].map(String::from).to_vec(),
					mask: [
						"~~~~~~~~~~~~~~~~~~~~~~~~",
						"~ Tilde.town Cadastre  ~",
						"~                      ~",
						"~ Any tilde.town user  ~",
						"~ can claim a parcel   ~",
						"~ of land to show some ~",
						"~ awesome ascii art    ~",
						"~                      ~",
						"~ * 111111111111       ~",
						"~ * 222222222222222    ~",
						"~      Made by 3333333 ~",
						"~~~~~~~~~~~~~~~~~~~~~~~~"
					].map(String::from).to_vec(),
					links: hashmap!(
						'1' => "https://tilde.town/~troido/cadastre".to_string(),
						'2' => "https://github.com/jmdejong/cadastre".to_string(),
						'3' => "https://tilde.town/~troido/index.html".to_string()
					)
				},
				PosKey::new(1, 0) => Parcel {
					owner: Owner::Public,
					location: Pos::new(1, 0),
					art: [
						"                       .",
						"                       .",
						"__                     .",
						" ~\\________            .",
						"_   ~  ~   \\_ {%%}     .",
						" \\_______~<><{%%%%}    .",
						"         \\   ~{%%}     .",
						"          \\><>!||      .",
						"          |~  !||      .",
						"           \\  ~ `\\     .",
						"            \\__   \\    .",
						"               \\~  |   ."
					].map(String::from).to_vec(),
					mask: [
						"                        ",
						"                        ",
						"                        ",
						"                        ",
						"                        ",
						"                        ",
						"                        ",
						"                        ",
						"                        ",
						"                        ",
						"                        ",
						"                        "
					].map(String::from).to_vec(),
					links: HashMap::new()
				}
			)
		}
	}

	#[test]
	fn deserialize_town_from_json() {
		let town = r#"
{
	"places": {
		"0,0": {
			"owner": "vilmibm",
			"location": [0, 0],
			"art": [
				"+------.................",
				"|      |               .",
				" . |      |           . ",
				"..|          |         .",
				"|              |       .",
				"|                 |   . ",
				"|     feels         |.  ",
				"|       must          | ",
				"|         flow         |",
				"|         _            |",
				"|      ---  -_         |",
				"+------ .......--------π"
			],
			"linkmask": [
				"+------.................",
				"|      |               .",
				" . |      |           . ",
				"..|          |         .",
				"|              |       .",
				"|                 |   . ",
				"|     11111         |.  ",
				"|       1111          | ",
				"|         1111         |",
				"|         _            |",
				"|      ---  -_         |",
				"+------ .......--------2"
			],
			"links": {
				"1": "https://tilde.town/~vilmibm",
				"2": "https://libraryofbabel.info/random.cgi"
			}
		},
		"0,1": {
			"owner": "troido",
			"location": [0, 1],
			"art": [
				"+==()=================+.",
				"| (%&8)  /\\       _,__|.",
				"|(&(%)%)/  \\    . __,_|.",
				"| (%8%)/_##_\\   .     |.",
				"|  ||/ |    |   . @   |.",
				"|  ||  | /\\ | * . @   |.",
				"|  ||  |_||_|   .     |.",
				"|        ..  *  . \"'` |.",
				"| (%) O  ........     |.",
				"|        ..    ~troido|.",
				"+=======#  #==========+.",
				"........................"
			],
			"linkmask": [
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
			],
			"links": {
				"1": "https://tilde.town/~troido/cadastre/",
				"2": "https://tilde.town/~troido/index.html",
				"3": "https://tilde.town/~troido/entrance.html"
			}
		},
		"1,1": {
			"owner": "@_admin",
			"location": [1, 1],
			"art": [
				"╔══════════════════════╗",
				"║ Tilde.town Cadastre  ║",
				"║                      ║",
				"║ Any tilde.town user  ║",
				"║ can claim a parcel   ║",
				"║ of land to show some ║",
				"║ awesome ascii art    ║",
				"║                      ║",
				"║ * Instructions       ║",
				"║ * source (github)    ║",
				"║      Made by ~troido ║",
				"╚══════════════════════╝"
			],
			"linkmask": [
				"~~~~~~~~~~~~~~~~~~~~~~~~",
				"~ Tilde.town Cadastre  ~",
				"~                      ~",
				"~ Any tilde.town user  ~",
				"~ can claim a parcel   ~",
				"~ of land to show some ~",
				"~ awesome ascii art    ~",
				"~                      ~",
				"~ * 111111111111       ~",
				"~ * 222222222222222    ~",
				"~      Made by 3333333 ~",
				"~~~~~~~~~~~~~~~~~~~~~~~~"
			],
			"links": {
				"1": "https://tilde.town/~troido/cadastre",
				"2": "https://github.com/jmdejong/cadastre",
				"3": "https://tilde.town/~troido/index.html"
			}
		},
		"1,0": {
			"owner": null,
			"location": [1, 0],
			"art": [
				"                       .",
				"                       .",
				"__                     .",
				" ~\\________            .",
				"_   ~  ~   \\_ {%%}     .",
				" \\_______~<><{%%%%}    .",
				"         \\   ~{%%}     .",
				"          \\><>!||      .",
				"          |~  !||      .",
				"           \\  ~ `\\     .",
				"            \\__   \\    .",
				"               \\~  |   ."
			],
			"linkmask": [
				"                        ",
				"                        ",
				"                        ",
				"                        ",
				"                        ",
				"                        ",
				"                        ",
				"                        ",
				"                        ",
				"                        ",
				"                        ",
				"                        "
			],
			"links": {}
		}
	},
	"seed": 8138474425133413201
}
		"#;
		let expected = little_town();
		let des = serde_json::from_str::<Cadastre>(town).unwrap();
		for (key, parcel) in &des.places {
			assert_eq!(parcel, expected.places.get(key).unwrap());
		}
		assert_eq!(des, expected);
	}
	#[test]
	fn deserializes_serialized() {
		let town = little_town();
		assert_eq!(serde_json::from_str::<Cadastre>(&serde_json::json!(town).to_string()).unwrap(), town);
	}

	fn compare_text(s1: &str, s2: &str) {
		for (i, (l1, l2)) in s1.lines().zip(s2.lines()).enumerate() {
			assert_eq!(l1, l2, "mismatch on line {}", i);
		}
		assert_eq!(s1, s2);
	}
}
