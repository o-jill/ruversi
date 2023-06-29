#!/bin/bash -x

# REPEAT=5
REPEAT=20

DATESTR=`date +%Y%m%d%H%M%S`
RESULT=speedcheck${DATESTR}.txt

search() {
# SDEPTH="--depth 9"
SDEPTH="--depth 11"
# FEATURES=""  # sse
FEATURES="--features=avx"

RFENS=("8/8/8/3Aa3/3aA3/8/8/8 b" "8/8/8/3aA3/3Aa3/8/8/8 b" "A1A1A3/1c4/Aa1dA/1c4/A1a1a3/2a2a2/2a3a1/2A4A b")
LENRFENS=${#RFENS[@]}

for ((i=0;i<${LENRFENS};i++)) do
  echo "Begin RFEN:${RFENS[$i]}" >> ${RESULT}
  for ((j=0;j<${REPEAT};j++)) do
    RUSTFLAGS="-Ctarget-cpu=native" cargo run --release ${FEATURES} -- --rfen "${RFENS[$i]}" ${SDEPTH} >> ${RESULT} 2>/dev/null
  done
  echo "End RFEN:${RFENS[$i]}" >> ${RESULT}
done

cat ${RESULT} | ruby ./tools/speedcheck.rb search
}

learn() {
LREPEAT="--repeat 20"
KIFUFILE="kifu/kifu0000000.txt"

  if ! [ -e $KIFUFILE ]; then
    echo "a kifu file was not found."
    echo "please prepare kifu files in ./kifu/ ."
    exit
  fi
#  echo "a kifu file was found."
  for ((j=0;j<${REPEAT};j++)) do
    RUSTFLAGS="-Ctarget-cpu=native" cargo run --release --features=avx -- --learn ${LREPEAT} >> ${RESULT} 2>/dev/null
  done

cat ${RESULT} | ruby ./tools/speedcheck.rb learn
}

game() {
# SDEPTH="--depth 5"
SDEPTH="--depth 7"
# FEATURES=""  # sse
FEATURES="--features=avx"
# DUELLV=1
DUELLV=2
# DUELLV=3

RUSTFLAGS="-Ctarget-cpu=native" cargo build --release ${FEATURES}

STARTDT=`date +%s`

# for ((j=0;j<${REPEAT};j++)) do
RUSTFLAGS="-Ctarget-cpu=native" cargo run --release ${FEATURES} -- --duel "${DUELLV}" ${SDEPTH} --ev1 data/evaltable.txt --ev2 data/evaltable.txt >> ${RESULT} 2>/dev/null
# done

FINISHDT=`date +%s`
DURATION=$((FINISHDT - STARTDT))
echo "duration: ${DURATION} sec." >> ${RESULT}
tail -n 6 ${RESULT} | ruby ./tools/speedcheck.rb game
}

help() {
  echo "$0 <mode>"
  echo "mode:"
  echo "  search : measure searching speed."
  echo "  learn : measure learning speed."
  echo "  game : measure game(duel) speed."
  echo "  help : show this help."
}

if [ "$1" = "search" ] ; then
  search
elif [ "$1" = "learn" ]; then
  learn
elif [ "$1" = "game" ]; then
  game
else
  help
fi
