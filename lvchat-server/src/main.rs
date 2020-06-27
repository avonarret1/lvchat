use lvchat_server::{config::Config, error::Error, run};

fn init_logger(config: &Config) {
    let logger = if config.debug {
        flexi_logger::Logger::with_str("lvchat_core=debug, lvchat_server=debug")
    } else if config.verbose {
        flexi_logger::Logger::with_str("lvchat_core=info, lvchat_server=info")
    } else {
        flexi_logger::Logger::with_str("lvchat_core=error, lvchat_server=error")
    };

    let logger = if let Some(ref path) = config.logs_path {
        let logger = logger.log_to_file().directory(path);

        if !config.quiet {
            logger.duplicate_to_stderr(flexi_logger::Duplicate::All)
        } else {
            logger
        }
    } else {
        if config.quiet {
            logger.do_not_log()
        } else {
            logger
        }
    };

    logger.start().unwrap();
}

fn main() -> Result<(), Error> {
    let config = Config::init();

    init_logger(&config);

    log::info!("Using {:#?}", config);

    if let Err(e) = run(config) {
        log::error!("Error: {}", e);
    }

    Ok(())
}
