use bollard::container::Config as ContainerConfig;
use bollard::models::{
    ContainerStateStatusEnum, HostConfig, PortBinding, PortMap, RestartPolicy,
    RestartPolicyNameEnum,
};
use bollard::service;
use std::path::PathBuf;

use crate::config::{self, Config};
use crate::providers::backends;

const IMAGE_NAME: &str = "itzg/minecraft-server";
const IMAGE_TAG: &str = "latest";

#[derive(Debug, PartialEq)]
pub enum ContainerState {
    Unknown,
    NotFound,
    Stopped,
    Running,
}

#[cfg_attr(test, mockall::automock)]
pub trait ContainerProvider {
    fn create_container(&self, config: &Config, data_path: &PathBuf) -> Result<(), ()>;
    fn delete_container(&self, config: &Config) -> Result<(), ()>;
    fn start_container(&self, config: &Config) -> Result<(), ()>;
    fn stop_container(&self, config: &Config) -> Result<(), ()>;
    fn get_container_status(&self, config: &Config) -> Result<ContainerState, ()>;
    fn get_container_rcon_address(&self, config: &Config) -> Result<(String, String), ()>;
}

pub struct ContainerProviderImpl<T: backends::docker::DockerBackend> {
    docker: T,
}

pub fn new_from_defaults() -> Result<ContainerProviderImpl<backends::docker::DockerBackendImpl>, ()>
{
    Ok(ContainerProviderImpl {
        docker: backends::docker::new_from_defaults()?,
    })
}

impl<T: backends::docker::DockerBackend> ContainerProvider for ContainerProviderImpl<T> {
    fn create_container(&self, config: &Config, data_path: &PathBuf) -> Result<(), ()> {
        let data_path = match data_path.to_str() {
            Some(s) => Ok(s),
            None => {
                log::error!("Unable to convert data path to string");
                Err(())
            }
        }?;

        let mut port_map = PortMap::new();
        port_map.insert(
            "25565/tcp".to_owned(),
            Some(vec![PortBinding {
                host_ip: Some(config.host.to_owned()),
                host_port: Some(config.port.to_string()),
            }]),
        );
        port_map.insert(
            "25575/tcp".to_owned(),
            Some(vec![PortBinding {
                host_ip: Some("127.0.0.1".to_owned()),
                host_port: None,
            }]),
        );

        let mut env = vec![String::from("EULA=true")];
        match &config.server {
            config::Server::Vanilla(server_config) => {
                env.append(&mut vec![
                    String::from("TYPE=VANILLA"),
                    format!("VERSION={}", server_config.version),
                ]);
            }
        }

        self.docker.download_image(IMAGE_NAME, IMAGE_TAG)?;

        self.docker.create_container(
            &config.name,
            ContainerConfig {
                image: Some(format!("{}:{}", IMAGE_NAME, IMAGE_TAG)),
                env: Some(env),
                host_config: Some(HostConfig {
                    binds: Some(vec![format!("{}:/data", data_path)]),
                    port_bindings: Some(port_map),
                    restart_policy: Some(RestartPolicy {
                        name: Some(RestartPolicyNameEnum::ALWAYS),
                        maximum_retry_count: None,
                    }),
                    ..std::default::Default::default()
                }),
                ..std::default::Default::default()
            },
        )
    }

    fn delete_container(&self, config: &Config) -> Result<(), ()> {
        self.docker.delete_container(&config.name)
    }

    fn start_container(&self, config: &Config) -> Result<(), ()> {
        self.docker.start_container(&config.name)
    }

    fn stop_container(&self, config: &Config) -> Result<(), ()> {
        self.docker.stop_container(&config.name)
    }

    fn get_container_status(&self, config: &Config) -> Result<ContainerState, ()> {
        match self.docker.inspect_container(&config.name) {
            Ok(backends::docker::InspectResult::Ok(result)) => match result.state {
                None => Ok(ContainerState::Unknown),
                Some(state) => match state.status {
                    Some(ContainerStateStatusEnum::CREATED)
                    | Some(ContainerStateStatusEnum::EMPTY)
                    | Some(ContainerStateStatusEnum::EXITED)
                    | Some(ContainerStateStatusEnum::DEAD)
                    | Some(ContainerStateStatusEnum::PAUSED) => Ok(ContainerState::Stopped),
                    Some(ContainerStateStatusEnum::RUNNING)
                    | Some(ContainerStateStatusEnum::RESTARTING) => Ok(ContainerState::Running),
                    Some(ContainerStateStatusEnum::REMOVING) => Ok(ContainerState::NotFound),
                    None => Ok(ContainerState::Unknown),
                },
            },
            Ok(backends::docker::InspectResult::NotFound) => Ok(ContainerState::NotFound),
            Err(()) => Err(()),
        }
    }

