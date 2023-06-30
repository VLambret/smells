#! /bin/bash

set -u



title='Influence of the number of files on time execution, with '${LINE_COUNT_OF_FILE}' lines by folder and a depth of '${DEPTH}''
x_axis_label='Time (sec)'
y_axis_label='Number of files in root'

grep "real" "smells_times${ANALYSIS_NUMBER}.txt" | sed 's/[^0-9.]//g' | awk '{print $0, "'${i}'"}' >>  "smells_times_tmp${ANALYSIS_NUMBER}.txt"
#grep "user" smells_times.txt | sed 's/[^0-9.]//g'| sed "s/^/${i} /"  >> smells_times_tmp.txt
#grep "sys" smells_times.txt | sed 's/[^0-9.]//g'| sed 's/^/3 /'  >> smells_times_tmp.txt

'/c/Program Files/gnuplot/bin/gnuplot' -p -e "set xlabel '${x_axis_label}'; set ylabel '${y_axis_label}'; set title '${title}'; plot [0:0.5] 'smells_times_tmp${ANALYSIS_NUMBER}.txt' u 1:2 w lines"