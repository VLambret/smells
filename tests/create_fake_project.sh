#! /bin/bash

set -u

NUMBER_OF_FILES_BY_FOLDER=$1
DEPTH=$2
LINE_COUNT=$3
FOLDER_NAME=$4

if [ -z "$NUMBER_OF_FILES_BY_FOLDER" ]
then
  echo "usage: $0 <number of files>" >&2
  exit 42
fi

if [ -z "$DEPTH" ]
then
  echo "usage: $0 <depth>" >&2
  exit 42
fi

if [ -z "$FOLDER_NAME" ]
then
  echo "usage: $0 <folder_name>" >&2
  exit 42
fi

if [ -d "$FOLDER_NAME" ]
then
  exit 0
fi

rm -f "$FOLDER_NAME"
mkdir -p "$FOLDER_NAME"
cd "$FOLDER_NAME" || exit 43
for h in $(seq 1 "$DEPTH")
do
  mkdir -p "$FOLDER_NAME${h}"
  cd "$FOLDER_NAME${h}" || exit 43
  for i in $(seq 1 $NUMBER_OF_FILES_BY_FOLDER)
  do
      for j in $(seq 1 "$LINE_COUNT")
      do
        echo "line : $j"
      done > "fake_file${i}"
    done
done


