# example:
#  $ ruby speedcheck.rb < speedcheck.txt

def search(lines)
    values = []
    nodes = 0
    lines.each {|line|
        if line.start_with?("val:")
            # puts line
# val:-31.888197 20237 nodes. @@d3[]e3@@f2[]c2@@d2[]c3@@b1[]d1@@c1 11msec
            m = / (\d+) nodes\. .+ (\d+)msec/.match(line)
            nodes = m[1].to_i
            values.push(m[2].to_i)
            next
        elsif line.start_with?("Begin RFEN:")
            values = []
            nodes = 0
            next
        elsif line.start_with?("End RFEN:")
            avg = values.sum.fdiv(values.length)
            vari = values.map {|elem| elem * elem}.sum.fdiv(values.length) - avg * avg
            sd = Math.sqrt(vari)
            min, max = values.minmax
            puts "speed: #{'%.2f' % (nodes / avg)} nodes/msec"
            puts "#{nodes} nodes / #{'%.2f' % avg} ± #{'%.2f' % sd} msec (#{min} -- #{max})"
            next
        end
    }
end

def learn(lines)
    values = []
    files = 0
    repeat = 0
    lines.each {|line|
        if line.start_with?("processing time:")
# processing time: 0min 6.0sec (602.6msec/batch, 0.069msec/file)
            m = / (\d+)min (\d+\.\d)sec/.match(line)
            values.push(m[1].to_i * 60 + m[2].to_f)
            next
        elsif line.start_with?("total,")
# total,8687,win,1630,draw,56,lose,7001
            m = /total.(\d+),/.match(line)
            files = m[1].to_i
            next
        elsif line =~ /^\d+ \/ \d+\r/
            repeat = line.split(' ')[-1].to_i
            next
        end
    }
    avg = values.sum.fdiv(values.length)
    vari = values.map {|elem| elem * elem}.sum.fdiv(values.length) - avg * avg
    sd = Math.sqrt(vari)
    min, max = values.minmax
    puts "#{'%.1f' % avg } ± #{'%.1f' % sd} sec (#{min} -- #{max}) for #{files} files x #{repeat} repeats"
    print "speed: #{'%.1f' % (avg * 1000.0 / repeat)} msec/batch, "
    puts "#{'%.1f' % (avg * 1000000.0 / repeat / files)} usec/file"
end

# total,8,win,4,draw,0,lose,4,balance,0,8,50.00%,R,+0.0
# ev1 @@,win,0,draw,0,lose,4
# ev1 [],win,4,draw,0,lose,0
# ev1:data/evaltable.txt
# ev2:data/evaltable.txt
# duration: 14 sec.
def game(lines)
    value = 0;
    games = 1;
    lines.each {|line|
        puts line

        if line.start_with?("duration:")
            # duration: 14 sec.
            m = / (\d+) sec/.match(line)
            value = m[1].to_f
            next
        elsif line.start_with?("total,")
            # total,8687,win,1630,draw,56,lose,7001
            m = /total.(\d+),/.match(line)
            games = m[1].to_i
            next
        end
    }
    puts "#{'%.2f' % (value / games)} sec/game = #{value} / #{games}"
end

def help()
    puts "ruby #{__FILE__} <mode>"
    puts "  search : summarize searching speed."
    puts "  learn : summarize learning speed."
    puts "  game : summarize game speed."
    puts "  help : show this help."
end

# main
md = ARGV[0] || "help"

if md == "search"
    search($stdin.read.split("\n"))
elsif md == "learn"
    learn($stdin.read.split("\n"))
elsif md == "game"
    game($stdin.read.split("\n"))
elsif md == "help"
    help()
else
    help()
end
