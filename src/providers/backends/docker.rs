use bollard::container::{Config, CreateContainerOptions};

#[cfg_attr(test, mockall::automock)]
pub trait DockerBackend {
    fn version(&self) -> Result<bollard::system::Version, bollard::errors::Error>;
    fn create_container(&self, name: &str, container_config: Config<String>) -> Result<(), ()>;
    fn delete_container(&self, name: &str) -> Result<(), ()>;
}

pub struct DockerBackendImpl {
    docker: bollard::Docker,
}

pub fn new_from_defaults() -> Result<DockerBackendImpl, ()> {
    Ok(DockerBackendImpl {
        docker: bollard::Docker::connect_with_local_defaults().or_else(|err| {
            log::debug!("Failed to connect to docker: {}", err);
            Err(())
        })?,
    })
}

impl DockerBackend for DockerBackendImpl {
    fn version(&self) -> Result<bollard::system::Version, bollard::errors::Error> {
        futures::executor::block_on(self.docker.version())
    }

    fn create_container(&self, name: &str, container_config: Config<String>) -> Result<(), ()> {
        log::trace!("Creating container {}", name);
        match futures::executor::block_on(self.docker.create_container(
            Some(CreateContainerOptions { name: name }),
            container_config,
        )) {
            Ok(response) => {
                response.warnings.iter().for_each(|warning| {
                    log::warn!("Warning: {}", warning);
                });
                Ok(())
            }
            Err(err) => {
                log::trace!("Unable to create container {}: {}", name, err);
                Err(())
            }
        }
    }

    fn delete_container(&self, name: &str) -> Result<(), ()> {
        log::trace!("Deleting container {}", name);
        futures::executor::block_on(self.docker.remove_container(name, None)).or_else(|err| {
            log::trace!("Unable to delete container {}: {}", name, err);
            Err(())
        })
    }
}
