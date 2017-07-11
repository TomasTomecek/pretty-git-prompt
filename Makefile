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
STABLE_BUILD_IMAGE="${USER}/pretty-git-prompt"
NIGHTLY_BUILD_IMAGE="${USER}/pretty-git-prompt:dev"
STABLE_CONTAINER_RUN=docker run --rm -v ${PWD}:/src:Z -ti $(STABLE_BUILD_IMAGE)
# breaks CI: -v ~/.cargo/registry/:/home/pretty/.cargo/registry/:Z
NIGHTLY_CONTAINER_RUN=docker run --rm -v ${PWD}:/src:Z -ti $(NIGHTLY_BUILD_IMAGE)

default: build


compile: nightly-build

build: stable-build

stable-environment:
	docker build --tag $(STABLE_BUILD_IMAGE) .
nightly-environment:
	docker build --tag $(NIGHTLY_BUILD_IMAGE) -f ./Dockerfile.dev .

stable-build: stable-environment
	$(STABLE_CONTAINER_RUN) make exec-stable-build
nightly-build:
	$(NIGHTLY_CONTAINER_RUN) make exec-nightly-build

exec-stable-build: target/release/pretty-git-prompt

exec-nightly-build: target/debug/pretty-git-prompt

export GIT_REPO_IS_DIRTY := $(shell if git diff-index --quiet HEAD -- ; then echo "no"; else echo "yes"; fi)
ifeq ($(origin TRAVIS_COMMIT), undefined)
	export TRAVIS_COMMIT := $(shell git rev-parse --short HEAD)
endif
target/release/pretty-git-prompt: $(DEPS)
	LIBZ_SYS_STATIC=1 cargo build --release
target/debug/pretty-git-prompt: $(DEPS)
	cargo build -vvvv


test:
	$(NIGHTLY_CONTAINER_RUN) make exec-test

exec-test: target/debug/pretty-git-prompt
	py.test-3 -vv tests
	cargo test --verbose
	$(shell cargo clippy || :)

# compile and inject into container
# open prompt with prepared git repo
zsh-demo:
	$(NIGHTLY_CONTAINER_RUN) files/demo.py zsh
bash-demo:
	$(NIGHTLY_CONTAINER_RUN) files/demo.py bash


shell:
	$(NIGHTLY_CONTAINER_RUN) zsh -l

show-work:
	egrep --color=yes -C 3 "(TODO|FIXME)" $(DEPS) Makefile Dockerfile


release:
	cargo build --target ${TARGET} --release
	cp -av target/${TARGET}/release/${PROJECT_NAME} "${PROJECT_NAME}-${TRAVIS_TAG}-${TARGET}"
