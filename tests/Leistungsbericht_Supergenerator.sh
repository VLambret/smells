#! /bin/bash

set -u

function find_smells_executable() {
	ROOT_FOLDER="$(realpath $(dirname $0))/../"
	SMELLS=$(find "${ROOT_FOLDER}/target/debug/" -maxdepth 1 -executable -name "smells*")
}

find_smells_executable

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
  (/bin/time -f %e "$SMELLS" "$FAKE_FOLDER") 2> "$RESULT_FILE" > /dev/null

}

# Flat file system
for FILE_NUMBER in 1000
do
  run_analyses $FILE_NUMBER 1 1 1 "flat_file_system.$FILE_NUMBER"
  #run_analyses $FILE_NUMBER 1 500 1 "flat_medium_file_system.$FILE_NUMBER"
done
  exit


for DEPTH in 50
do
  run_analyses 100 $DEPTH 500 2 "linear_depth.$DEPTH"
done
