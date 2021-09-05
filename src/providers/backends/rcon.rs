use rcon::{Builder, Connection};

#[cfg_attr(test, mockall::automock(type Output = MockRconBackend;))]
pub trait RconBackendFactory {
    type Output: RconBackend;

    fn create(&self, host: &str, port: &str) -> Result<Self::Output, ()>;
}

pub struct RconBackendFactoryImpl {}

impl RconBackendFactory for RconBackendFactoryImpl {
    type Output = RconBackendImpl;

    fn create(&self, host: &str, port: &str) -> Result<RconBackendImpl, ()> {
        Ok(RconBackendImpl {
            connection: futures::executor::block_on(
                Builder::new()
                    .enable_minecraft_quirks(true)
                    .connect(format!("{}:{}", host, port), "minecraft"),
            )
            .or_else(|err| {
                log::trace!("Unable to connect to {}:{}: {}", host, port, err);
                Err(())
            })?,
        })
    }
}

#[cfg_attr(test, mockall::automock)]
pub trait RconBackend: std::marker::Sized {
    fn cmd(&mut self, cmd: &str) -> Result<String, ()>;
}

pub struct RconBackendImpl {
    connection: Connection,
}

impl RconBackend for RconBackendImpl {
    fn cmd(&mut self, cmd: &str) -> Result<String, ()> {
        Ok(
            futures::executor::block_on(self.connection.cmd(cmd)).or_else(|err| {
                log::trace!("Failed to execute rcon command: {}", err);
                Err(())
            })?,
        )
    }
}
