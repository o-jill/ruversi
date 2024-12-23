#!/bin/bash
#!/bin/bash -x

# REPEAT=5
REPEAT=20

DATESTR=`date +%Y%m%d%H%M%S`
RESULT=speedcheck${DATESTR}.txt

EVFILE=data/evaltable.txt

search() {
# SDEPTH="--depth 9"
SDEPTH="--depth 11"
# FEATURES=""  # sse
FEATURES="--features=avx"

RFENS=("8/8/8/3Aa3/3aA3/8/8/8 b" "8/8/8/3aA3/3Aa3/8/8/8 b" "A1A1A3/1c4/Aa1dA/1c4/A1a1a3/2a2a2/2a3a1/2A4A b")
LENRFENS=${#RFENS[@]}

for ((i=0;i<${LENRFENS};i++)) do
  echo -n "Begin RFEN:${RFENS[$i]}"
  echo "Begin RFEN:${RFENS[$i]}" >> ${RESULT}
  for ((j=0;j<${REPEAT};j++)) do
    RUSTFLAGS="-Ctarget-cpu=native" cargo run --release ${FEATURES} -- --rfen "${RFENS[$i]}" ${SDEPTH} --ev1 ${EVFILE} >> ${RESULT} 2>/dev/null
    echo -n " ${j}"
  done
  echo "End RFEN:${RFENS[$i]}" >> ${RESULT}
  echo "End RFEN:${RFENS[$i]}"
done

cat ${RESULT} | ruby ./tools/speedcheck.rb search
}

learn() {
LREPEAT="--repeat 1000"
# LREPEAT="--repeat 1000 --minibatch"
KIFUFILE="kifu/kifu00000000.txt"
# FEATURES=""  # sse
FEATURES="--features=avx"

RUSTFLAGS="-Ctarget-cpu=native" cargo build --release ${FEATURES}

  if ! [ -e $KIFUFILE ]; then
    echo "a kifu file was not found."
    echo "please prepare kifu files in ./kifu/ ."
    exit
  fi
#  echo "a kifu file was found."
  for ((j=0;j<${REPEAT};j++)) do
    echo -n " ${j}"
    RUSTFLAGS="-Ctarget-cpu=native" cargo run --release ${FEATURES} -- --learn ${LREPEAT} >> ${RESULT} 2>/dev/null
  done
  echo " ${REPEAT}"

cat ${RESULT} | ruby ./tools/speedcheck.rb learn
}

game() {
SDEPTH="--depth 7"
# FEATURES=""  # sse
FEATURES="--features=avx"
# DUELLV=1
DUELLV=2
# DUELLV=3

RUSTFLAGS="-Ctarget-cpu=native" cargo build --release ${FEATURES} 2>/dev/null

for ((j=0;j<${REPEAT};j++)) do
  STARTDT=`date +%s.%3N`

  RUSTFLAGS="-Ctarget-cpu=native" cargo run --release ${FEATURES} -- --duel "${DUELLV}" ${SDEPTH}  --ev1 ${EVFILE} --ev2 ${EVFILE} --silent >> ${RESULT} 2>/dev/null

  FINISHDT=`date +%s.%3N`
  DURATION=`echo "scale=3; ${FINISHDT} - ${STARTDT}" | bc`  # $((FINISHDT - STARTDT))
  echo "duration: ${DURATION} sec." >> ${RESULT}
  tail -n 1 ${RESULT}
done
tail -n 6 ${RESULT}
ruby ./tools/speedcheck.rb game < ${RESULT}
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
