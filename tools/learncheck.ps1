# download kifus from github and learn them, evaluate the results.

# quit if kifu/newevaltable.txt exist
$knevtbl = "kifu/newevaltable"
$FILE = "${knewvaltbl}.txt"
function downloadkifu() {
  # check if $FILE exists
  if (Test-Path ${FILE}) {
    Write-Output "${FILE} exists."
    Write-Output "please clean up kifu/."
    exit 255
  }

  # download kifus
  ruby tools/ikkatsu.rb < ghpw.txt
}

downloadkifu

$env:RUSTFLAGS = "-Ctarget-cpu=native"

# $ETA = 0.01
$DEPTH = 7
# $FEATURES = ""  # sse
$FEATURES = "--features=avx"

# start learning

$BASEWEIGHT = "data/evaltable.txt"

# $ARRETA = @("0.19", "0.51", "0.23", "0.1123")
# $ARRREPEAT = @("5", "9", "12", "15", "20")
# $PROGRESS = "5,9,12,15"
$ARRETA = @("0.0007", "0.0005", "0.0003", "0.0001")
$ARRREPEAT = @("1000", "1400", "2000", "3000", "5000")
# $PROGRESS = "1000,1400,2000,3000"
$PROGRESS = $ARRREPEAT[0..($ARRREPEAT.count - 2)] -join ','

$REPEAT = $ARRREPEAT[-1]

function learn_duel($eta) {
  # learn
  cargo run --release ${FEATURES} -- --learn --eta ${eta} --repeat ${REPEAT} --ev1 ${BASEWEIGHT} --progress ${PROGRESS}

  # rename weights
  for ($j = 0; j -lt $ARRREPEAT.count - 1; $j++) {
    $rp = ${ARRREPEAT[$j]}
    Move-Item -Force -Path ${knevtbl}.r${rp}.txt -Destination ${knevtbl}-e$1-r${rp}.txt
  }
  Copy-Item -Force -Path ${FILE} -Destination ${knevtbl}-e${eta}-r${REPEAT}.txt

  # check if learned well
  foreach ($rpt in $ARRREPEAT) {
    cargo run --release ${FEATURES} -- --duel --ev1 ${BASEWEIGHT} --ev2 ${knevtbl}-e$1-r${rpt}.txt --depth ${DEPTH} | tee-object duel-$1-o-${rpt}.txt
  }
}

foreach ($eta in $ARRETA) {
  learn_duel ${eta}
}

# summarize results
$DATESTR = Get-Date -Format "yyyyMMddHHmmss"
$RESULT = "training-${DATESTR}.txt"
foreach ($eta in $ARRETA) {
  foreach ($rpt in $ARRREPEAT) {
    Get-Content -Tail 5 duel-${eta}-o-${rpt}.txt >> ${RESULT}
  }
}

Get-Content ${RESULT}
