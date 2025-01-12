#!/usr/bin/ruby

# example:
#  $ ruby runspeedcheck.rb <option>

REPEAT=5
# REPEAT=20

EVFILE='data/evaltable.txt'

DATESTR=Time.now.strftime('%Y%m%d%H%M%S')
RESULT="speedcheck#{DATESTR}.txt"

# 時間計測
#
# returns: [sec_elapsed, result_yield]
def elapsed_time_of(message = '')
  starting = Process.clock_gettime(Process::CLOCK_MONOTONIC)
  ret = yield
  ending = Process.clock_gettime(Process::CLOCK_MONOTONIC)
  sec_elapsed = ending - starting
  # p "#{message} spend #{(sec_elapsed * 1000).floor(2)}ms"

  [sec_elapsed, ret]
end

# コマンドの実行
def exec_command(cmd)
  result = `#{cmd}`
  raise RuntimeError, "`#{cmd}` is failed." unless $?.success?
  result
end

# tailコマンドの代わり
def tail(fname, lines)
  ret = nil
  File.open(fname) do |io|
    ret = io.reverse_each().lazy().first(lines).reverse()
  end

  ret
end

# stdoutとfnameにmsgを改行付き出力
def echo(msg, fname)
  # puts(msg)
  print(msg)

  File.open(fname, 'a') do |f|
    f.puts(msg)
  end
end

# Hello, reversi world!
# mode:Rfen
# read eval table: data/evaltable.txt
# |__|__|__|__|__|__|__|__|
# |__|__|__|__|__|__|__|__|
# |__|__|__|__|__|__|__|__|
# |__|__|__|@@|[]|__|__|__|
# |__|__|__|[]|@@|__|__|__|
# |__|__|__|__|__|__|__|__|
# |__|__|__|__|__|__|__|__|
# |__|__|__|__|__|__|__|__|
# @@'s turn.
# val:26.44183 257230 nodes. @@d6[]a4@@b6[]a2@@b8[]a7@@a6[]e7@@a8[]00@@e6[]00@@e4 337msec
#
# returns: [number of nodes, elapsed time in msec]
def readresult(line)
  m = / (\d+) nodes.+ (\d+)msec/.match(line)
  nodes = m[1].to_i
  elapsed = m[2].to_i

  [nodes, elapsed]
end

# calc statistics
#
# nodes: # of nodes
# elapsed: array of elapsed time in msec
def searchresult(nodes, elapsed)
  avg = elapsed.sum.fdiv(elapsed.length)
  vari = elapsed.map {|elem| elem * elem}.sum.fdiv(elapsed.length) - avg * avg
  sd = Math.sqrt(vari)
  min, max = elapsed.minmax

  puts "speed: #{'%.2f' % (nodes / avg)} nodes/msec"
  puts "#{nodes} nodes / #{'%.2f' % avg} ± #{'%.2f' % sd} msec (#{min} -- #{max})"
end

def search()
  # sdepth = '--depth 9'
  sdepth = '--depth 11'
  # features = ''  # sse
  # features = '--features=avx'
  features = ENV['FEATURES']  # read feature from env
  puts "features: #{features}"
  rfens = [
    "8/8/8/3Aa3/3aA3/8/8/8 b", "8/8/8/3aA3/3Aa3/8/8/8 b",
    "A1A1A3/1c4/Aa1dA/1c4/A1a1a3/2a2a2/2a3a1/2A4A b"
  ].freeze
  result_nodes = Array.new(rfens.length(), -1)
  result_elapsed = Array.new(rfens.length()) {[]}

  rfens.each_with_index do |rfen, i|
    echo("Begin RFEN:#{rfen}", RESULT)
    for j in 0..REPEAT do
      # runcmd = "cargo run --release #{features} -- --rfen \"#{rfen}\" #{sdepth} --ev1 #{EVFILE} >> #{RESULT} 2>/dev/null"
      runcmd = "cargo run --release #{features} -- --rfen \"#{rfen}\" #{sdepth} --ev1 #{EVFILE} 2>/dev/null"
      res = exec_command(runcmd)
      (nodes, elapsed) = readresult(res.split("\n").last())
      result_nodes[i] = nodes
      result_elapsed[i] << elapsed
      print(" #{j}")
    end
    echo("End RFEN:#{rfen}", RESULT)
  end

  result_nodes.each_with_index do |nd, i|
    searchresult(nd, result_elapsed[i])
  end
end

def learn()
  puts("deprecated.")
end

# total,8,win,4,draw,0,lose,4,balance,0,8,50.00%,R,+0.0
# ev1 @@,win,0,draw,0,lose,4
# ev1 [],win,4,draw,0,lose,0
# ev1:data/evaltable.txt
# ev2:data/evaltable.txt
def gameresult(lines, elapsed)
  # p lines
  value = 0;
  games = 1;
  lines.each {|line|
    puts line

    if line.start_with?("total,")
      # ex.
      # total,8687,win,1630,draw,56,lose,7001
      m = /total.(\d+),/.match(line)
      games = m[1].to_i
      break
    end
  }
  msec = ((elapsed * 10000) + 0.5).floor() * 0.1
  puts("#{'%.2f' % (msec / games)} msec/game = #{msec} / #{games}")
end

def game()
  # sdepth = "--depth 5"
  sdepth = "--depth 7"

  # features = ""  # sse
  # features = "--features=avx"
  features = ENV["FEATURES"]  # read feature from env
  puts "features: #{features}"

  # duellv=1
  duellv=2
  # duellv=3

  buildcmd = "cargo build --release #{features}"
  exec_command(buildcmd)

  txtout = nil
  for j in 0..REPEAT do
    print("#{j} ")
    elapsed, _res = elapsed_time_of() do
      # runcmd = "cargo run --release --silent #{features} -- --duel #{duellv} #{sdepth}  --ev1 #{evfile} --ev2 #{evfile} >> #{RESULT} 2>/dev/null"
      runcmd = "cargo run --release #{features} -- --silent --duel #{duellv} #{sdepth}  --ev1 #{EVFILE} --ev2 #{EVFILE} 2>/dev/null"
      txtout = exec_command(runcmd)
      # echo(txtout, RESULT)
    end
 
    # gameresult(tail(RESULT, 5), elapsed)
    gameresult(txtout.split("\n").reverse, elapsed)
  end
end

def help()
  puts "#{$0} <mode>"
  puts "mode:"
  puts "  search : measure searching speed."
  puts "  learn : [no longer supported!] measure learning speed."
  puts "  game : measure game(duel) speed."
  puts "  help : show this help."
end

# main
md = ARGV[0] || "help"

if md == "search"
    search()
elsif md == "learn"
    learn()
elsif md == "game"
    game()
elsif md == "help"
    help()
else
    help()
end
