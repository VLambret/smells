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

IMAGE_NAME := rust:1.70-bookworm
SMELLS_DIR := $(shell realpath .)

DOCKER_RUN := docker run --rm \
	-v $(SMELLS_DIR):/smells \
	-v cargo_cache:/usr/local/cargo/registry \
	--workdir=/smells \
	$(IMAGE_NAME)

d_build: 
	 $(EXEC_CMD) "$(DOCKER_RUN) make build"

d_test: 
	 $(EXEC_CMD) "$(DOCKER_RUN) make test"

d_shell:
	 $(DOCKER_RUN) bash

################################################################################
# PERF
################################################################################*

perf_test:
	cargo build --release
	./fs_generator/run.sh 2000

perf_analysis:
	cargo build --release
	heaptrack ./target/release/smells ./fs_generator/root_directory_4000
