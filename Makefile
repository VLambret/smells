ifeq ($(OS),Windows_NT)
	EXEC_CMD = winpty powershell -command
else
	EXEC_CMD = sh -c
endif

.PHONY: build test

all: build test

build: 
	cargo build

test: 
	cargo test

################################################################################
# BACKLOG
################################################################################

backlog: backlog.png
	eog backlog.png

backlog.png: backlog.dot
	dot -Tpng $^ -o $@

backlog.dot: backlog.py
	python3 $^ > $@
	cat $@

################################################################################
# DOCKER
################################################################################*

IMAGE_NAME := smells-test:latest
SMELLS_DIR := $(shell cygpath -w $(shell realpath .))

DOCKER_RUN := docker run -t -i --rm \
	-v $(SMELLS_DIR):/smells \
	-v cargo_cache:/usr/local/cargo/registry \
	$(IMAGE_NAME)

d_build_image: 
	docker build -t $(IMAGE_NAME) .

d_build: 
	 $(EXEC_CMD) "$(DOCKER_RUN) make build"

d_test: 
	 $(EXEC_CMD) "$(DOCKER_RUN) make test"

d_shell:
	 $(DOCKER_RUN) bash
