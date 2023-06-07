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

DOCKER_RUN := winpty docker run -t -i -v C:\Users\Lucas\git\smells:/app --rm \
	--name $(CONTAINER_NAME) $(IMAGE_NAME)

docker_build_image: ## Build an image from a Dockerfile
	winpty docker build -t $(IMAGE_NAME) .

docker_tests: ## Create the container
	 $(DOCKER_RUN) cargo test

docker_shell: ## Run a shell in the container
	 $(DOCKER_RUN) bash

docker_stop_rm: ## Stop and remove the specified container
	winpty docker stop $(CONTAINER_NAME) && docker rm $(CONTAINER_NAME)
