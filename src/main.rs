
mod background;
mod cadastre;
mod config;
mod owner;
mod parcel;
mod pos;
mod util;
mod strutil;

mod main {

	use std::fs;
	use std::fs::File;
	use std::io::{Write, ErrorKind};
	use std::path::Path;
	use clap::Parser;
	use crate::{
		config::{Command, Action, Config},
		cadastre::Cadastre,
		parcel::Parcel,
		owner::Owner,
	};

	pub fn main() {
		let command: Command = Command::parse();
		match command.action {
			Action::Init(config) => {
				write_file_safe(&config.town_json, serde_json::to_string(&Cadastre::empty()).expect("Failed to serialize cadastre"))
					.expect("Failed to write town json file");
			}
			Action::Update(config) => {
				let old: Cadastre = read_old_cadastre(&config);
				let cadastre: Cadastre = generate_cadastre(&config, &old);
				write_file_safe(&config.town_json, serde_json::to_string(&Cadastre::empty()).expect("Failed to serialize cadastre"))
					.expect("Failed to write town json file");
				render(&config, &cadastre);
			}
			Action::Render(config) => {
				let cadastre: Cadastre = read_old_cadastre(&config);
				render(&config, &cadastre);
			}
		}
	}

	fn read_old_cadastre(config: &Config) -> Cadastre {
		serde_json::from_str(
			fs::read_to_string(config.town_json_old.clone().unwrap_or(config.town_json.clone()))
				.expect("Unable to read existing town json file")
				.as_str()
		).expect("Existing town file is not valid json")
	}

	fn render(config: &Config, cadastre: &Cadastre) {
		let mut text_file = File::create(&config.txt_render).expect("Failed to open file for txt render");
		cadastre.render_text(25, 25, |txt| text_file.write_all(txt.as_bytes()).expect("Failed to write txt render to file"));
		let mut html_file = File::create(&config.html_render).expect("Failed to open file for html render");
		cadastre.render_html(25, 25, |html| html_file.write_all(html.as_bytes()).expect("Failed to write html render to file"));
	}

	fn generate_cadastre(config: &Config, old: &Cadastre) -> Cadastre {
		let adminparcels = config.admin_parcel.iter()
			.filter_map(|path| read_parcel(path, Owner::Admin));

		let userparcels = fs::read_dir(&config.homedirs).expect("Failed to find home directories")
			.filter_map(Result::ok)
			.map(|entry| entry.path())
			.filter_map(|homedir| read_parcel(&homedir.join(&config.parcel_in_home), Owner::from_homedir(&homedir)?));

		let publicparcels = config.public_parcels.iter()
			.flat_map(|dir| fs::read_dir(dir).expect("Failed to read public plot directory"))
			.filter_map(Result::ok)
			.map(|entry| entry.path())
			.filter(|path| path.extension().is_some_and(|ext| ext == "prcl"))
			.filter_map(|path| read_parcel(&path, Owner::Public));

		let parcels = adminparcels.chain(userparcels).chain(publicparcels);

		Cadastre::build(&old, parcels)
	}

	fn read_parcel(path: &Path, owner: Owner) -> Option<Parcel> {
		let text = match fs::read_to_string(path) {
			Ok(text) => text,
			Err(io_err) => {
				if io_err.kind() != ErrorKind::NotFound {
					eprintln!("Can't read parcel {:?} of {:?}: {}", path, owner, io_err);
				}
				return None
			}
		};
		match Parcel::from_text(text.as_str(), owner.clone()) {
			Ok(parcel) => Some(parcel),
			Err(parse_err) => {
				eprintln!("Failed parsing parcel {:?} of {:?}:\n{}", path, owner, parse_err);
				None
			}
		}
	}

	fn write_file_safe<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<(), std::io::Error> {
		let temppath = path
			.as_ref()
			.with_file_name(
				format!(
					".{}.tmp",
					path.as_ref()
						.file_name()
						.ok_or_else(|| std::io::Error::other("Can't write a to a directory"))?
						.to_str()
						.unwrap_or("invalid")
				)
			);
		fs::write(&temppath, contents)?;
		fs::rename(&temppath, path)?;
		Ok(())
	}
}


fn main() {
	main::main()
}
