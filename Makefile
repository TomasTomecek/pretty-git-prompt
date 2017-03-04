.PHONY=build build-container

default: build

compile:
	docker run -v ${PWD}:/app -v ~/.cargo:/root/.cargo -ti ${USER}/pretty-git-prompt

build: build-container
	docker run -v ${PWD}:/app -ti ${USER}/pretty-git-prompt

build-container:
	docker build --tag ${USER}/pretty-git-prompt .

test: compile
	py.test-3 -vv tests
