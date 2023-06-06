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

docker_build_image: ## Build an image from a Dockerfile
	winpty docker build -t $(IMAGE_NAME) .

docker_shell: ## Start a shell inside the container
	echo $(PWD)
	winpty docker run -it --rm --name $(CONTAINER_NAME) $(IMAGE_NAME) bash

docker_tests: ## Create the container
	winpty docker run -t -i -v C:\Users\Lucas\git\smells:/app --rm \
	--name $(CONTAINER_NAME) $(IMAGE_NAME)

docker_bash: ## Run the existing container
	winpty docker exec -it $(CONTAINER_NAME) bash

docker_stop_rm: ## Stop and remove the specified container
	winpty docker stop $(CONTAINER_NAME) && docker rm $(CONTAINER_NAME)
