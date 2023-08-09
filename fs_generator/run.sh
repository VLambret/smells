#!/bin/bash

NUMBER_OF_FILES=$1

SCRIPT_DIR=$(dirname $0)
cd $SCRIPT_DIR

./target/release/fs_generator $NUMBER_OF_FILES 1 1 0
time ../target/release/smells ./root_directory/ > /dev/null
mv root_directory/ root_directory_"${NUMBER_OF_FILES}"
