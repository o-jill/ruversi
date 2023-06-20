#!/bin/bash -x


DEPTH=11

DATESTR=`date +%Y%m%d%H%M%S`
RESULT=speedcheck${DATESTR}.txt

RFENS=("8/8/8/3Aa3/3aA3/8/8/8 b" "8/8/8/3aA3/3Aa3/8/8/8 b" "A1A1A3/1c4/Aa1dA/1c4/A1a1a3/2a2a2/2a3a1/2A4A b")
LENRFENS=${#RFENS[@]}

for ((i=0;i<${LENRFENS};i++)) do
  echo "Begin RFEN:${RFENS[$i]}" >> ${RESULT}
  for ((j=0;j<20;j++)) do
    RUSTFLAGS="-Ctarget-cpu=native" cargo run --release --features=avx -- --rfen "${RFENS[$i]}" --depth ${DEPTH} >> ${RESULT} 2>/dev/null
  done
  echo "End RFEN:${RFENS[$i]}" >> ${RESULT}
done

cat ${RESULT} | ruby ./tools/speedcheck.rb
