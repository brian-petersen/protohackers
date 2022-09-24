# Protohackers

Solutions to problems found on https://protohackers.com/.

To pass the problems, a server must be running. A `fly.toml`
file is provided to host on https://fly.io.

General steps to take to solve a new problem are:

1. Add solution server to `main.rs` with a new port
1. Update `fly.toml` by adding the new port as as service
1. Set up via `flyctl launch`
    - Choose to copy from exiting config file
    - The previous app name will be overridden
1. Deploy via `flyctl deploy`
1. Copy the public IPv4 address and pass off on protohackers: `flyctl ips list`
1. Destroy the spun up service: `flyctl destroy $APP_NAME`
    - The solutions don't need to be long running
