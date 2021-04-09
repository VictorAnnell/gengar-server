# Gengar Server
> VaccMe server back-end written in Rust by group Gengar

The server back-end to the [VaccMe mobile application] written in Rust for the [Computer Systems with Project Work] course at [Uppsala University], spring 2021.

[VaccMe mobile application]: https://github.com/horrokotoj/Gengar
[Computer Systems with Project Work]: https://www.uu.se/en/admissions/master/selma/kursplan/?kpid=39194
[Uppsala University]: https://www.uu.se/en

## Developing

### Setting up project locally

1. Install Rust using [rustup].

2. Clone this repository.

3. Create a `.env` file in the repository's root directory, and add the connection details for your databases.

	 *Note:* If you set up MySQL manually and didn't specify the MySQL user to be one with elevated
	 permissions, you'll want to run a command like ```mysql -c "GRANT ALL ON
	 `gengar_dev`.* TO ''@'localhost';" -uroot```, or something similar for the
	 user that you've specified.

	 If you have [Docker] and [Docker Compose] you can use the provided docker-compose.yml file to automatically set up a MySQL container to work against. You can start the container with this command from the repositories root directory:

	```shell
	$ docker-compose up -d
	```

	To stop the container:

	```shell
	$ docker-compose stop
	```

	To stop and remove the container and the associated docker volume:

	```shell
	$ docker-compose down -v
	```
	See [.env.sample](.env.sample) for an example that works with this docker setup.

4. With this done try running the test suite to confirm everything works:
	```
	$ cargo test
	```

> Instructions taken in large part from [diesel-rs]

[rustup]: https://rustup.rs/
[Docker]: https://www.docker.com/
[Docker Compose]: https://docs.docker.com/compose/install/
[diesel-rs]: https://github.com/diesel-rs/diesel/blob/master/CONTRIBUTING.md

### Building

Build and run the project with cargo:

```shell
$ cargo run
```

Replace `run` with `build` if you want to build but not run.

## Contributing

### Coding Style

The code follows the [Rust Style Guide], enforced using [rustfmt], with additional linting provided by [clippy].

To run formatting tests locally:

1. Install rustfmt and clippy (if they aren't already) by running:
   ```
   $ rustup component add rustfmt
   $ rustup component add clippy
   ```

2. Run clippy using cargo from the root of the project repo.
   ```
   $ cargo clippy --all-targets --all-features
   ```
   Each PR needs to compile without warning.

3. Run rustfmt using cargo from the root of your diesel repo.

   To see changes that need to be made, run:

   ```
   $ cargo fmt --all -- --check
   ```

   If all code is properly formatted (e.g. if you have not made any changes),
   this should run without error or output.
   If your code needs to be reformatted,
   you will see a diff between your code and properly formatted code.
   Note that a PR with code in need of reformatting cannot be merged.
   Once you are ready to apply the formatting changes, run:

   ```
   $ cargo fmt --all
   ```

   You won't see any output, but all your files will be corrected.

> Instructions taken in large part from [diesel-rs]

[Rust Style Guide]: https://github.com/rust-dev-tools/fmt-rfcs/blob/master/guide/guide.md
[rustfmt]: https://github.com/rust-lang/rustfmt
[clippy]: https://github.com/rust-lang/rust-clippy

## Links

- Repository: https://github.com/victor-021/gengar-server
- VaccMe Mobile Application: https://github.com/horrokotoj/Gengar
