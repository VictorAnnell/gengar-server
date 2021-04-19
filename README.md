# Gengar Server
> VaccMe server back-end written in Rust by group Gengar

The server back-end to the [VaccMe mobile application][] written in Rust for the
[Computer Systems with Project Work] course at [Uppsala University][], spring 2021.

[computer systems with project work]: https://www.uu.se/en/admissions/master/selma/kursplan/?kpid=39194
[uppsala university]:                 https://www.uu.se/en
[vaccme mobile application]:          https://github.com/horrokotoj/Gengar

## Developing

### Setting up project locally

1. Install Rust using [rustup][].

2. Clone this repository.

3. Setup your development MySQL database if you haven't already done so.

   *Note:* If you set up MySQL manually and didn't specify the MySQL user to be one with elevated permissions,
   you'll want to run a command like ``mysql -c "GRANT ALL ON `gengar_dev`.* TO ''@'localhost';" -uroot``,
   or something similar for the user that you've specified.

   #### Docker

   If you have [Docker][] and [Docker Compose][] installed you can use the provided
   docker-compose.yml file to automatically set up a MySQL container to work against.
   You can start the container with this command from the repositories root directory:

   ```shell
   docker-compose up -d
   ```

   To stop the container:

   ```shell
   docker-compose stop
   ```

   To stop and remove the container and the associated docker volume:

   ```shell
   docker-compose down -v
   ```

   ##### Adminer

   The provided docker-compose.yml file also defines a Adminer container which will be created and started alongside the MySQL container.
   Adminer provides a web interface for database management which can be used for easy administration of your database while working on the application.
   Access Adminer by visiting the URL [`localhost:8080`](http://localhost:8080) in a browser after starting the container.
   There use the following credentials to login:

   ```
   Server: gengar.mysql
   Username: root
   Password: gengar
   ```

4. Create a `.env` file in the repository's root directory, and add the connection details for your database.

   See [.env.sample](.env.sample) for an example that works with the previously mentioned docker setup.

5. With this done try running the test suite to confirm everything works:

   ```
   cargo test
   ```

> Instructions taken in large part from [diesel-rs][]

[diesel-rs]:      https://github.com/diesel-rs/diesel/blob/master/CONTRIBUTING.md
[docker compose]: https://docs.docker.com/compose/install/
[docker]:         https://www.docker.com/
[rustup]:         https://rustup.rs/

### Building

Build and run the project with cargo:

```shell
cargo run
```

Replace `run` with `build` if you want to build but not run.

### Cleaning

Remove generated artifacts:

```shell
cargo clean
```

### Generating documentation

Generate and open documentation for the local package and all dependencies:

```shell
cargo doc --open
```

## Contributing

### Coding Style

The code follows the [Rust Style Guide][], enforced using [rustfmt][],
with additional linting provided by [clippy][].

To run formatting tests locally:

1. Install rustfmt and clippy (if they aren't already) by running:

   ```shell
   rustup component add rustfmt
   rustup component add clippy
   ```

2. Run clippy using cargo from the root of the project repo.

   ```shell
   cargo clippy --all-targets --all-features
   ```

   Each PR needs to compile without warning.

3. Run rustfmt using cargo from the root of your diesel repo.

   To see changes that need to be made, run:

   ```shell
   cargo fmt --all -- --check
   ```

   If all code is properly formatted (e.g. if you have not made any changes),
   this should run without error or output.
   If your code needs to be reformatted,
   you will see a diff between your code and properly formatted code.
   Note that a PR with code in need of reformatting cannot be merged.
   Once you are ready to apply the formatting changes, run:

   ```shell
   cargo fmt --all
   ```

   You won't see any output, but all your files will be corrected.

> Instructions taken in large part from [diesel-rs][]

[clippy]:           https://github.com/rust-lang/rust-clippy
[rust style guide]: https://github.com/rust-dev-tools/fmt-rfcs/blob/master/guide/guide.md
[rustfmt]:          https://github.com/rust-lang/rustfmt

## Links

* Repository: <https://github.com/victor-021/gengar-server>
* VaccMe Mobile Application: <https://github.com/horrokotoj/Gengar>
