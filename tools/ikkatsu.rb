require 'zip'
# gem rubyzip is required.

# example:
#  $ ruby ikkatsu.rb < token.txt

$token = gets.chomp  # read github token from stdin
$owner = "o-jill"
$repo = "ruversi"
$logfile = "ikkatsu.log"
$archive = "archive/"
$narchive = 23  # 99999999

def download(url, fn)
  FileUtils.mkpath($archive + File.dirname(fn))
  puts "download #{fn} to #{$archive} ..."
  system(
    [
      "curl", "-L", "--output", $archive + fn,
      "-H", "\"Accept: application/vnd.github+json\"", "-H",
      "\"Authorization: token #{$token}\"", url,
    ].join(' '))
end

def unzip(fn)
  dest = "kifu/"
  puts "unzip #{fn} to #{dest} ..."
  Zip::File.open($archive + fn) do |zip|
    zip.each do |entry|
      # p entry.name
      dir = File.join(dest, File.dirname(entry.name))
      FileUtils.mkpath(dir)
      zip.extract(entry, dest + entry.name) { true }
    end
  end
end

# curl \
#   -H "Accept: application/vnd.github+json" \ 
#   -H "Authorization: token <TOKEN>" \
#   https://api.github.com/repos/OWNER/REPO/actions/artifacts
def dl_list
  puts"downloading list..."
  system(
    [
      "curl", "-H", "\"Accept: application/vnd.github+json\"",
      # "-H", "\"Authorization: token #{$token}\"",
      "https://api.github.com/repos/#{$owner}/#{$repo}/actions/artifacts"
    ].join(' '), [:out, :err]=>[$logfile, "w"])
end


# puts "token[#{$token}]"

dl_list

fname = ""
File.open($logfile, mode = "rt"){|f|
  f.each_line{|line|
    # puts line
    # "name": "kifu-N9_20220720154803",
    n = line.match(/name": "(.+)",/)
    if n
      fname = n[1] + ".zip"
      next
    end

    # "archive_download_url": "https://api.github.com/repos/OWNER/REPO/actions/artifacts/ARTIFACT-ID/zip",
    m = line.match(/archive_download_url": "(http.+zip)/)
    next unless m

    next if fname == ""
    # puts m[1]
    download(m[1], fname)
    unzip(fname)

    $narchive -= 1
    break if $narchive.zero?
  }
}
