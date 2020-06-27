use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Config {
    #[structopt(short, long)]
    pub verbose: bool,

    #[structopt(short, long)]
    pub debug: bool,

    #[structopt(short, long)]
    pub quiet: bool,

    #[structopt(short, long, default_value = "5050")]
    pub port: u16,

    #[structopt(long = "logs")]
    pub logs_path: Option<PathBuf>,
    //#[structopt(long)]
    //pub client_timeout
}

impl Config {
    pub fn init() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                Config {
                    verbose: true,
                    debug: true,
                    quiet: false,
                    port: 5050,
                    logs_path: None,//Some(PathBuf::from("logs")),
                }
            } else {
                <Self as StructOpt>::from_args()
            }
        }
    }
}
