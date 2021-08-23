use bollard::container::Config as ContainerConfig;
use bollard::models::{HostConfig, PortBinding, PortMap, RestartPolicy, RestartPolicyNameEnum};
use std::path::PathBuf;

use crate::config::Config;
use crate::providers::backends;

#[cfg_attr(test, mockall::automock)]
pub trait ContainerProvider {
    fn get_version(&self) -> Result<String, String>;
    fn create_container(&self, config: &Config, data_path: &PathBuf) -> Result<(), ()>;
    fn delete_container(&self, config: &Config) -> Result<(), ()>;
    fn start_container(&self, config: &Config) -> Result<(), ()>;
    fn stop_container(&self, config: &Config) -> Result<(), ()>;
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
    fn get_version(&self) -> Result<String, String> {
        match self.docker.version() {
            Ok(version) => Ok(version.version.unwrap_or("unknown".to_owned())),
            Err(_) => Err("Failed to fetch version".to_owned()),
        }
    }

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

        self.docker.create_container(
            &config.name,
            ContainerConfig {
                image: Some("itzg/minecraft-server".to_owned()),
                env: Some(vec![
                    String::from("TYPE=VANILLA"),
                    String::from("VERSION=1.17.1"),
                    String::from("EULA=true"),
                ]),
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
        }
    }

    #[test]
    fn test_create_container() {
        let mut container_provider = get_container_provider();
        let config = get_config();
        let data_path = PathBuf::from("path");

        container_provider
            .docker
            .expect_create_container()
            .withf(|name, container_config| {
                name == "name"
                    && container_config.image == Some("itzg/minecraft-server".to_owned())
                    && container_config.env
                        == Some(vec![
                            String::from("TYPE=VANILLA"),
                            String::from("VERSION=1.17.1"),
                            String::from("EULA=true"),
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
                                        port_bindings.len() == 1
                                            && match port_bindings.get("25565/tcp") {
                                                None => false,
                                                Some(port) => {
                                                    port == &Some(vec![PortBinding {
                                                        host_ip: Some("0.0.0.0".to_owned()),
                                                        host_port: Some("25565".to_owned()),
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
}
