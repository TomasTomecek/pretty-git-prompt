# exec-* targets execute commands of the target directly
# rest is executed in a container
#
# stable container produces binaries which are meant to be used in production
# nightly container is meant for development & testing (b/c of clippy)
#
# TODO: cache build container: run it and exec statements inside
#                              or figure out bind-mounted cargo cache

.PHONY=default compile build stable-environment nightly-environment stable-build nightly-build exec-stable-build exec-nightly-build test exec-test
DEPS=$(wildcard src/*.rs)
BUILD_IMAGE="docker.io/tomastomecek/pretty-git-prompt"
CONTAINER_RUN=podman run --rm -v ${PWD}:/src:Z -w /src -ti $(BUILD_IMAGE)

default: build


build: release-build

build-environment:
	podman build --tag $(BUILD_IMAGE) .

release-build: build-environment
	$(CONTAINER_RUN) make exec-release-build
debug-build: build-environment
	$(CONTAINER_RUN) make exec-debug-build

exec-release-build: target/release/pretty-git-prompt

exec-debug-build: target/debug/pretty-git-prompt

target/release/pretty-git-prompt: $(DEPS)
	LIBZ_SYS_STATIC=1 cargo build --release
target/debug/pretty-git-prompt: $(DEPS)
	cargo build -vvvv


test:
	$(CONTAINER_RUN) make exec-test

exec-test: target/debug/pretty-git-prompt
	py.test-3 -vv tests
	cargo test --verbose
	$(shell cargo clippy || :)

# compile and inject into container
# open prompt with prepared git repo
zsh-demo:
	$(CONTAINER_RUN) files/demo.py zsh
bash-demo:
	$(CONTAINER_RUN) files/demo.py bash


shell:
	$(CONTAINER_RUN) zsh -l

release:
	cargo build --target ${TARGET} --release
	cp -av target/${TARGET}/release/${PROJECT_NAME} "${PROJECT_NAME}-${TRAVIS_TAG}-${TARGET}"
