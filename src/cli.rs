use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
pub struct Args {
    #[arg(long, default_value_t = 3000)]
    pub port: u16,

    #[arg(long, default_value = ".")]
    pub data_dir: PathBuf,

    #[arg(default_value = "db.json")]
    pub state: PathBuf,

    #[arg(long, short)]
    pub username: Option<String>,

    #[arg(long, short)]
    pub password: Option<String>,
}
