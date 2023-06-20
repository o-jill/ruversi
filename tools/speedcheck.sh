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

help() {
  echo "$0 <mode>"
  echo "mode:"
  echo "  search : measure searching speed."
  echo "  learn : measure learning speed."
  echo "  game : not yet."
  echo "  help : show this help."
}

if [ "$1" = "search" ] ; then
  search
elif [ "$1" = "learn" ]; then
  learn
elif [ "$1" = "game" ]; then
  help
else
  help
fi
