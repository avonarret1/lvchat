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

    #[structopt(short, long)]
    pub host: String,

    #[structopt(short, long, default_value = "5050")]
    pub port: u16,

    #[structopt(short, long)]
    pub nick: String,

    #[structopt(long = "logs")]
    pub logs_path: Option<PathBuf>,
}

impl Config {
    pub fn new() -> Self {
        cfg_if::cfg_if! {
            if #[cfg(debug_assertions)] {
                Config {
                    verbose: true,
                    debug: true,
                    quiet: false,
                    host: format!("127.0.0.1"),
                    nick: format!("avonarret"),
                    port: 5050,
                    logs_path: None,//Some(PathBuf::from("logs")),
                }
            } else {
                <Self as StructOpt>::from_args()
            }
        }
    }
}
