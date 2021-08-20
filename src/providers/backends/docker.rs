pub trait DockerBackend {
    fn version(&self) -> Result<bollard::system::Version, bollard::errors::Error>;
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
}
