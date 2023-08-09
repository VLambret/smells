#!/bin/bash

NUMBER_OF_FILES=$1

SCRIPT_DIR=$(dirname $0)
cd $SCRIPT_DIR

if [ -z "$NUMBER_OF_FILES" ]
then
	echo "usage: $0 <NUMBER_OF_FILE>" >&2
	exit 42
fi

GENERATED_FOLDER=root_directory_"${NUMBER_OF_FILES}"

if [ ! -d $GENERATED_FOLDER ]
then
	./target/release/fs_generator $NUMBER_OF_FILES 1 1 0
	mv root_directory/ $GENERATED_FOLDER
fi

time ../target/release/smells $GENERATED_FOLDER > /dev/null
