#! /bin/bash

set -u

NUMBER_OF_FILES_BY_FOLDER=$1
LINE_COUNT=$2
DEPTH=$3

if [ -z "$NUMBER_OF_FILES_BY_FOLDER" ]
then
  echo "usage: $0 <number of files>" >&2
  exit 42
fi

FOLDER_NAME="fake_project_with_${NUMBER_OF_FILES_BY_FOLDER}_files_by_folder_and_${DEPTH}_folders_deep"

rm -rf "$FOLDER_NAME"
mkdir -p "$FOLDER_NAME"
# shellcheck disable=SC2164
cd "$FOLDER_NAME"
for h in $(seq 1 "$DEPTH")
do
  mkdir -p "$FOLDER_NAME${h}"
  # shellcheck disable=SC2164
  cd "$FOLDER_NAME${h}"
  # shellcheck disable=SC2086
  for i in $(seq 1 $NUMBER_OF_FILES_BY_FOLDER)
  do
      for j in $(seq 1 "$LINE_COUNT")
      do
        echo "line : $j" >> "fake_file${i}"
      done
    done
done


