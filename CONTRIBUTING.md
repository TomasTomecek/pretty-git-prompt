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

## Generated files don't belong to the repository

Please never commit files which are generated while building the tool or
running the tests, most notably `target/` (Rust build artifacts), `__pycache__/`
and `*.pyc` (Python bytecode of the integration test suite) and `.pytest_cache/`.
All of them are listed in [`.gitignore`](./.gitignore); if you committed one by
accident, drop it from the index with
`git rm -r --cached path/to/generated/files`.

## Releasing

A release is a version bump merged through a pull request, a git tag, a GitHub
release, a crates.io upload and — from 0.2.2 on — an automated Fedora update.
Versions follow semver-ish `MAJOR.MINOR.PATCH` and both tags and releases are
named after the bare version, without a `v` prefix (`0.2.2`, not `v0.2.2`).

### 1. Prepare the release commit

Start from an up to date `master` and open a branch named after the version
(`0.2.3-release`, this is what past releases used):

```
$ git checkout master && git pull
$ git checkout -b 0.2.3-release
```

Bump the version in:

* `Cargo.toml` — the `version` field,
* `Cargo.lock` — run `cargo build` (or `cargo update -p pretty-git-prompt`) so
  the `pretty-git-prompt` entry matches, don't edit it by hand,
* [`rust-pretty-git-prompt.spec`](./rust-pretty-git-prompt.spec) — the
  `Version` field. `Release` is `%autorelease` and the changelog is
  `%autochangelog`, so nothing else needs to be touched there. (Releases before
  0.2.2 also reset `Release` to `1%{?dist}` and added a `%changelog` entry;
  this is no longer needed.)
* `README.md` — the download URLs in the installation section still contain the
  version, update them if you are going to attach binaries to the release.

Commit it as `0.2.3 release`, one commit, nothing else in it:

```
$ git commit -sam "0.2.3 release"
```

### 2. Verify and merge

Run `make test` locally and open the pull request. Wait for the packit jobs:
the `rpm-build:*` jobs prove that `cargo package` still produces a valid
archive and that the spec builds with the new version, `testing-farm:*` runs
the test plan against those RPMs. Merge once they are green.

### 3. Tag

Tag the merge commit on `master` and push the tag:

```
$ git checkout master && git pull
$ git tag 0.2.3
$ git push origin 0.2.3
```

### 4. GitHub release

Create the release from the tag
([releases page](https://github.com/TomasTomecek/pretty-git-prompt/releases)),
with the tag name as the title, and use GitHub's *Generate release notes* — the
0.2.2 notes are the generated list of merged pull requests, new contributors
and a `Full Changelog` compare link, so keep that shape.

Binaries are optional: 0.1.x and 0.2.0 shipped
`pretty-git-prompt-${VERSION}-${TARGET}` assets built by the `make release`
target (`LIBZ_SYS_STATIC=1 cargo build --target ${TARGET} --release`), which
used to be driven by Travis CI. Since 0.2.1 no assets are attached and users
are expected to install the RPM or build from source. If you do attach them,
build the target you want and upload the file:

```
$ make exec-release-build  # or: PROJECT_NAME=pretty-git-prompt TARGET=x86_64-unknown-linux-gnu TRAVIS_TAG=0.2.3 make release
$ gh release upload 0.2.3 pretty-git-prompt-0.2.3-x86_64-unknown-linux-gnu
```

### 5. crates.io

The crate is published manually (`cargo publish` needs a crates.io token with
publish rights for `pretty-git-prompt`):

```
$ cargo publish --dry-run
$ cargo publish
```

This is a separate step from the GitHub release and has historically lagged
behind it (0.2.2 was tagged in February 2024 and published in March 2024).
Publishing matters for Fedora: the spec uses `%{crates_source}`, so the
downstream build only works once the version is on crates.io.

### 6. Fedora

Publishing the GitHub release triggers packit's `propose_downstream` job, which
opens the dist-git pull request for `rawhide`; merging it triggers the
`koji_build` job. Both are configured in [`.packit.yaml`](./.packit.yaml).
Watch the packit dashboard for the project and fix the spec file upstream if
the downstream build fails. Older Fedora branches are not updated
automatically — if you want the new version there, submit the dist-git update
and a Bodhi update for those branches yourself.
