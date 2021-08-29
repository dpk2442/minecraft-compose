use crate::config::Config;
use rcon::Builder;
use rustyline::error::ReadlineError;
use rustyline::Editor;

#[cfg_attr(test, mockall::automock)]
pub trait RconProvider {
    fn run_interactive_rcon_session(
        &self,
        config: &Config,
        host: &str,
        port: &str,
    ) -> Result<(), ()>;
}

pub struct RconProviderImpl {}

impl RconProvider for RconProviderImpl {
    fn run_interactive_rcon_session(
        &self,
        config: &Config,
        host: &str,
        port: &str,
    ) -> Result<(), ()> {
        log::trace!("Establishing rcon connection to {}:{}", host, port);
        futures::executor::block_on(async {
            let mut connection = Builder::new()
                .enable_minecraft_quirks(true)
                .connect(format!("{}:{}", host, port), "minecraft")
                .await
                .or_else(|err| {
                    log::trace!("Unable to connect to {}:{}: {}", host, port, err);
                    Err(())
                })?;

            let mut rl = Editor::<()>::new();
            loop {
                let readline = rl.readline(&format!("[{}] > ", config.name));
                match readline {
                    Ok(line) => {
                        rl.add_history_entry(&line);
                        let response = connection.cmd(&line).await.or_else(|err| {
                            log::trace!("Failed to execute rcon command: {}", err);
                            Err(())
                        })?;

                        if response.len() > 0 {
                            log::info!("{}", response);
                        }

                        Ok(())
                    }
                    Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                        break;
                    }
                    Err(err) => {
                        log::trace!("Encountered a readline error: {}", err);
                        Err(())
                    }
                }?
            }

            Ok(())
        })
    }
}

pub fn new_from_defaults() -> RconProviderImpl {
    RconProviderImpl {}
}
