#! /bin/bash

set -u

RESULT_FOLDER="data/generated_results"
mkdir -p $RESULT_FOLDER

function run_analyses() {
      NUMBER_OF_FILES_BY_FOLDER_VAR=$1
      DEPTH_VAR=$2
      LINE_COUNT_OF_FILE_VAR=$3
      CASE_NAME=$5
      FAKE_FOLDER="data/generated/$CASE_NAME"
      EXECUTION_ID=$(date +%s.%3N)
      RESULT_FILE="${RESULT_FOLDER}/${CASE_NAME}.${EXECUTION_ID}.time"

  ./create_fake_project.sh "${NUMBER_OF_FILES_BY_FOLDER_VAR}" "${DEPTH_VAR}" "${LINE_COUNT_OF_FILE_VAR}" "${FAKE_FOLDER}"
  (/bin/time -f %e ../target/debug/smells.exe "$FAKE_FOLDER") 2> "$RESULT_FILE"

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