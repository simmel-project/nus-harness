#!/bin/sh
baud_rate=8013
f_hi=21000
f_lo=19000
sample_rate=62500
filter_size=9

echo "Baud,Silence,Errors" > log.csv
for silence_prefix in $(seq 0 1 20)
do
    for baud_rate in $(seq 7990 0.5 8010)
    do
        ../nus-harness/target/release/nus-tests.exe \
                    --input reference/reference.bin \
                    --output reference/generated-afsk.wav \
                    -r $sample_rate \
                    -h $f_hi \
                    -l $f_lo \
                    -b $baud_rate \
                    --silence-prefix $silence_prefix
        ./afsktest reference/generated-afsk.wav $sample_rate $baud_rate $f_lo $f_hi $filter_size > /dev/null
        echo "$baud_rate, $silence_prefix, $?" >> log.csv
    done
done

gnuplot-qt <<EOF
set key autotitle columnhead
set title 'Varying baud rate and silence prefix with F\_LO = $f_lo and F\_HI = $f_hi at $sample_rate Hz'
set xlabel 'baud rate'
set ylabel '% of packets decoded'
plot 'log.csv' using 1:(\$2==0?\$3:1/0) with lines title '0 samples', \
     'log.csv' using 1:(\$2==1?\$3:1/0) with lines title '1 samples', \
     'log.csv' using 1:(\$2==2?\$3:1/0) with lines title '2 samples', \
     'log.csv' using 1:(\$2==3?\$3:1/0) with lines title '3 samples', \
     'log.csv' using 1:(\$2==4?\$3:1/0) with lines title '4 samples', \
     'log.csv' using 1:(\$2==5?\$3:1/0) with lines title '5 samples', \
     'log.csv' using 1:(\$2==6?\$3:1/0) with lines title '6 samples', \
     'log.csv' using 1:(\$2==7?\$3:1/0) with lines title '7 samples', \
     'log.csv' using 1:(\$2==8?\$3:1/0) with lines title '8 samples', \
     'log.csv' using 1:(\$2==9?\$3:1/0) with lines title '9 samples', \
     'log.csv' using 1:(\$2==10?\$3:1/0) with lines title '10 samples', \
     'log.csv' using 1:(\$2==11?\$3:1/0) with lines title '11 samples', \
     'log.csv' using 1:(\$2==12?\$3:1/0) with lines title '12 samples', \
     'log.csv' using 1:(\$2==13?\$3:1/0) with lines title '13 samples', \
     'log.csv' using 1:(\$2==14?\$3:1/0) with lines title '14 samples', \
     'log.csv' using 1:(\$2==15?\$3:1/0) with lines title '15 samples', \
     'log.csv' using 1:(\$2==16?\$3:1/0) with lines title '16 samples', \
     'log.csv' using 1:(\$2==17?\$3:1/0) with lines title '17 samples', \
     'log.csv' using 1:(\$2==18?\$3:1/0) with lines title '18 samples', \
     'log.csv' using 1:(\$2==19?\$3:1/0) with lines title '19 samples', \
     'log.csv' using 1:(\$2==20?\$3:1/0) with lines title '20 samples'
pause mouse close "Close the window to continue\n"
EOF

# set logscale y
# plot 'log.csv' using 1:($5==6?$6:1/0) with lines title 'Filter size 6', \
#      'log.csv' using 1:($5==7?$6:1/0) with lines title 'Filter size 7', \
#      'log.csv' using 1:($5==8?$6:1/0) with lines title 'Filter size 8', \
#      'log.csv' using 1:($5==9?$6:1/0) with lines title 'Filter size 9', \
#      'log.csv' using 1:($5==10?$6:1/0) with lines title 'Filter size 10'
# pause -1 "Hit return to continue"
