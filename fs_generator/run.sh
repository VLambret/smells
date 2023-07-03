#!/bin/bash

NUMBER_OF_FILES=$1

./target/release/fs_generator.exe $NUMBER_OF_FILES 2 1 0
time ../target/debug/smells.exe ./root_directory/ > /dev/null
mv root_directory/ root_directory_"${NUMBER_OF_FILES}"
