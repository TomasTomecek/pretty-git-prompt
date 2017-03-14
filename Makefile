# exec-* targets execute the target targetly
# rest is executed in a container
#
# stable container is meant to be used
# unstable is meant for development & testing

.PHONY=default compile build build-unstable-container build-stable-container stable-build unstable-build exec-stable-build exec-unstable-build test exec-test
DEFAULT_RUST_STABLE_VERSION="1.15.1"
DEFAULT_RUST_UNSTABLE_VERSION="nightly"
DEPS=src/cli.rs src/main.rs src/format.rs

default: build


compile: unstable-build

build: stable-build

build-stable-container:
	docker build --build-arg RUST_CHANNEL=$(DEFAULT_RUST_STABLE_VERSION) --build-arg WITH_TEST=no --tag ${USER}/pretty-git-prompt .
build-unstable-container:
	docker build --build-arg RUST_CHANNEL=$(DEFAULT_RUST_UNSTABLE_VERSION) --build-arg WITH_TEST=yes --tag ${USER}/pretty-git-prompt .

stable-build: build-stable-container
	docker run -v ${PWD}:/app:Z -v ~/.cargo/registry/:/root/.cargo/registry/:Z -ti ${USER}/pretty-git-prompt make exec-stable-build
unstable-build:
	docker run -v ${PWD}:/app:Z -v ~/.cargo/registry/:/root/.cargo/registry/:Z -ti ${USER}/pretty-git-prompt make exec-unstable-build

exec-stable-build: target/release/pretty-git-prompt

exec-unstable-build: target/debug/pretty-git-prompt

target/release/pretty-git-prompt: $(DEPS)
	cargo build --release
	rm -rf target/release/build/
target/debug/pretty-git-prompt: $(DEPS)
	cargo build
	rm -rf target/debug/build/


test:
	docker run -v ${PWD}:/app:Z -v ~/.cargo/registry/:/root/.cargo/registry/:Z -ti ${USER}/pretty-git-prompt make exec-test

exec-test: target/debug/pretty-git-prompt
	py.test-3 -vv tests
	cargo test
	cargo clippy
