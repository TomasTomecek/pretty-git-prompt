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

Only the first two steps are done by hand: bump the version and merge it.
Merging the release commit into `master` starts
[`.github/workflows/release.yml`](./.github/workflows/release.yml), which tags
it and does the GitHub release, the binaries and the crates.io upload; Fedora
then follows from the release event via packit.

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

### 3. Watch the release workflow

Merging the pull request triggers the `release` workflow, which

1. tags the merge as `0.2.3` and pushes the tag — this is the `Tag the release`
   job, it is what decides that a push to `master` is a release at all: it
   takes the version from `Cargo.toml` and continues only if the push contains
   the matching `0.2.3 release` commit, so any other push to `master` ends
   there. An already existing tag is reused rather than moved,
2. creates the GitHub release as a *draft*, titled after the tag, with GitHub's
   generated release notes (the same shape as the 0.2.2 notes: merged pull
   requests, new contributors, a `Full Changelog` compare link),
3. builds `LIBZ_SYS_STATIC=1 cargo build --target ${TARGET} --release` for
   `x86_64-unknown-linux-gnu`, `x86_64-apple-darwin` and
   `aarch64-apple-darwin` and attaches the binaries as
   `pretty-git-prompt-${VERSION}-${TARGET}`, the names 0.1.x and 0.2.0 used,
4. publishes the release once all the assets are attached — it is deliberately
   drafted first, publishing is what packit reacts to and the release should
   not be seen without its binaries,
5. runs `cargo publish`, skipping it if the version is already on crates.io.

Pushing the tag by hand (`git tag 0.2.3 && git push origin 0.2.3`) still works
and starts the same workflow, which then refuses to continue if the tag does
not equal `version` in `Cargo.toml`. A tag pushed by the workflow itself does
not start a second run of it — GitHub does not trigger workflows from events
made with the automatic `GITHUB_TOKEN` — which is why the tagging is a job of
this workflow instead of a separate one.

The same workflow can be started manually (*Actions* → *release* → *Run
workflow*) with an existing tag as the input, e.g. to redo a run that failed
halfway through. Everything in it is idempotent: an existing tag and release
are reused, assets are re-uploaded with `--clobber` and an already published
crate is left alone.

If you need a binary locally, that is still `make exec-release-build` or
`PROJECT_NAME=pretty-git-prompt TARGET=x86_64-unknown-linux-gnu
VERSION=0.3.0 make release`.

No crates.io API token is stored anywhere: the workflow uses [trusted
publishing](https://crates.io/docs/trusted-publishing), i.e. it exchanges the
job's OIDC token (`permissions: id-token: write`) for a short-lived registry
token via `rust-lang/crates-io-auth-action`. This requires the crate owner
(@TomasTomecek) to have registered this repository and the `release` workflow as
a trusted publisher in the crates.io settings of `pretty-git-prompt`; if that is
missing, the `crates-io` job fails while the GitHub release itself is already
done. Publishing matters for Fedora: the spec uses `%{crates_source}`, so the
downstream build only works once the version is on crates.io. Historically this
step lagged behind the tag by weeks (0.2.2 was tagged in February 2024 and
published in March 2024), which is the main reason it is automated now.

### 4. Fedora

Publishing the GitHub release triggers packit's `propose_downstream` job, which
opens the dist-git pull request for `rawhide`; merging it triggers the
`koji_build` job. Both are configured in [`.packit.yaml`](./.packit.yaml).
Watch the packit dashboard for the project and fix the spec file upstream if
the downstream build fails. Older Fedora branches are not updated
automatically — if you want the new version there, submit the dist-git update
and a Bodhi update for those branches yourself.
