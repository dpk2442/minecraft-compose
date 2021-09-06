use crate::config::Config;
use crate::providers::backends::input::{
    InputBackend, InputBackendFactory, InputBackendFactoryImpl, InputResponse,
};
use crate::providers::backends::rcon::{RconBackend, RconBackendFactory, RconBackendFactoryImpl};

#[cfg_attr(test, mockall::automock)]
pub trait GameProvider {
    fn run_interactive_rcon_session(
        &self,
        config: &Config,
        host: &str,
        port: &str,
    ) -> Result<(), ()>;
}

pub struct GameProviderImpl<
    RconBackendFactoryType: RconBackendFactory,
    InputBackendFactoryType: InputBackendFactory,
> {
    rcon_backend_factory: RconBackendFactoryType,
    input_backend_factory: InputBackendFactoryType,
}

impl<RconBackendFactoryType: RconBackendFactory, InputBackendFactoryType: InputBackendFactory>
    GameProvider for GameProviderImpl<RconBackendFactoryType, InputBackendFactoryType>
{
    fn run_interactive_rcon_session(
        &self,
        config: &Config,
        host: &str,
        port: &str,
    ) -> Result<(), ()> {
        log::trace!("Establishing rcon connection to {}:{}", host, port);
        let mut rcon_backend = self.rcon_backend_factory.create(host, port)?;
        let mut input_backend = self.input_backend_factory.create();
        loop {
            match input_backend.get_line(&format!("[{}] > ", config.name)) {
                Ok(InputResponse::Input(line)) => {
                    let response = rcon_backend.cmd(&line)?;
                    if response.len() > 0 {
                        log::info!("{}", response);
                    }

                    Ok(())
                }
                Ok(InputResponse::EndOfInput) => break,
                Err(()) => Err(()),
            }?
        }

        Ok(())
    }
}

pub fn new_from_defaults() -> GameProviderImpl<RconBackendFactoryImpl, InputBackendFactoryImpl> {
    GameProviderImpl {
        rcon_backend_factory: RconBackendFactoryImpl {},
        input_backend_factory: InputBackendFactoryImpl {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use mockall::{predicate::eq, Sequence};

    fn get_config() -> Config {
        Config {
            name: "name".to_owned(),
            host: "0.0.0.0".to_owned(),
            port: 25565,
            server: config::Server {
                version: "1.17.1".to_owned(),
                server_type: config::ServerType::Vanilla,
            },
        }
    }

    mod test_interactive_rcon_session {
        use super::*;
        use crate::providers::backends::input::{MockInputBackend, MockInputBackendFactory};
        use crate::providers::backends::rcon::{MockRconBackend, MockRconBackendFactory};

        fn setup(
            input_responses: Vec<Result<InputResponse, ()>>,
            rcon_inputs: Vec<String>,
            rcon_responses: Vec<Result<String, ()>>,
        ) -> GameProviderImpl<MockRconBackendFactory, MockInputBackendFactory> {
            let mut rcon_sequence = Sequence::new();
            let mut mock_rcon_backend = MockRconBackend::new();
            for rcon_idx in 0..rcon_inputs.len() {
                let expected_input = rcon_inputs[rcon_idx].to_owned();
                let response = rcon_responses[rcon_idx].clone();
                mock_rcon_backend
                    .expect_cmd()
                    .times(1)
                    .withf(move |input| input == expected_input)
                    .return_once(move |_| response)
                    .in_sequence(&mut rcon_sequence);
            }

            let mut mock_rcon_factory = MockRconBackendFactory::new();
            mock_rcon_factory
                .expect_create()
                .with(eq("host"), eq("port"))
                .times(1)
                .return_once(move |_, _| Ok(mock_rcon_backend));

            let mut input_sequence = Sequence::new();
            let mut mock_input_backend = MockInputBackend::new();
            for input_response in input_responses {
                mock_input_backend
                    .expect_get_line()
                    .times(1)
                    .with(eq("[name] > "))
                    .return_once(move |_| input_response.clone())
                    .in_sequence(&mut input_sequence);
            }

            let mut mock_input_factory = MockInputBackendFactory::new();
            mock_input_factory
                .expect_create()
                .with()
                .times(1)
                .return_once(move || mock_input_backend);

            GameProviderImpl {
                rcon_backend_factory: mock_rcon_factory,
                input_backend_factory: mock_input_factory,
            }
        }

        #[test]
        fn error_on_connect() {
            let config = get_config();

            let mut mock_rcon_factory = MockRconBackendFactory::new();
            mock_rcon_factory
                .expect_create()
                .with(eq("host"), eq("port"))
                .times(1)
                .return_once(move |_, _| Err(()));

            let game_provider = GameProviderImpl {
                rcon_backend_factory: mock_rcon_factory,
                input_backend_factory: MockInputBackendFactory::new(),
            };

            assert_eq!(
                Err(()),
                game_provider.run_interactive_rcon_session(&config, "host", "port")
            );
        }

        #[test]
        fn error_reading_input() {
            let config = get_config();
            let game_provider = setup(vec![Err(())], vec![], vec![]);

            assert_eq!(
                Err(()),
                game_provider.run_interactive_rcon_session(&config, "host", "port")
            );
        }

        #[test]
        fn error_running_cmd() {
            let config = get_config();
            let game_provider = setup(
                vec![Ok(InputResponse::Input("test".to_owned()))],
                vec!["test".to_owned()],
                vec![Err(())],
            );

            assert_eq!(
                Err(()),
                game_provider.run_interactive_rcon_session(&config, "host", "port")
            );
        }

        #[test]
        fn success() {
            let config = get_config();
            let game_provider = setup(
                vec![
                    Ok(InputResponse::Input("test1".to_owned())),
                    Ok(InputResponse::Input("test2".to_owned())),
                    Ok(InputResponse::EndOfInput),
                ],
                vec!["test1".to_owned(), "test2".to_owned()],
                vec![Ok("response1".to_owned()), Ok("response2".to_owned())],
            );

            assert_eq!(
                Ok(()),
                game_provider.run_interactive_rcon_session(&config, "host", "port")
            );
        }
    }
}
