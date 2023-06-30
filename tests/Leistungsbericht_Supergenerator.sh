#! /bin/bash

set -u


function run_analyses() {
      NUMBER_OF_FILES_BY_FOLDER_VAR=$1
      DEPTH_VAR=$2
      LINE_COUNT_OF_FILE_VAR=$3
      TESTED_PARAMETER_VAR=$4
      OUTPUT_FILE=$5

      ./Leistungsberichtsgenerator.sh "${NUMBER_OF_FILES_BY_FOLDER_VAR}" "${DEPTH_VAR}" "${LINE_COUNT_OF_FILE_VAR}" "${TESTED_PARAMETER_VAR}" "${NUMBER_OF_ANALYSES}" "${OUTPUT_FILE}"
}

# Flat file system
for FILE_NUMBER in 100 1000 5000 100000
do
  run_analyses $FILE_NUMBER 1 1 1 "flat_file_system.$FILE_NUMBER"
  run_analyses $FILE_NUMBER 1 500 1 "flat_medium_file_system.$FILE_NUMBER"
done


for DEPTH in 50
do
  run_analyses 100 $DEPTH 500 2 "linear_depth.$DEPTH"
done