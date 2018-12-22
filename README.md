This project creates a website where people can use [associative memory](https://en.wikipedia.org/wiki/Associative_memory_(psychology)) techniques to store information they know.
A working, free version of this can be found at [https://mindmap.trangar.com](https://mindmap.trangar.com).

## Build

To build the project, you'll need a **nightly** version of [rustup](https://rustup.rs).
After obtaining rustup, Simply run `cargo build` to build the project.

## Run

To run the project:
1. rename `.env.example` to `.env` and enter a valid postgres database url.
2. rename `Rocket.toml.example` to `Rocket.toml` and enter a valid postgres database url.
3. Install [diesel-cli](https://github.com/diesel-rs/diesel/tree/master/diesel_cli): `cargo install diesel_cli --no-default-features --features "postgres"`
4. Run `diesel migration run` to configure the database.
5. Run `cargo run` to start the web application.
6. Browse to `http://localhost:8000`

## Configuration

To configure the website, see [Configuring Rocket.toml](https://rocket.rs/guide/configuration/#rockettoml).

## Publishing

To publish this tool somewhere, copy the following items to the server:
- static
- templates
- Rocket.toml

Make sure to run `cargo build --release`, then copy `target/release/mindmap_server` to your server.

Configure the database by having a valid `.env` file, and running `diesel migration run`.

On the server, run the `mindmap_server` executable as a service or in a detached shell. 
