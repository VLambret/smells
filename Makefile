.PHONY: build test

all: build test

build:
	cargo build

test:
	mkdir -p tests/data/empty_folder
	mkdir -p tests/data/folder_with_one_empty_folder/empty_folder
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
