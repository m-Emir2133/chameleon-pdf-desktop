# Kısa yolların Windows'ta uçtan uca denemesi (GitHub Actions windows-latest ya da kendi bilgisayarın).
#   pwsh scripts/e2e-kisayol.ps1 -Exe src-tauri\target\release\chameleon-pdf.exe -Pptx <örnek.pptx>
# 1) Masaüstü simgesi uygulamanın kendi koduyla kurulur (--kisayol masaustu pdf).
# 2) PPTX, Explorer'ın sürükle-bırakta kullandığı yolla (kısayolun IDropTarget'ı) simgeye bırakılır.
# 3) PDF masaüstünde belirmeli ve yalnızca bu iş için açılan uygulama kendiliğinden kapanmalı.
# 4) Uygulama açıkken sağ tık menüsünden iki PDF "birleştir"e gönderilir: tek pencere, tek sonuç.
param(
  [Parameter(Mandatory)] [string]$Exe,
  [Parameter(Mandatory)] [string]$Pptx,
  [string]$Cikti = 'e2e-cikti'
)
$ErrorActionPreference = 'Stop'
$Exe = (Resolve-Path $Exe).Path
$Pptx = (Resolve-Path $Pptx).Path
New-Item -ItemType Directory -Force $Cikti | Out-Null
$Cikti = (Resolve-Path $Cikti).Path
$masaustu = [Environment]::GetFolderPath('Desktop')
$hatalar = [System.Collections.Generic.List[string]]::new()
function Adim($m) { Write-Host "`n=== $m" }
function Kontrol($kosul, $m) { if ($kosul) { Write-Host "  OK   $m" } else { Write-Host "  HATA $m"; $hatalar.Add($m) } }
function Bekle([scriptblock]$kosul, [int]$saniye) {
  $son = (Get-Date).AddSeconds($saniye)
  while ((Get-Date) -lt $son) { if (& $kosul) { return $true }; Start-Sleep -Milliseconds 500 }
  return [bool](& $kosul)
}
function Uygulama { @(Get-Process chameleon-pdf -ErrorAction SilentlyContinue) }
function Calistir([string[]]$a) { (Start-Process $Exe -ArgumentList $a -Wait -PassThru -WindowStyle Hidden).ExitCode }
function EkranGoruntusu($ad) {
  try {
    Add-Type -AssemblyName System.Windows.Forms, System.Drawing
    $b = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
    $bmp = New-Object System.Drawing.Bitmap $b.Width, $b.Height
    $g = [System.Drawing.Graphics]::FromImage($bmp); $g.CopyFromScreen($b.Location, [System.Drawing.Point]::Empty, $b.Size)
    $bmp.Save((Join-Path $Cikti $ad)); $g.Dispose(); $bmp.Dispose()
  } catch { Write-Host "  (ekran görüntüsü alınamadı: $_)" }
}

