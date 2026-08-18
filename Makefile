# exec-* targets execute commands of the target directly
# rest is executed in a container built from ./Dockerfile
#
# TODO: cache build container: run it and exec statements inside
#                              or figure out bind-mounted cargo cache

.PHONY: default build build-environment release-build debug-build \
        exec-release-build exec-debug-build test exec-test \
        zsh-demo bash-demo shell release
DEPS=$(wildcard src/*.rs)
BUILD_IMAGE="docker.io/tomastomecek/pretty-git-prompt"
CONTAINER_RUN=podman run --rm -v ${PWD}:/src:Z -w /src -i $(BUILD_IMAGE)
# for targets which need a terminal: podman run -t fails without one
CONTAINER_RUN_TTY=podman run --rm -v ${PWD}:/src:Z -w /src -ti $(BUILD_IMAGE)

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
	py.test-3 -vv tests/integration
	cargo test --verbose
	# linting is advisory for now
	-cargo clippy

# compile and inject into container
# open prompt with prepared git repo
zsh-demo:
	$(CONTAINER_RUN_TTY) files/demo.py zsh
bash-demo:
	$(CONTAINER_RUN_TTY) files/demo.py bash


shell:
	$(CONTAINER_RUN_TTY) zsh -l

release:
	cargo build --target ${TARGET} --release
	cp -av target/${TARGET}/release/${PROJECT_NAME} "${PROJECT_NAME}-${VERSION}-${TARGET}"
