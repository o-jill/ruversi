# example:
#  $ ruby speedcheck.rb < speedcheck.txt

def read(lines)
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
            puts "#{nodes} nodes / #{'%.2f' % avg} +- #{'%.2f' % sd} msec (#{min} -- #{max})"
            next
        end
    }
end

read(readlines)