    fn get_container_rcon_address(&self, config: &Config) -> Result<(String, String), ()> {
        match self.docker.inspect_container(&config.name) {
            Ok(backends::docker::InspectResult::Ok(service::ContainerInspectResponse {
                network_settings:
                    Some(service::NetworkSettings {
                        ports: Some(ports), ..
                    }),
                ..
            })) => match ports.get("25575/tcp") {
                Some(Some(bindings)) => {
                    if bindings.len() != 1 {
                        return Err(());
                    }
                    Ok((
                        bindings[0].host_ip.as_ref().unwrap().to_owned(),
                        bindings[0].host_port.as_ref().unwrap().to_owned(),
                    ))
                }
                _ => Err(()),
            },
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;

    use super::*;
    use crate::config;
    use crate::providers::backends::docker::MockDockerBackend;

    fn get_container_provider() -> ContainerProviderImpl<MockDockerBackend> {
        ContainerProviderImpl {
            docker: MockDockerBackend::new(),
        }
    }

    fn get_config() -> config::Config {
        config::Config {
            name: "name".to_owned(),
            host: "0.0.0.0".to_owned(),
            port: 25565,
            server: config::Server::Vanilla(config::VanillaServer {
                version: "1.17.1".to_owned(),
            }),
        }
    }

    #[test]
    fn test_create_container() {
        let mut container_provider = get_container_provider();
        let config = get_config();
        let data_path = PathBuf::from("path");

        container_provider
            .docker
            .expect_download_image()
            .with(eq("itzg/minecraft-server"), eq("latest"))
            .times(1)
            .returning(|_, _| Ok(()));

        container_provider
            .docker
            .expect_create_container()
            .withf(|name, container_config| {
                name == "name"
                    && container_config.image == Some("itzg/minecraft-server:latest".to_owned())
                    && container_config.env
                        == Some(vec![
                            String::from("EULA=true"),
                            String::from("TYPE=VANILLA"),
                            String::from("VERSION=1.17.1"),
                        ])
                    && match &container_config.host_config {
                        None => false,
                        Some(host_config) => {
                            host_config.binds == Some(vec!["path:/data".to_owned()])
                                && host_config.restart_policy
                                    == Some(RestartPolicy {
                                        name: Some(RestartPolicyNameEnum::ALWAYS),
                                        maximum_retry_count: None,
                                    })
                                && match &host_config.port_bindings {
                                    None => false,
                                    Some(port_bindings) => {
                                        port_bindings.len() == 2
                                            && match port_bindings.get("25565/tcp") {
                                                None => false,
                                                Some(port) => {
                                                    port == &Some(vec![PortBinding {
                                                        host_ip: Some("0.0.0.0".to_owned()),
                                                        host_port: Some("25565".to_owned()),
                                                    }])
                                                }
                                            }
                                            && match port_bindings.get("25575/tcp") {
                                                None => false,
                                                Some(port) => {
                                                    port == &Some(vec![PortBinding {
                                                        host_ip: Some("127.0.0.1".to_owned()),
                                                        host_port: None,
                                                    }])
                                                }
                                            }
                                    }
                                }
                        }
                    }
            })
            .times(1)
            .returning(|_, _| Ok(()));

        assert_eq!(
            Ok(()),
            container_provider.create_container(&config, &data_path)
        );
    }

    #[test]
    fn test_delete_container() {
        let mut container_provider = get_container_provider();
        let config = get_config();

        container_provider
            .docker
            .expect_delete_container()
            .with(eq("name"))
            .times(1)
            .returning(|_| Ok(()));

        assert_eq!(Ok(()), container_provider.delete_container(&config));
    }

    #[test]
    fn test_start_container() {
        let mut container_provider = get_container_provider();
        let config = get_config();

        container_provider
            .docker
            .expect_start_container()
            .with(eq("name"))
            .times(1)
            .returning(|_| Ok(()));

        assert_eq!(Ok(()), container_provider.start_container(&config));
    }

    #[test]
    fn test_stop_container() {
        let mut container_provider = get_container_provider();
        let config = get_config();

        container_provider
            .docker
            .expect_stop_container()
            .with(eq("name"))
            .times(1)
            .returning(|_| Ok(()));

        assert_eq!(Ok(()), container_provider.stop_container(&config));
    }

    mod test_get_container_status {
        use bollard::models::{ContainerInspectResponse, ContainerState as DockerContainerState};

        use super::*;

        #[test]
        fn inspect_result_not_found() {
            let mut container_provider = get_container_provider();
            let config = get_config();

            container_provider
                .docker
                .expect_inspect_container()
                .with(eq("name"))
                .times(1)
                .returning(|_| Ok(backends::docker::InspectResult::NotFound));

            assert_eq!(
                Ok(ContainerState::NotFound),
                container_provider.get_container_status(&config)
            );
        }

        #[test]
        fn none_state() {
            let mut container_provider = get_container_provider();
            let config = get_config();

            container_provider
                .docker
                .expect_inspect_container()
                .with(eq("name"))
                .times(1)
                .returning(|_| {
                    Ok(backends::docker::InspectResult::Ok(
                        std::default::Default::default(),
                    ))
                });

            assert_eq!(
                Ok(ContainerState::Unknown),
                container_provider.get_container_status(&config)
            );
        }

        macro_rules! get_container_status_tests {
            ($($name:ident: $docker_status:expr, $result_state:expr;)*) => {
            $(
                #[test]
                fn $name() {
                    let mut container_provider = get_container_provider();
                    let config = get_config();

                    container_provider
                        .docker
                        .expect_inspect_container()
                        .with(eq("name"))
                        .times(1)
                        .returning(|_| Ok(backends::docker::InspectResult::Ok(ContainerInspectResponse {
                            state: Some(DockerContainerState {
                                status: $docker_status,
                                ..std::default::Default::default()
                            }),
                            ..std::default::Default::default()
                        })));

                    assert_eq!(
                        Ok($result_state),
                        container_provider.get_container_status(&config)
                    );
                }
            )*
            }
        }

        get_container_status_tests! {
            status_none: None, ContainerState::Unknown;
            status_created: Some(ContainerStateStatusEnum::CREATED), ContainerState::Stopped;
            status_empty: Some(ContainerStateStatusEnum::EMPTY), ContainerState::Stopped;
            status_exited: Some(ContainerStateStatusEnum::EXITED), ContainerState::Stopped;
            status_dead: Some(ContainerStateStatusEnum::DEAD), ContainerState::Stopped;
            status_paused: Some(ContainerStateStatusEnum::PAUSED), ContainerState::Stopped;
            status_running: Some(ContainerStateStatusEnum::RUNNING), ContainerState::Running;
            status_restarting: Some(ContainerStateStatusEnum::RESTARTING), ContainerState::Running;
            status_removing: Some(ContainerStateStatusEnum::REMOVING), ContainerState::NotFound;
        }
    }

    mod test_get_rcon_address {
        use std::collections::HashMap;

        use super::*;
        use bollard::service::PortBinding;

        fn build_ports_map(include_binding: bool) -> HashMap<String, Option<Vec<PortBinding>>> {
            let mut ports_map: std::collections::HashMap<
                String,
                Option<Vec<bollard::service::PortBinding>>,
            > = std::collections::HashMap::new();

            if include_binding {
                ports_map.insert(
                    "25575/tcp".to_owned(),
                    Some(vec![bollard::service::PortBinding {
                        host_ip: Some("host".to_owned()),
                        host_port: Some("port".to_owned()),
                    }]),
                );
            }

            ports_map
        }

        #[test]
        fn success() {
            let mut container_provider = get_container_provider();
            let config = get_config();

            container_provider
                .docker
                .expect_inspect_container()
                .with(eq("name"))
                .times(1)
                .returning(|_| {
                    Ok(backends::docker::InspectResult::Ok(
                        bollard::models::ContainerInspectResponse {
                            network_settings: Some(service::NetworkSettings {
                                ports: Some(build_ports_map(true)),
                                ..std::default::Default::default()
                            }),
                            ..std::default::Default::default()
                        },
                    ))
                });

            assert_eq!(
                Ok(("host".to_owned(), "port".to_owned())),
                container_provider.get_container_rcon_address(&config)
            );
        }

        #[test]
        fn incorrect_binding_count() {
            let mut container_provider = get_container_provider();
            let config = get_config();

            container_provider
                .docker
                .expect_inspect_container()
                .with(eq("name"))
                .times(1)
                .returning(|_| {
                    Ok(backends::docker::InspectResult::Ok(
                        bollard::models::ContainerInspectResponse {
                            network_settings: Some(service::NetworkSettings {
                                ports: Some(build_ports_map(false)),
                                ..std::default::Default::default()
                            }),
                            ..std::default::Default::default()
                        },
                    ))
                });

            assert_eq!(
                Err(()),
                container_provider.get_container_rcon_address(&config)
            );
        }
    }
}
