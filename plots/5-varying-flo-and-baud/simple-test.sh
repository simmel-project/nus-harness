#!/bin/sh
baud_rate=8013
f_hi=21000
f_lo=19000
sample_rate=62500
filter_size=9

# echo "Baud, Errors" > log.csv
# for f_lo in $(seq 18990 19010)
# do
#     for baud_rate in $(seq 7900 8100)
#     do
#         ../ltc-modulate/target/release/ltc-modulate.exe \
#                     --input reference/reference.bin \
#                     --output reference/generated-afsk.wav \
#                     -r $sample_rate \
#                     -h $f_hi \
#                     -l $f_lo \
#                     -b $baud_rate
#         # echo ./afsktest reference/generated-afsk.wav $sample_rate $baud_rate $f_lo $f_hi $filter_size

#         ./afsktest reference/generated-afsk.wav $sample_rate $baud_rate $f_lo $f_hi $filter_size > /dev/null
#         echo "$baud_rate, $f_lo, $?" >> log.csv
#     done
# done

gnuplot-qt <<EOF
set key autotitle columnhead
set title 'Varying baud rate and F\_LO with F\_HI = $f_hi at 44100 Hz'
set xlabel 'baud rate'
set ylabel '% of packets decoded'
plot 'log.csv' using 1:(\$2==18990?(\$3*100/51):1/0) with lines title 'F\_LO = 18990', \
     'log.csv' using 1:(\$2==18991?(\$3*100/51):1/0) with lines title 'F\_LO = 18991', \
     'log.csv' using 1:(\$2==18992?(\$3*100/51):1/0) with lines title 'F\_LO = 18992', \
     'log.csv' using 1:(\$2==18993?(\$3*100/51):1/0) with lines title 'F\_LO = 18993', \
     'log.csv' using 1:(\$2==18994?(\$3*100/51):1/0) with lines title 'F\_LO = 18994', \
     'log.csv' using 1:(\$2==18995?(\$3*100/51):1/0) with lines title 'F\_LO = 18995', \
     'log.csv' using 1:(\$2==18996?(\$3*100/51):1/0) with lines title 'F\_LO = 18996', \
     'log.csv' using 1:(\$2==18997?(\$3*100/51):1/0) with lines title 'F\_LO = 18997', \
     'log.csv' using 1:(\$2==18998?(\$3*100/51):1/0) with lines title 'F\_LO = 18998', \
     'log.csv' using 1:(\$2==18999?(\$3*100/51):1/0) with lines title 'F\_LO = 18999', \
     'log.csv' using 1:(\$2==19000?(\$3*100/51):1/0) with lines title 'F\_LO = 19000', \
     'log.csv' using 1:(\$2==19001?(\$3*100/51):1/0) with lines title 'F\_LO = 19001', \
     'log.csv' using 1:(\$2==19002?(\$3*100/51):1/0) with lines title 'F\_LO = 19002', \
     'log.csv' using 1:(\$2==19003?(\$3*100/51):1/0) with lines title 'F\_LO = 19003', \
     'log.csv' using 1:(\$2==19004?(\$3*100/51):1/0) with lines title 'F\_LO = 19004', \
     'log.csv' using 1:(\$2==19005?(\$3*100/51):1/0) with lines title 'F\_LO = 19005', \
     'log.csv' using 1:(\$2==19006?(\$3*100/51):1/0) with lines title 'F\_LO = 19006', \
     'log.csv' using 1:(\$2==19007?(\$3*100/51):1/0) with lines title 'F\_LO = 19007', \
     'log.csv' using 1:(\$2==19008?(\$3*100/51):1/0) with lines title 'F\_LO = 19008', \
     'log.csv' using 1:(\$2==19009?(\$3*100/51):1/0) with lines title 'F\_LO = 19009', \
     'log.csv' using 1:(\$2==19010?(\$3*100/51):1/0) with lines title 'F\_LO = 19010'
 pause mouse close "Close the window to continue\n"
EOF

# set logscale y
# plot 'log.csv' using 1:($5==6?$6:1/0) with lines title 'Filter size 6', \
#      'log.csv' using 1:($5==7?$6:1/0) with lines title 'Filter size 7', \
#      'log.csv' using 1:($5==8?$6:1/0) with lines title 'Filter size 8', \
#      'log.csv' using 1:($5==9?$6:1/0) with lines title 'Filter size 9', \
#      'log.csv' using 1:($5==10?$6:1/0) with lines title 'Filter size 10'
# pause -1 "Hit return to continue"
