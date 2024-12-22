
use std::path::PathBuf;
use clap::{Parser, Subcommand, Args};

#[derive(Debug, Args)]
pub struct Config {

	/// the directory containing a list of all homedirs for users
	#[arg(long, default_value="/home/", env="CADASTRE_HOME_DIRS")]
	pub homedirs: PathBuf,

	/// the location of the user's parcel within their own home dir
	#[arg(long, default_value=".cadastre/home.txt", env="CADASTRE_TOWN_JSON_PATH")]
	pub parcel_in_home: PathBuf,

	/// the location of the admin parcel
	#[arg(long, env="CADASTRE_ADMIN_PARCEL_FILE")]
	pub admin_parcel: Vec<PathBuf>,

	/// the directories for public parcels
	#[arg(long, env="CADASTRE_PUBLIC_PARCELS_DIRS")]
	pub public_parcels: Vec<PathBuf>,

	/// location where to write the town json representation
	#[arg(long, default_value="./town.json", env="CADASTRE_TOWN_JSON_FILE")]
	pub town_json: PathBuf,

	/// location from which to read the old town json representation
	#[arg(long, env="CADASTRE_TOWN_JSON_OLD_FILE")]
	pub town_json_old: Option<PathBuf>,

	/// location to write town.txt
	#[arg(long, default_value="./town.txt", env="CADASTRE_TXT_RENDER_FILE")]
	pub txt_render: PathBuf,
	/// location to write town.html
	#[arg(long, default_value="./town.html", env="CADASTRE_HTML_RENDER_FILE")]
	pub html_render: PathBuf
}

#[derive(Debug, Parser)]
#[command(name = "cadastrs", version, author, about)]
pub struct Command {

	#[command(subcommand)]
	pub action: Action,
}


#[derive(Debug, Subcommand)]
pub enum Action {
	/// Create new cadastre world
	Init(Config),
	/// Update cadastre world with townie data
	Update(Config)
}
