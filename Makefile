ifeq ($(OS),Windows_NT)
	DOCKER = winpty docker
else
	DOCKER = docker
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
CONTAINER_NAME := smells_container1
SMELLS_DIR := $(shell realpath .)
CARGO_CACHE := $(shell realpath ~/.cargo)

DOCKER_RUN := $(DOCKER) run -t -i --rm \
	-v $(SMELLS_DIR):/smells \
	-v $(CARGO_CACHE):/usr/local/cargo/ \
	--name $(CONTAINER_NAME) $(IMAGE_NAME)
	

docker_build_image: ## Build an image from a Dockerfile
	$(DOCKER) build -t $(IMAGE_NAME) .

docker_tests: ## Create the container
	 $(DOCKER_RUN) cargo test

docker_shell: ## Run a shell in the container
	 $(DOCKER_RUN) bash

docker_stop_rm: ## Stop and remove the specified container
	$(DOCKER) stop $(CONTAINER_NAME) 
	$(DOCKER) rm $(CONTAINER_NAME)
