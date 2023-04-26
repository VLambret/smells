.PHONY: build test

all: build test

build:
	cargo build

test:
	cargo test --offline

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

IMAGE_NAME:=smells:latest
CONTAINER_NAME:=smells

docker_build_image: ## Build an image from a Dockerfile
	docker build -t $(IMAGE_NAME) .
 
docker_shell: ## Start a shell inside the container
	echo $(PWD)
	docker run -it --rm --name $(CONTAINER_NAME) $(IMAGE_NAME) /bin/bash