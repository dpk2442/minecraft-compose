# MinecraftCompose [![Build and Test](https://github.com/dpk2442/minecraft-compose/actions/workflows/test.yml/badge.svg)](https://github.com/dpk2442/minecraft-compose/actions/workflows/test.yml)

A tool to manage Minecraft servers.

## Installing

Either download the correct binary for your system from the latest release or run

```
cargo install --git https://github.com/dpk2442/minecraft-compose.git
```

## Getting Started

1. Create an empty directory, this directory will contain the config and minecraft server data.
2. Create a config file, typically called `minecraft-compose.toml` filled with the desired values for your server.
3. Run `minecraft-compose up` to create the container and start the server (see config section for details).
4. Run `minecraft-compose status` to check on the status of the server.

## Usage

```
USAGE:
    minecraft-compose [FLAGS] [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -q, --quiet      Silences all output except errors
    -V, --version    Prints version information
    -v, --verbose    Prints additional output

OPTIONS:
    -f, --file <FILE>    Sets the file to use, defaults to ./minecraft-compose.toml

SUBCOMMANDS:
    console    Connects a console to the server
    create     Creates the server container
    destroy    Destroys the server container
    down       Stops and destroys the server container
    help       Prints this message or the help of the given subcommand(s)
    start      Starts the server container
    status     Displays the container status
    stop       Stops the server container
    up         Creates and starts the server container
```

## Config

The config is a [TOML](https://toml.io/) document, with sections and fields as described below.

### Example

```toml
name = "server"
host = "0.0.0.0"
port = 25565

[server]
type = "vanilla"
version = "1.17.1"
```

### Definition

```toml
name = "The name of the server container"
host = "The address to bind to on the host machine"
port = "The port to bind to on the host machine"

[server]
# This section defines details about the type and version of server to run
# Additional fields may be needed based on the server type.
type = "The type of the server"
version = "The version of minecraft the server should run"
```

#### Vanilla

A vanilla server should have `type` set to `vanilla`.
