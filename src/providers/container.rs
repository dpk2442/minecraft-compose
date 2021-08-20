use crate::providers::backends;

#[cfg_attr(test, mockall::automock)]
pub trait ContainerProvider {
    fn get_version(&self) -> Result<String, String>;
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
}
