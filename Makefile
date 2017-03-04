.PHONY=build build-container

default: build

compile:
	docker run -v ${PWD}:/app -ti ${USER}/pretty-git-prompt

build: build-container
	docker run -v ${PWD}:/app -ti ${USER}/pretty-git-prompt

build-container:
	docker build --tag ${USER}/pretty-git-prompt .
