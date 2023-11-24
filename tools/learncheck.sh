#!/bin/bash -x

# quit if kifu/newevaltable.txt exist

FILE="kifu/newevaltable.txt"

downloadkifu() {
# check if $FILE exists
if [ -e $FILE ]; then
  echo ${FILE}" exists."
  echo "please clean up kifu/."
  exit 255
fi

# download kifus
ruby tools/ikkatsu.rb < ghpw.txt
}

downloadkifu

# ETA=0.01
DEPTH=7
DUELLVL=5

# start learning
# original
# RUSTFLAGS="-Ctarget-cpu=native" cargo run --release -- --learn --repeat 1 --eta 0.000001 --ev1 RANDOM
# BASEWEIGHT=kifu/newevaltable.txt.orig
# mv kifu/newevaltable.txt ${BASEWEIGHT}
# RUSTFLAGS="-Ctarget-cpu=native" cargo run --release --features=avx -- --duel --ev1 RANDOM --ev2 ${BASEWEIGHT} --depth ${DEPTH}| tee duel-${ETA}-r-orig.txt
BASEWEIGHT=data/evaltable.txt
# BASECOMMAND=RUSTFLAGS="-Ctarget-cpu=native" cargo run --release --features=avx --
# ARRETA=(0.19 0.51 0.23 0.1123)
# ARRREPEAT=(5 9 12 15 20)
# PROGRESS="5,9,12,15"
# REPEAT=20
ARRETA=(0.0007 0.0005 0.0003 0.0001)
LENETA=${#ARRETA[@]}
ARRREPEAT=(1000 1400 2000 3000 5000)
LENREPEAT=${#ARRREPEAT[@]}
LENREPEATM1=$((${LENREPEAT} - 1))
REPEAT=${ARRREPEAT[${LENREPEATM1}]}
PROGRESS=`echo "${ARRREPEAT[*]}" | cut -d " " -f 1-${LENREPEATM1} | sed 's/ /,/g'`

# $1: eta
learn_duel() {
# learn
RUSTFLAGS="-Ctarget-cpu=native" cargo run --release --features=avx -- --learn --eta $1 --repeat ${REPEAT} --ev1 ${BASEWEIGHT} --progress ${PROGRESS}
# rename weights
for ((j=0;j<${LENREPEATM1};j++)) do
  mv -f kifu/newevaltable.r${ARRREPEAT[$j]}.txt kifu/newevaltable-e$1-r${ARRREPEAT[$j]}.txt
done
cp -f ${FILE} kifu/newevaltable-e$1-r${REPEAT}.txt

# check if learned well
for ((j=0;j<${LENREPEAT};j++)) do
  RUSTFLAGS="-Ctarget-cpu=native" cargo run --release --features=avx -- --duel ${DUELLVL} --ev1 ${BASEWEIGHT} --ev2 kifu/newevaltable-e$1-r${ARRREPEAT[$j]}.txt --depth ${DEPTH}| tee duel-$1-o-${ARRREPEAT[$j]}.txt
done
}

for ((i=0;i<${LENETA};i++)) do
  learn_duel ${ARRETA[$i]}
done

# summarize results
DATESTR=`date +%Y%m%d`
RESULT=training-${DATESTR}.txt
for ((i=0;i<${LENETA};i++)) do
  for ((j=0;j<${LENREPEAT};j++)) do
    tail -n 5 duel-${ARRETA[$i]}-o-${ARRREPEAT[$j]}.txt >> ${RESULT}
  done
done
cat $RESULT