# Explorer'ın sürükle-bırak yolu: kaynak dosyanın IDataObject'i, hedef kısayolun IDropTarget'ı.
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Threading;
public static class Birak {
  [StructLayout(LayoutKind.Sequential)] public struct POINTL { public int x; public int y; }
  [ComImport, Guid("00000122-0000-0000-C000-000000000046"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
  interface IDropTarget {
    [PreserveSig] int DragEnter(IntPtr veri, uint k, POINTL p, ref uint e);
    [PreserveSig] int DragOver(uint k, POINTL p, ref uint e);
    [PreserveSig] int DragLeave();
    [PreserveSig] int Drop(IntPtr veri, uint k, POINTL p, ref uint e);
  }
  [ComImport, Guid("43826d1e-e718-42ee-bc55-a1e261c37bfe"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
  interface IShellItem {
    [PreserveSig] int BindToHandler(IntPtr pbc, ref Guid bhid, ref Guid riid, out IntPtr ppv);
  }
  [DllImport("shell32.dll", CharSet = CharSet.Unicode)]
  static extern int SHCreateItemFromParsingName(string yol, IntPtr pbc, ref Guid riid, out IntPtr ppv);

  static IntPtr Isleyici(string yol, string bhid, string iid) {
    Guid isi = typeof(IShellItem).GUID; IntPtr p;
    int h = SHCreateItemFromParsingName(yol, IntPtr.Zero, ref isi, out p);
    if (h != 0) throw new Exception(string.Format("SHCreateItemFromParsingName 0x{0:X} {1}", h, yol));
    var oge = (IShellItem)Marshal.GetObjectForIUnknown(p); Marshal.Release(p);
    Guid b = new Guid(bhid), i = new Guid(iid); IntPtr sonuc;
    h = oge.BindToHandler(IntPtr.Zero, ref b, ref i, out sonuc);
    if (h != 0) throw new Exception(string.Format("BindToHandler 0x{0:X} {1}", h, yol));
    return sonuc;
  }

  public static string Yap(string dosya, string hedef) {
    string sonuc = null;
    var t = new Thread(() => {
      try {
        // BHID_DataObject: Explorer'ın sürüklerken taşıdığı veri; BHID_SFUIObject + IID_IDropTarget: bırakılan simge.
        IntPtr veri = Isleyici(dosya, "b8c0bd9f-ed24-455c-83e6-d5390c4fe8c4", "0000010e-0000-0000-C000-000000000046");
        IntPtr dtp = Isleyici(hedef, "3981e225-f559-11d3-8e3a-00c04f6837d5", "00000122-0000-0000-C000-000000000046");
        var dt = (IDropTarget)Marshal.GetObjectForIUnknown(dtp);
        var p = new POINTL(); uint e = 7; const uint SOL = 1;
        int h1 = dt.DragEnter(veri, SOL, p, ref e); uint e2 = e;
        int h2 = dt.DragOver(SOL, p, ref e2);
        int h3 = dt.Drop(veri, SOL, p, ref e2);
        Thread.Sleep(3000); // bırakma işleyicisi programı arka planda başlatabilir
        sonuc = string.Format("DragEnter=0x{0:X} DragOver=0x{1:X} Drop=0x{2:X} effect={3}", h1, h2, h3, e2);
      } catch (Exception x) { sonuc = "HATA " + x.Message; }
    });
    t.SetApartmentState(ApartmentState.STA); t.Start(); t.Join();
    return sonuc;
  }
}
'@

Get-Process chameleon-pdf -ErrorAction SilentlyContinue | Stop-Process -Force

Adim 'Masaüstü simgesi kuruluyor (--kisayol masaustu pdf)'
Kontrol ((Calistir '--kisayol', 'masaustu', 'pdf') -eq 0) 'kurulum komutu 0 ile çıktı'
$lnk = Join-Path $masaustu "PDF'e çevir.lnk"
Kontrol (Test-Path -LiteralPath $lnk) "masaüstünde '$lnk' var"
$k = (New-Object -ComObject WScript.Shell).CreateShortcut($lnk)
Write-Host "  hedef: $($k.TargetPath) $($k.Arguments) | simge: $($k.IconLocation)"
Kontrol ($k.TargetPath -eq $Exe -and $k.Arguments -eq '--eylem pdf --masaustu') 'kısayol exe ve argümanları doğru'
Kontrol (Test-Path -LiteralPath ($k.IconLocation -replace ',\d+$', '')) 'simge dosyası var'

Adim 'PPTX simgenin üzerine bırakılıyor (uygulama kapalıyken)'
$kaynak = Join-Path $env:TEMP 'chameleon e2e\Sunum deneme ğüş.pptx'
New-Item -ItemType Directory -Force (Split-Path $kaynak) | Out-Null
Copy-Item -LiteralPath $Pptx $kaynak -Force
$pdf = Join-Path $masaustu 'Sunum deneme ğüş.pdf'
Remove-Item -LiteralPath $pdf -ErrorAction SilentlyContinue
$t0 = Get-Date
$birak = [Birak]::Yap($kaynak, $lnk)
Write-Host "  $birak"
Kontrol ($birak -notlike 'HATA*') 'Explorer sürükle-bırak çağrısı yapıldı'
Kontrol (Bekle { (Uygulama).Count -gt 0 -or (Test-Path -LiteralPath $pdf) } 30) 'bırakınca uygulama başladı'
Start-Sleep 3; EkranGoruntusu '1-birakinca.png'
Kontrol (Bekle { Test-Path -LiteralPath $pdf } 240) "PDF masaüstünde: $pdf"
if (Test-Path -LiteralPath $pdf) {
  Write-Host ("  {0:N1} sn, {1} bayt" -f ((Get-Date) - $t0).TotalSeconds, (Get-Item -LiteralPath $pdf).Length)
  $bas = [Text.Encoding]::ASCII.GetString([IO.File]::ReadAllBytes($pdf)[0..4])
  Kontrol ($bas -eq '%PDF-') 'dosya gerçekten PDF'
  Copy-Item -LiteralPath $pdf (Join-Path $Cikti 'masaustu-sonuc.pdf')
}
EkranGoruntusu '2-bitince.png'
Kontrol (Bekle { (Uygulama).Count -eq 0 } 30) 'iş bitince uygulama kendiliğinden kapandı'

Adim 'Sağ tık menüsü kuruluyor ve uygulama açıkken iki PDF birleştiriliyor'
Kontrol ((Calistir '--kisayol', 'sagtik') -eq 0) 'sağ tık kurulumu 0 ile çıktı'
$anahtar = 'HKCU:\Software\Classes\SystemFileAssociations\.pptx\shell\ChameleonPDF'
Kontrol (Test-Path $anahtar) 'kayıt defterinde .pptx girdisi var'
Write-Host "  .pptx komutu: $((Get-ItemProperty "$anahtar\command").'(default)')"
Write-Host "  .pptx etiketi: $((Get-ItemProperty $anahtar).'(default)')"
$klasor = Join-Path $env:TEMP 'chameleon e2e\birlestir'
New-Item -ItemType Directory -Force $klasor | Out-Null
Get-ChildItem $klasor | Remove-Item -Force
if (Test-Path -LiteralPath $pdf) {
  Copy-Item -LiteralPath $pdf (Join-Path $klasor 'b10.pdf'); Copy-Item -LiteralPath $pdf (Join-Path $klasor 'b2.pdf')
}
$ana = Start-Process $Exe -PassThru
Start-Sleep 8
$kabuk = New-Object -ComObject Shell.Application
$ns = $kabuk.Namespace($klasor)
foreach ($ad in 'b10.pdf', 'b2.pdf') {
  $oge = $ns.ParseName($ad)
  $fiiller = @($oge.Verbs() | ForEach-Object { $_.Name })
  Write-Host "  $ad fiilleri: $($fiiller -join ' | ')"
  $oge.InvokeVerb('ChameleonPDFBirlestir')
}
Kontrol (Bekle { @(Get-ChildItem $klasor -Filter 'b2 - *.pdf').Count -gt 0 } 60) 'birleştirilmiş PDF kaynağın yanında (b2 önce)'
Get-ChildItem $klasor | ForEach-Object { Write-Host "  $($_.Name) $($_.Length)" }
Kontrol (@(Get-ChildItem $klasor -Filter '* - *.pdf').Count -eq 1) 'iki dosya tek işte birleşti'
Start-Sleep 2
Kontrol ((Uygulama).Count -eq 1 -and -not $ana.HasExited) 'tek pencere kaldı ve önceden açık uygulama kapanmadı'
EkranGoruntusu '3-sagtik.png'
$ana | Stop-Process -Force -ErrorAction SilentlyContinue

Adim 'Gönder menüsü ve kaldırma'
Kontrol ((Calistir '--kisayol', 'gonder') -eq 0) 'Gönder kurulumu 0 ile çıktı'
$gonder = [Environment]::GetFolderPath('SendTo')
Kontrol (@(Get-ChildItem -LiteralPath $gonder -Filter '*(Chameleon).lnk').Count -eq 3) 'Gönder klasöründe 3 kısayol'
foreach ($tur in 'sagtik', 'gonder') { Kontrol ((Calistir '--kisayol', $tur, '--kaldir') -eq 0) "$tur kaldırıldı" }
Kontrol ((Calistir '--kisayol', 'masaustu', 'pdf', '--kaldir') -eq 0) 'masaüstü simgesi kaldırıldı'
Kontrol (-not (Test-Path $anahtar)) 'kayıt defteri girdisi silindi'
Kontrol (-not (Test-Path -LiteralPath $lnk)) 'masaüstü kısayolu silindi'
Kontrol (@(Get-ChildItem -LiteralPath $gonder -Filter '*(Chameleon).lnk').Count -eq 0) 'Gönder kısayolları silindi'

Write-Host ''
if ($hatalar.Count) { Write-Host "BAŞARISIZ: $($hatalar.Count) kontrol"; $hatalar | ForEach-Object { Write-Host " - $_" }; exit 1 }
Write-Host 'TÜM KONTROLLER GEÇTİ'
