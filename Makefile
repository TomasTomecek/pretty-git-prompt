# exec-* targets execute the target targetly
# rest is executed in a container
#
# stable container is meant to be used
# unstable is meant for development & testing

.PHONY=default compile build build-unstable-container build-stable-container stable-build unstable-build exec-stable-build exec-unstable-build test exec-test
RUST_STABLE_SPEC="1.15.1"
RUST_UNSTABLE_SPEC="nightly-2017-03-16"
DEPS=$(wildcard src/*.rs)
CURRENT_USER="$(shell id -u)"
STABLE_BUILD_IMAGE="${USER}/pretty-git-prompt"
UNSTABLE_BUILD_IMAGE="${USER}/pretty-git-prompt:dev"
STABLE_CONTAINER_RUN=docker run --rm -v ${PWD}:/app:Z -v ~/.cargo/registry/:/home/pretty/.cargo/registry/:Z -ti $(STABLE_BUILD_IMAGE)
UNSTABLE_CONTAINER_RUN=docker run --rm -v ${PWD}:/app:Z -v ~/.cargo/registry/:/home/pretty/.cargo/registry/:Z -ti $(UNSTABLE_BUILD_IMAGE)

default: build


compile: unstable-build

build: stable-build

build-stable-container:
	docker build --build-arg USER_ID=$(CURRENT_USER) --build-arg RUST_SPEC=$(RUST_STABLE_SPEC) --build-arg WITH_TEST=no --tag $(STABLE_BUILD_IMAGE) .
build-unstable-container:
	docker build --build-arg USER_ID=$(CURRENT_USER) --build-arg RUST_SPEC=$(RUST_UNSTABLE_SPEC) --build-arg WITH_TEST=yes --tag $(UNSTABLE_BUILD_IMAGE) .

stable-build: build-stable-container
	$(STABLE_CONTAINER_RUN) make exec-stable-build
unstable-build:
	$(UNSTABLE_CONTAINER_RUN) make exec-unstable-build

exec-stable-build: target/release/pretty-git-prompt

exec-unstable-build: target/debug/pretty-git-prompt

target/release/pretty-git-prompt: $(DEPS)
	cargo build --release
target/debug/pretty-git-prompt: $(DEPS)
	cargo build


test:
	$(UNSTABLE_CONTAINER_RUN) make exec-test

exec-test: target/debug/pretty-git-prompt
	py.test-3 -vv tests
	cargo test
	$(shell cargo clippy || :)

# compile and inject into container
# open prompt with prepared git repo
# TODO: interactively run commands and see how prompt changes
use-case-1:
	$(UNSTABLE_CONTAINER_RUN) tests/functional/zsh.sh


show-work:
	egrep -C 3 "(TODO|FIXME)" $(DEPS) Makefile Dockerfile
