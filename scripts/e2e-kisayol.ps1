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
    [PreserveSig] int DragEnter([MarshalAs(UnmanagedType.Interface)] object o, uint k, POINTL p, ref uint e);
    [PreserveSig] int DragOver(uint k, POINTL p, ref uint e);
    [PreserveSig] int DragLeave();
    [PreserveSig] int Drop([MarshalAs(UnmanagedType.Interface)] object o, uint k, POINTL p, ref uint e);
  }
  [ComImport, Guid("000214E6-0000-0000-C000-000000000046"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
  interface IShellFolder {
    void ParseDisplayName(IntPtr h, IntPtr b, [MarshalAs(UnmanagedType.LPWStr)] string n, IntPtr e, out IntPtr p, IntPtr a);
    void EnumObjects(IntPtr h, int f, out IntPtr e);
    void BindToObject(IntPtr p, IntPtr b, ref Guid r, out IntPtr o);
    void BindToStorage(IntPtr p, IntPtr b, ref Guid r, out IntPtr o);
    [PreserveSig] int CompareIDs(IntPtr l, IntPtr a, IntPtr b);
    void CreateViewObject(IntPtr h, ref Guid r, out IntPtr o);
    void GetAttributesOf(uint c, IntPtr a, ref uint f);
    void GetUIObjectOf(IntPtr h, uint c, [In] IntPtr[] a, ref Guid r, IntPtr z, [MarshalAs(UnmanagedType.Interface)] out object o);
  }
  [DllImport("shell32.dll", CharSet = CharSet.Unicode, PreserveSig = false)]
  static extern void SHParseDisplayName(string n, IntPtr b, out IntPtr p, uint i, out uint o);
  [DllImport("shell32.dll", PreserveSig = false)]
  static extern void SHBindToParent(IntPtr p, ref Guid r, [MarshalAs(UnmanagedType.Interface)] out object o, out IntPtr son);

  static object UiNesnesi(string yol, Guid iid) {
    IntPtr pidl; uint a; SHParseDisplayName(yol, IntPtr.Zero, out pidl, 0, out a);
    Guid isf = typeof(IShellFolder).GUID; object f; IntPtr son;
    SHBindToParent(pidl, ref isf, out f, out son);
    object o; ((IShellFolder)f).GetUIObjectOf(IntPtr.Zero, 1, new[] { son }, ref iid, IntPtr.Zero, out o);
    return o;
  }

  public static string Yap(string dosya, string hedef) {
    string sonuc = null;
    var t = new Thread(() => {
      try {
        object veri = UiNesnesi(dosya, new Guid("0000010e-0000-0000-C000-000000000046"));
        var dt = (IDropTarget)UiNesnesi(hedef, typeof(IDropTarget).GUID);
        var p = new POINTL(); uint e = 7; const uint SOL = 1;
        int h1 = dt.DragEnter(veri, SOL, p, ref e); uint e2 = e;
        int h2 = dt.DragOver(SOL, p, ref e2);
        int h3 = dt.Drop(veri, SOL, p, ref e2);
        Thread.Sleep(3000); // bırakma işleyicisi programı arka planda başlatabilir
        sonuc = string.Format("DragEnter=0x{0:X} DragOver=0x{1:X} Drop=0x{2:X} effect={3}", h1, h2, h3, e2);
      } catch (Exception x) { sonuc = "HATA " + x; }
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
Write-Host "  $([Birak]::Yap($kaynak, $lnk))"
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
