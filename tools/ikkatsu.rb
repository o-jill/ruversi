require 'net/http'
require 'json'
require 'fileutils'
require 'zip'
# gem rubyzip is required.

# usage:
#  $ ruby tools/ikkatsu.rb < token.txt

$token = gets&.chomp || ""
$owner = "o-jill"
$repo = "ruversi"
$logfile = "ikkatsu.log"
$archive = "archive/"
$narchive = 200  # limit how many archives to download (set large if you want all)
$target_downloads = 100
$downloads = 0
$unzipped = 0
$pagemax = 3

def dl_list_all
  page = 1
  File.open($logfile, "w") do |f|
    loop do
      uri = URI("https://api.github.com/repos/#{$owner}/#{$repo}/actions/artifacts?per_page=100&page=#{page}")
      req = Net::HTTP::Get.new(uri)
      req['Accept'] = 'application/vnd.github+json'
      req['Authorization'] = "token #{$token}" unless $token.empty?

      res = Net::HTTP.start(uri.host, uri.port, use_ssl: true) do |http|
        http.request(req)
      end

      if res.is_a?(Net::HTTPSuccess)
        f.puts res.body
        json = JSON.parse(res.body)
        artifacts = json['artifacts'] || []
        puts "page #{page}: #{artifacts.size} artifacts"
        break if artifacts.empty?
        # Save artifacts info lines for later parsing (existing script expects name/...archive_download_url lines)
        artifacts.each do |a|
          # Write entries in similar patterns to original parsing (name and archive_download_url lines)
          f.puts %Q{  "name": "#{a['name']}",}
          f.puts %Q{  "archive_download_url": "#{a['archive_download_url']}",}
        end

        page += 1
        break if page > $pagemax
      else
        warn "Failed to fetch page #{page}: #{res.code} #{res.message}"
        warn "Response body: #{res.body}"
        break
      end
    end
  end
end

def download(url, fn)
  return false unless fn.start_with?('kifu-')

  $downloads += 1

  archive_path = File.join($archive, fn)
  if File.exist?(archive_path)
    puts "already downloaded #{fn}, skipping."
    return false
  end

  FileUtils.mkpath(File.dirname(archive_path))
  puts "download #{fn} to #{$archive} ..."
  # Use array form of system to avoid shell quoting issues
  cmd = [
    "curl", "-L", "--fail", "--silent", "--show-error",
    "--output", archive_path,
    "-H", "Accept: application/vnd.github+json",
  ]
  cmd += ["-H", "Authorization: token #{$token}"] unless $token.empty?
  cmd << url

  ok = system(*cmd)
  unless ok
    warn "curl failed for #{fn} (#{url})"
  end
  ok
end

def unzip(fn)
  return unless fn.start_with?('kifu')

  $unzipped += 1

  dest = "kifu/"
  puts "unzip #{fn} to #{dest} ..."
  Zip::File.open(File.join($archive, fn)) do |zip|
    zip.each do |entry|
      dir = File.join(dest, File.dirname(entry.name))
      FileUtils.mkpath(dir)
      zip.extract(entry, File.join(dest, entry.name)) { true }
    end
  end
end

# Run: get all pages and write a simple logfile similar to original format
dl_list_all

fname = ""
dl_table = Array.new($target_downloads)
File.open($logfile, mode = "rt"){|f|
  f.each_line{|line|
    # "name": "kifu-N9_20220720154803",
    n = line.match(/name": "(.+)",/)
    if n
      num = n[1].match(/N(\d+)_/)
      unless num
        fname = ''
        next
      end
      # puts "#{n[1]}, num:#{num[1]}"
      # puts "dl_table:#{dl_table}"
      idx = num[1].to_i
      if idx >= $target_downloads  # invalid number!
        print "ERROR: idx:#{idx} >= #{$target_downloads}"
        fname = ''
        next
      end
      # puts "idx:#{idx} dl_table[idx]:#{dl_table[idx]}"
      if dl_table[idx]
        fname = ''
        next
      end

      dl_table[idx] = 1 # mark as downloaded
      fname = n[1] + ".zip"
      next
    end

    # "archive_download_url": "https://api.github.com/repos/OWNER/REPO/actions/artifacts/ARTIFACT-ID/zip",
    m = line.match(/archive_download_url": "(http.+zip)/)
    next unless m

    next if fname == ""
    if download(m[1], fname)
      unzip(fname)
    end

    $narchive -= 1
    break if $narchive <= 0
    fname = ""
  }
}

puts "downloaded: #{$downloads}"
puts "unzipped: #{$unzipped}"
