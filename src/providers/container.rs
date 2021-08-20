use crate::providers::backends;

pub trait ContainerProvider {
    fn get_version(&self) -> Result<bollard::system::Version, String>;
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
    fn get_version(&self) -> Result<bollard::system::Version, String> {
        self.docker
            .version()
            .or_else(|_| Err("Failed to fetch version".to_owned()))
    }
}
