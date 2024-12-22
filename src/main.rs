
mod cadastre;
mod config;
mod parcel;
mod pos;
mod util;
mod strutil;

mod main {

	use std::fs;
	use std::path::Path;
	use std::io::ErrorKind;
	use clap::Parser;
	use crate::{
		config::{Command, Action, Config},
		cadastre::Cadastre,
		parcel::{Parcel, Owner},
	};

	pub fn main() {
		let command: Command = Command::parse();
		println!("{:?}", command);
		match command.action {
			Action::Init(config) => update(config, Cadastre::empty()),
			Action::Update(config) => {
				let old: Cadastre = serde_json::from_str(
					fs::read_to_string(config.town_json_old.clone().unwrap_or(config.town_json.clone())).unwrap().as_str()
				).unwrap();
				update(config, old);
			}
		}
	}

	fn update(config: Config, old: Cadastre) {
		let cadastre = generate_cadastre(&config, &old);

		println!("{:?}", cadastre);
		write_file_safe(config.town_json, serde_json::to_string(&cadastre).unwrap()).unwrap();
	}

	fn generate_cadastre(config: &Config, old: &Cadastre) {
		let adminparcels = config.admin_parcel.iter()
			.filter_map(|path| read_parcel(path, Owner::Admin));

		let userparcels = fs::read_dir(&config.homedirs).unwrap()
			.filter_map(Result::ok)
			.map(|entry| entry.path())
			.filter_map(|homedir| read_parcel(&homedir.join(&config.parcel_in_home), Owner::from_homedir(&homedir)?));

		let publicparcels = config.public_parcels.iter()
			.flat_map(|dir| fs::read_dir(dir).unwrap())
			.filter_map(Result::ok)
			.map(|entry| entry.path())
			.filter(|path| path.extension().is_some_and(|ext| ext == "prcl"))
			.filter_map(|path| read_parcel(&path, Owner::Public));

		let parcels = adminparcels.chain(userparcels).chain(publicparcels);

		Cadastre::build(&old, parcels);
	}

	fn read_parcel(path: &Path, owner: Owner) -> Option<Parcel> {
		let text = match fs::read_to_string(path) {
			Ok(text) => text,
			Err(io_err) => {
				if io_err.kind() != ErrorKind::NotFound {
					eprintln!("Can't read parcel {:?} of {:?}: {:?}", path, owner, io_err);
				}
				return None
			}
		};
		match Parcel::from_text(text.as_str(), owner.clone()) {
			Ok(parcel) => Some(parcel),
			Err(parse_err) => {
				eprintln!("Failed parsing parcel {:?} of {:?}: {:?}", path, owner, parse_err);
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
						.ok_or_else(|| std::io::Error::new(ErrorKind::IsADirectory, "Can't write a to a directory"))?
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
