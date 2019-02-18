# rust-rocket-workspace

This is a boilerplate for getting started with [rocket.rs](https://rocket.rs).
The app is structured as a cargo workspace to make it easier to develop
modules in separation (e.g. adding a proc_macro crate, or splitting the app into
microservices).

The default web application will bundle and serve resources when built with docker
without needing to mount the resources in a volume separately.

Clone or fork this repo, and get going!

## Running

 - `cargo run --bin web`

## Included Modules
- `rocket`, `rocket_contrib` - Self explanatory. Server crate & additions for 
templating, static files, json and UUID handling. To enable msgpack support, 
add `"msgpack"` to `web/Cargo.toml`.
- `failure` - Smoother cross-crate error handling. Makes it easier to define
and use more specific `Error` types.
- `serde`, `serde_derive`, `serde_json` - For easily serializing structs for
an API, or to pass into a Template (or save to disk, or send over the network, 
etc...)
- `config` - Simple app configuration, supporting per-environment files and
prefixed environment variables

## Building

### From source

- For development `cargo build --bin web`
- For production: `cargo build --bin web --release`

### With Docker

- The docker image is configured for release builds, with layer caching for
dependencies: `docker build -t tag:version --target web .`