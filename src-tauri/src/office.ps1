# Office belgelerini yüklü Office uygulamalarıyla PDF'e çevirir.
# Girdi: her satırı "kaynak<TAB>hedef" olan UTF-8 liste dosyası.
# Çıktı (stdout, satır satır): BASLA/OK/HATA<TAB>kaynak[<TAB>ileti]
param([string]$Liste)
$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [Text.Encoding]::UTF8

# Kullanıcının zaten açık olan Office penceresini kapatmamak için önceden çalışanları not et.
$onceden = @{
  pp = [bool](Get-Process POWERPNT -ErrorAction SilentlyContinue)
  wd = [bool](Get-Process WINWORD -ErrorAction SilentlyContinue)
  xl = [bool](Get-Process EXCEL -ErrorAction SilentlyContinue)
}
$pp = $null; $wd = $null; $xl = $null

function Yaz($tur, $kaynak, $ileti = '') { [Console]::Out.WriteLine("$tur`t$kaynak`t$ileti"); [Console]::Out.Flush() }

foreach ($satir in [IO.File]::ReadAllLines($Liste, [Text.Encoding]::UTF8)) {
  if (-not $satir) { continue }
  $kaynak, $hedef = $satir -split "`t", 2
  Yaz 'BASLA' $kaynak
  try {
    switch -Regex ([IO.Path]::GetExtension($kaynak).ToLowerInvariant()) {
      '^\.(pptx?|pptm|ppsx?|potx?|odp)$' {
        if (-not $pp) { $pp = New-Object -ComObject PowerPoint.Application }
        # ReadOnly, Untitled=false, WithWindow=false
        $sunum = $pp.Presentations.Open($kaynak, -1, 0, 0)
        try { $sunum.SaveAs($hedef, 32) } finally { $sunum.Close() }
      }
      '^\.(docx?|docm|dotx?|odt|rtf)$' {
        if (-not $wd) { $wd = New-Object -ComObject Word.Application; $wd.Visible = $false; $wd.DisplayAlerts = 0 }
        $belge = $wd.Documents.Open($kaynak, $false, $true, $false)
        try { $belge.ExportAsFixedFormat($hedef, 17) } finally { $belge.Close(0) }
      }
      '^\.(xlsx?|xlsm|xlsb|ods|csv)$' {
        if (-not $xl) { $xl = New-Object -ComObject Excel.Application; $xl.Visible = $false; $xl.DisplayAlerts = $false }
        $kitap = $xl.Workbooks.Open($kaynak, 0, $true)
        try { $kitap.ExportAsFixedFormat(0, $hedef) } finally { $kitap.Close($false) }
      }
      default { throw 'Desteklenmeyen dosya türü' }
    }
    Yaz 'OK' $kaynak $hedef
  } catch {
    Yaz 'HATA' $kaynak ($_.Exception.Message -replace '\s+', ' ')
  }
}

if ($pp -and -not $onceden.pp) { try { $pp.Quit() } catch {} }
# Word.Quit'e PowerShell'den argüman verilemiyor ([ref] istiyor); belgeler zaten kapalı.
if ($wd -and -not $onceden.wd) { try { $wd.Quit() } catch {} }
if ($xl -and -not $onceden.xl) { try { $xl.Quit() } catch {} }
foreach ($o in $pp, $wd, $xl) { if ($o) { [void][Runtime.InteropServices.Marshal]::ReleaseComObject($o) } }
