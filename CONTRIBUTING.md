# Contributing to pretty-git-prompt

Thank you for your interest! This document describes how to build, test and
ship changes to pretty-git-prompt.

## Support

This is an open source project. There is no guaranteed support, everything is
best effort. If you encounter an issue, please
[submit it](https://github.com/TomasTomecek/pretty-git-prompt/issues) — the
best thing to do in the meanwhile is to try fixing it yourself.

## Design principles

The project builds upon several principles:

1. Configurable as much as possible.
2. Pretty and useful.
3. As few dependencies as possible.
4. Easy to contribute to:
   * Build with a single command.
   * Build inside a predictable environment.
   * Test with a single command.

Please keep them in mind when proposing a change: a new runtime dependency, for
example, needs a good reason, and anything user-visible should be configurable
through the config file rather than hardcoded.

## Repository layout

| Path | Content |
| --- | --- |
| `src/main.rs` | CLI (clap) and glue between the modules |
| `src/backend.rs` | everything which talks to libgit2 |
| `src/models.rs` | the values which can be displayed and how they are rendered |
| `src/conf.rs` | config file parsing, validation and the default config |
| `src/constants.rs` | shared constants and the default config path |
| `src/util.rs` | the `log!` macro; must not import anything from the project |
| `tests/integration/` | pytest suite driving the built binary in real repositories |
| `tests/full.fmf`, `plans/main.fmf` | test metadata for tmt / Testing Farm |
| `files/` | shell configs used by the demo, sample colored configs |
| `.packit.yaml`, `rust-pretty-git-prompt.spec` | Fedora packaging and CI |

## Development environment

Everything runs in a container, so all you need is `podman` and `make`. The
image is built from [`Dockerfile`](./Dockerfile) (Fedora + cargo, clippy, git,
pytest, pexpect, zsh):

```
$ make build-environment
```

The container targets bind-mount the working directory to `/src`, so the build
artifacts land in your `target/` directory as usual.

If you have cargo available on your system, you can skip the container
altogether and use the `exec-*` targets, which run the very same commands
directly on your machine.

| In a container | Directly |
| --- | --- |
| `make build` (alias for `make release-build`) | `make exec-release-build` |
| `make debug-build` | `make exec-debug-build` |
| `make test` | `make exec-test` |

`make shell` opens a shell inside the build container, and `make zsh-demo` /
`make bash-demo` start an interactive prompt in a prepared git repository —
this is one of the ways the tool is verified to work.

## Tests

```
$ make test
```

runs both suites plus clippy (advisory for now, it does not fail the build):

* **Unit tests** (`cargo test`) live next to the code in `src/`. They must not
  depend on the machine they run on: create a git repository with the
  `init_git!` macro in a `TempDir` instead of using this repository, and point
  `XDG_CONFIG_HOME` at a temporary directory instead of reading the config of
  whoever runs the tests. Tests which set environment variables take
  `constants::ENV_LOCK`, because the environment is shared by all threads of
  the test binary.
* **Integration tests** (`pytest tests/integration`) build git repositories in
  various states and assert on the output of the `pretty-git-prompt` binary
  found in `$PATH`. When running them outside of the container, make sure the
  binary you want to test is the first one in `$PATH`:

  ```
  $ cargo build
  $ PATH="${PWD}/target/debug:${PATH}" pytest-3 -vv tests/integration
  ```

Please make sure both suites pass and that you are not introducing new
compiler warnings before opening a pull request.

## Continuous integration

CI is provided by [packit](https://packit.dev/) and runs on every pull request:

* **`rpm-build:*`** — builds the RPM in copr for every supported Fedora, using
  `cargo package` to produce the archive.
* **`testing-farm:*`** — installs those RPMs and executes the test plan in
  [`plans/main.fmf`](./plans/main.fmf), which discovers the tests tagged `full`
  in [`tests/full.fmf`](./tests/full.fmf).

Both suites are described in `tests/full.fmf`. Keep in mind when editing it:

* the tests run from the root of the repository (`path: /` on the parent node),
* the integration tests exercise the *installed* package, i.e. the copr build of
  the pull request, not a binary built during the test.

You can validate the metadata locally with `tmt lint` (`pip install tmt`).

## Submitting changes

* One logical change per commit; explain in the commit message *why* the change
  is needed, the diff already says what it does.
* Reference the issue you are fixing (`Fixes #123`) in the pull request.
* If the change is user-visible, document it in `README.md` and — when it
  concerns the output — in the comments of the default config in
  `src/conf.rs`, which is what users get from
  `pretty-git-prompt create-default-config`.

## Releasing

1. Bump `version` in `Cargo.toml` and tag the commit.
2. The binaries attached to a GitHub release are produced by `make release`,
   which builds with `LIBZ_SYS_STATIC=1` for the requested `${TARGET}` (this
   used to be wired up in [`.travis.yml`](./.travis.yml)).
3. `packit` then proposes the downstream update to Fedora rawhide and builds it
   in koji; the spec file lives in
   [`rust-pretty-git-prompt.spec`](./rust-pretty-git-prompt.spec) and is
   generated by `rust2rpm`.
