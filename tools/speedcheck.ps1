$REPEAT = 20
$DATESTR = Get-Date -Format "yyyyMMddHHmmss"
$RESULT = "speedcheck${DATESTR}.txt"
$script_name = Split-Path -Leaf $PSCommandPath # スクリプト名を取得(バージョン3.0以降)

$env:RUSTFLAGS = "-Ctarget-cpu=native"

function search() {
  # $SDEPTH = "--depth 9"
  $SDEPTH = "--depth 11"
  # $FEATURES = ""  # sse
  $FEATURES = "--features=avx"

  $RFENS = @("8/8/8/3Aa3/3aA3/8/8/8 b", "8/8/8/3aA3/3Aa3/8/8/8 b", "A1A1A3/1c4/Aa1dA/1c4/A1a1a3/2a2a2/2a3a1/2A4A b")

  foreach ($rf in $RFENS) {
    Write-Output "Begin RFEN:${rf}" >> ${RESULT}
    for ($i = 0; $i -lt 10; $i++) {
      cargo run --release ${FEATURES} -- --rfen $rf ${SDEPTH} >> ${RESULT} 2> $null
    }
    Write-Output "End RFEN:${rf}" >> ${RESULT}
  }

  Get-Content ${RESULT} | ruby ./tools/speedcheck.rb search
}

function learn() {
  $LREPEAT = "--repeat 20"
  $KIFUFILE = "kifu/kifu0000000.txt"

  if (!(Test-Path ${KIFUFILE})) {
    Write-Output "a kifu file was not found."
    Write-Output "please prepare kifu files in ./kifu/ ."
    exit
  }
  # Write-Output "a kifu file was found."

  for($i = 0; $i -lt ${REPEAT}; $i++) {
    cargo run --release --features=avx -- --learn ${LREPEAT} >> ${RESULT} 2> $null
  }

  Get-Content ${RESULT} | ruby ./tools/speedcheck.rb learn
}

function game() {
  # $SDEPTH = "--depth 5"
  $SDEPTH = "--depth 7"
  # $FEATURES = ""  # sse
  $FEATURES = "--features=avx"
  # $DUELLV = 1
  $DUELLV = 2
  # $DUELLV = 3

  cargo build --release ${FEATURES}

  $STARTDT = Get-Date

  # for ($i = 0; $i -lt ${REPEAT}; $i++) {
  cargo run --release ${FEATURES} -- --duel "${DUELLV}" ${SDEPTH} --ev1 data/evaltable.txt --ev2 data/evaltable.txt >> ${RESULT} 2> $null
  # }

  $FINISHDT = Get-Date
  $DURATION = ($FINISHDT - $STARTDT).TotalSeconds
  Write-Output "duration: ${DURATION} sec." >> ${RESULT}
  Get-Content -Tail 6 ${RESULT} | ruby ./tools/speedcheck.rb game
}

function help() {
  Write-Host "${script_name} <mode>"
  Write-Host "mode:"
  Write-Host "  search : measure searching speed."
  Write-Host "  learn : measure learning speed."
  Write-Host "  game : measure game(duel) speed."
  Write-Host "  help : show this help."
}

if ($Args[0] -eq "search") {
  search
} elseif ($Args[0] -eq "learn") {
  learn
} elseif ($Args[0] -eq "game") {
  game
} else {
  help
}
