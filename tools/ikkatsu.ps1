# ikkatsu.ps1
#   converted from ikkatsu.rb by Gemini.

# GitHub Token を環境変数から読み込むことを推奨します。
# 例: $env:GH_TOKEN = "your_github_personal_access_token"
# または、スクリプト実行前にコマンドラインで設定: $GH_TOKEN = "your_token"
$GitHubToken = $env:GH_TOKEN
if ([string]::IsNullOrEmpty($GitHubToken)) {
    Write-Warning "GitHub Token (env:GH_TOKEN) が設定されていません。リストのダウンロードは認証なしで試行されますが、プライベートリポジトリやダウンロードには認証が必要です。"
    # 必要であれば、ここでユーザーにトークン入力を促すことも可能です。
    # $GitHubToken = Read-Host -Prompt "GitHub Personal Access Token を入力してください"
}

$Owner = "o-jill"
$Repo = "ruversi"
$LogFile = "ikkatsu.log"
$ArchiveDir = "archive" # ダウンロード先ディレクトリ
$ExtractDir = "kifu"   # 解凍先ディレクトリ
$NArchive = 105        # ダウンロードするアーカイブの最大数

# archive ディレクトリと kifu ディレクトリが存在しない場合は作成
if (-not (Test-Path $ArchiveDir)) {
    New-Item -ItemType Directory -Path $ArchiveDir | Out-Null
}
if (-not (Test-Path $ExtractDir)) {
    New-Item -ItemType Directory -Path $ExtractDir | Out-Null
}

function Download-Artifact {
    param(
        [string]$Url,
        [string]$FileName
    )

    # Rubyスクリプトの'kifu'プレフィックスフィルタを再現
    if (-not ($FileName.StartsWith('kifu'))) {
        return
    }

    $fullPath = Join-Path $ArchiveDir $FileName
    $targetDir = Split-Path $fullPath
    if (-not (Test-Path $targetDir)) {
        New-Item -ItemType Directory -Path $targetDir | Out-Null
    }

    Write-Host "Downloading $($FileName) to $($fullPath)..."

    $headers = @{
        "Accept" = "application/vnd.github+json"
    }
    if (-not ([string]::IsNullOrEmpty($GitHubToken))) {
        $headers["Authorization"] = "token $($GitHubToken)"
    }

    try {
        Invoke-WebRequest -Uri $Url -Headers $headers -OutFile $fullPath -UseBasicParsing -ErrorAction Stop
        Write-Host "Successfully downloaded $($FileName)."
    }
    catch {
        Write-Error "Failed to download $($FileName) from $($Url): $($_.Exception.Message)"
    }
}

function Expand-ArtifactZip {
    param(
        [string]$FileName
    )

    # Rubyスクリプトの'kifu'プレフィックスフィルタを再現
    if (-not ($FileName.StartsWith('kifu'))) {
        return
    }

    $zipPath = Join-Path $ArchiveDir $FileName
    $destinationPath = $ExtractDir

    if (-not (Test-Path $zipPath)) {
        Write-Warning "ZIPファイルが見つかりません: $($zipPath). スキップします。"
        return
    }

    Write-Host "Unzipping $($FileName) to $($destinationPath)..."
    try {
        # Expand-Archive を使用して解凍。既存ファイルは上書き。
        Expand-Archive -Path $zipPath -DestinationPath $destinationPath -Force -ErrorAction Stop
        Write-Host "Successfully unzipped $($FileName)."
    }
    catch {
        Write-Error "Failed to unzip $($FileName): $($_.Exception.Message)"
    }
}

function Get-ArtifactList {
    Write-Host "Downloading artifact list..."
    $listUrl = "https://api.github.com/repos/$($Owner)/$($Repo)/actions/artifacts?per_page=200"

    $headers = @{
        "Accept" = "application/vnd.github+json"
    }
    # リスト取得時にはトークンは必須ではないが、レート制限緩和のために含める
    if (-not ([string]::IsNullOrEmpty($GitHubToken))) {
        $headers["Authorization"] = "token $($GitHubToken)"
    }

    try {
        $response = Invoke-RestMethod -Uri $listUrl -Headers $headers -UseBasicParsing -ErrorAction Stop
        # JSONレスポンスをそのままログファイルに保存（Rubyスクリプトの動作を模倣）
        $response | ConvertTo-Json -Depth 10 | Set-Content $LogFile -Encoding UTF8
        return $response.artifacts
    }
    catch {
        Write-Error "Failed to download artifact list: $($_.Exception.Message)"
        return $null
    }
}

# --- メイン処理 ---
$artifacts = Get-ArtifactList
$downloadCount = 0

if ($null -ne $artifacts) {
    foreach ($artifact in $artifacts) {
        if ($downloadCount -ge $NArchive) {
            Write-Host "Maximum number of archives ($($NArchive)) reached. Stopping."
            break
        }

        $artifactName = "$($artifact.name).zip"
        $downloadUrl = $artifact.archive_download_url

        if ([string]::IsNullOrEmpty($downloadUrl)) {
            Write-Warning "Skipping artifact $($artifact.name) as download URL is missing."
            continue
        }

        Download-Artifact -Url $downloadUrl -FileName $artifactName
        Expand-ArtifactZip -FileName $artifactName

        $downloadCount++
    }
}
Write-Host "All specified downloads and unzips attempted. Total downloaded: $($downloadCount)"
