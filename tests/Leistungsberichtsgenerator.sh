#! /bin/bash

set -u

NUMBER_OF_FILES_BY_FOLDER=$1
DEPTH=$2
LINE_COUNT_OF_FILE=$3
TESTED_PARAMETER=$4
ANALYSIS_NUMBER=$5

#if [ -f "smells_times_tmp'${ANALYSIS_NUMBER}'.txt" ]; then
#  rm smells_times_tmp.txt
#fi
#
#if [ -f "smells_times'${ANALYSIS_NUMBER}'.txt" ]; then
#  rm smells_times.txt
#fi

if [ "$TESTED_PARAMETER" -eq 1 ]; then
    tested_variable=$NUMBER_OF_FILES_BY_FOLDER
    step=$((NUMBER_OF_FILES_BY_FOLDER / 25))
elif [ "$TESTED_PARAMETER" -eq 2 ]; then
      tested_variable=$DEPTH
    step=$((DEPTH / 25))
elif [ "$TESTED_PARAMETER" -eq 3 ]; then
    tested_variable=$LINE_COUNT_OF_FILE
    step=$((LINE_COUNT_OF_FILE / 25))
else
    echo "Invalid value for TESTED_PARAMETER"
    exit 1
fi


for i in $(seq 1 $step "${tested_variable}")
do
  if [ "$TESTED_PARAMETER" -eq 1 ]; then
    ./create_fake_flat_project_files.sh "${i}" "${DEPTH}" "${LINE_COUNT_OF_FILE}"
    (time ../target/debug/smells.exe "fake_project_with_${i}_files_with_${LINE_COUNT_OF_FILE}_lines_by_folder_and_${DEPTH}_folders_deep") 2> "smells_times${ANALYSIS_NUMBER}.txt"
    title='Influence of the number of files on time execution, with '${LINE_COUNT_OF_FILE}' lines by folder and a depth of '${DEPTH}''
    x_axis_label='Time (sec)'
    y_axis_label='Number of files in root'

  elif [ "$TESTED_PARAMETER" -eq 2 ]; then
    ./create_fake_flat_project_files.sh "${NUMBER_OF_FILES_BY_FOLDER}" "${i}" "${LINE_COUNT_OF_FILE}"
    (time ../target/debug/smells.exe "fake_project_with_${NUMBER_OF_FILES_BY_FOLDER}_files_with_${LINE_COUNT_OF_FILE}_lines_by_folder_and_${i}_folders_deep") 2> "smells_times${ANALYSIS_NUMBER}.txt"
    title='Influence of the depth on time execution, with '${NUMBER_OF_FILES_BY_FOLDER}' files by folder and '${LINE_COUNT_OF_FILE}' lines by file'
    x_axis_label='Time (sec)'
    y_axis_label='Depth'


  elif [ "$TESTED_PARAMETER" -eq 3 ]; then
    ./create_fake_flat_project_files.sh "${NUMBER_OF_FILES_BY_FOLDER}" "${DEPTH}" "${i}"
    (time ../target/debug/smells.exe "fake_project_with_${NUMBER_OF_FILES_BY_FOLDER}_files_with_${i}_lines_by_folder_and_${DEPTH}_folders_deep") 2> "smells_times${ANALYSIS_NUMBER}.txt"
    title='Influence of the number of lines on time execution, with '${NUMBER_OF_FILES_BY_FOLDER}' files by folder and a depth of '${DEPTH}''
    x_axis_label='Time (sec)'
    y_axis_label='Number of lines by files'

  else
        echo "Invalid value for TESTED_PARAMETER"
        exit 1
  fi

grep "real" "smells_times${ANALYSIS_NUMBER}.txt" | sed 's/[^0-9.]//g' | awk '{print $0, "'${i}'"}' >>  "smells_times_tmp${ANALYSIS_NUMBER}.txt"
#grep "user" smells_times.txt | sed 's/[^0-9.]//g'| sed "s/^/${i} /"  >> smells_times_tmp.txt
#grep "sys" smells_times.txt | sed 's/[^0-9.]//g'| sed 's/^/3 /'  >> smells_times_tmp.txt

done
'/c/Program Files/gnuplot/bin/gnuplot' -p -e "set xlabel '${x_axis_label}'; set ylabel '${y_axis_label}'; set title '${title}'; plot [0:0.5] 'smells_times_tmp${ANALYSIS_NUMBER}.txt' u 1:2 w lines"