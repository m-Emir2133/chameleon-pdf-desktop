//! Office belgelerini LibreOffice ile PDF'e çevirir. Microsoft Office kurulu değilse (ya da kullanıcı
//! isterse) kullanılır. LibreOffice ayrı bir profille, görünmez olarak çalışır; kullanıcının açık
//! LibreOffice pencerelerine dokunulmaz.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::os::windows::process::CommandExt;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Takılan bir belge uygulamayı sonsuza dek bekletmesin.
const ZAMAN_ASIMI: Duration = Duration::from_secs(300);

/// Aynı profili kullanan iki LibreOffice aynı anda çalışamaz; dönüşümler sırayla yapılır.
static KILIT: Mutex<()> = Mutex::new(());

fn gizli(c: &mut Command) -> &mut Command {
    #[cfg(windows)]
    c.creation_flags(CREATE_NO_WINDOW);
    c
}

/// `soffice` programının yolu. Sırayla: CHAMELEON_SOFFICE ortam değişkeni, kayıt defteri,
/// bilinen kurulum klasörleri, PATH.
pub fn bul() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("CHAMELEON_SOFFICE").map(PathBuf::from) {
        if p.is_file() {
            return Some(p);
        }
    }
    let ad = if cfg!(windows) { "soffice.exe" } else { "soffice" };
    let mut adaylar: Vec<PathBuf> = Vec::new();
    #[cfg(windows)]
    {
        for anahtar in [
            r"HKLM\SOFTWARE\LibreOffice\UNO\InstallPath",
            r"HKLM\SOFTWARE\WOW6432Node\LibreOffice\UNO\InstallPath",
            r"HKCU\SOFTWARE\LibreOffice\UNO\InstallPath",
        ] {
            if let Some(k) = reg_varsayilan(anahtar) {
                adaylar.push(PathBuf::from(k).join(ad));
            }
        }
        for degisken in ["ProgramW6432", "ProgramFiles", "ProgramFiles(x86)"] {
            if let Some(k) = std::env::var_os(degisken) {
                adaylar.push(PathBuf::from(k).join(r"LibreOffice\program").join(ad));
            }
        }
    }
    #[cfg(not(windows))]
    {
        adaylar.push(PathBuf::from("/usr/bin").join(ad));
        adaylar.push(PathBuf::from("/Applications/LibreOffice.app/Contents/MacOS").join(ad));
    }
    if let Some(yol) = std::env::var_os("PATH") {
        adaylar.extend(std::env::split_paths(&yol).map(|k| k.join(ad)));
    }
    adaylar.into_iter().find(|p| p.is_file())
}

/// `reg query <anahtar> /ve` çıktısından varsayılan değeri okur.
#[cfg(windows)]
fn reg_varsayilan(anahtar: &str) -> Option<String> {
    let cikti = gizli(Command::new("reg").args(["query", anahtar, "/ve"]))
        .stderr(Stdio::null())
        .output()
        .ok()
        .filter(|c| c.status.success())?;
    reg_degeri(&String::from_utf8_lossy(&cikti.stdout))
}

/// "    (Default)    REG_SZ    C:\Program Files\LibreOffice\program" satırından yolu ayıklar.
/// "(Default)" Windows diline göre değiştiği için türe ("REG_SZ") göre bölünür.
pub fn reg_degeri(cikti: &str) -> Option<String> {
    cikti.lines().find_map(|l| {
        let (_, deger) = l.split_once("REG_EXPAND_SZ").or_else(|| l.split_once("REG_SZ"))?;
        let deger = deger.trim();
        (!deger.is_empty()).then(|| deger.to_string())
    })
}

/// Yerel yolu LibreOffice'in beklediği `file:///` adresine çevirir.
pub fn dosya_url(yol: &Path) -> String {
    let duz = yol.to_string_lossy().replace('\\', "/");
    let mut s = String::from("file://");
    if !duz.starts_with('/') {
        s.push('/');
    }
    for b in duz.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' | b':' => s.push(b as char),
            _ => s.push_str(&format!("%{b:02X}")),
        }
    }
    s
}

/// Uygulamaya ait LibreOffice profili; ilk çalıştırmada oluşur, sonrakiler daha hızlı açılır.
fn profil() -> PathBuf {
    let kok = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    kok.join("com.bukalemun.masaustu").join("libreoffice-profil")
}

/// `kaynak` belgesini `hedef` PDF'ine çevirir. Hata iletisi ham hâliyle döner.
pub fn cevir(soffice: &Path, kaynak: &Path, hedef: &Path) -> Result<(), String> {
    let _sira = KILIT.lock().unwrap_or_else(|e| e.into_inner());
    let gecici = std::env::temp_dir().join(format!(
        "chameleon-lo-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()
    ));
    std::fs::create_dir_all(&gecici).map_err(|e| e.to_string())?;
    let sonuc = calistir(soffice, kaynak, hedef, &gecici);
    let _ = std::fs::remove_dir_all(&gecici);
    sonuc
}

fn calistir(soffice: &Path, kaynak: &Path, hedef: &Path, gecici: &Path) -> Result<(), String> {
    let mut cocuk = gizli(
        Command::new(soffice)
            .arg(format!("-env:UserInstallation={}", dosya_url(&profil())))
            .args(["--headless", "--invisible", "--nologo", "--norestore", "--nodefault", "--nolockcheck"])
            .args(["--convert-to", "pdf", "--outdir"])
            .arg(gecici)
            .arg(kaynak),
    )
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()
    .map_err(|e| format!("LibreOffice başlatılamadı / could not start LibreOffice: {e}"))?;

    let baslangic = Instant::now();
    loop {
        if cocuk.try_wait().map_err(|e| e.to_string())?.is_some() {
            break;
        }
        if baslangic.elapsed() > ZAMAN_ASIMI {
            oldur(&mut cocuk);
            return Err("timeout".into());
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    // Çıktının adı kaynaktan türetilir; tahmin etmek yerine klasördeki PDF'i al.
    let uretilen = std::fs::read_dir(gecici)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .find(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("pdf")))
        .ok_or_else(|| "no output".to_string())?;
    if std::fs::rename(&uretilen, hedef).is_err() {
        // Geçici klasör başka bir sürücüdeyse taşıma olmaz; kopyala.
        std::fs::copy(&uretilen, hedef).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn oldur(cocuk: &mut std::process::Child) {
    // Windows'ta soffice.exe yalnızca bir başlatıcı; asıl iş soffice.bin'de, ağacı birlikte kapat.
    #[cfg(windows)]
    {
        let _ = gizli(Command::new("taskkill").args(["/T", "/F", "/PID", &cocuk.id().to_string()]))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = cocuk.kill();
    let _ = cocuk.wait();
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn url() {
        assert_eq!(dosya_url(Path::new(r"C:\Users\Ali Veli\AppData")), "file:///C:/Users/Ali%20Veli/AppData");
        assert_eq!(dosya_url(Path::new("/tmp/ş")), "file:///tmp/%C5%9F");
    }

    #[test]
    fn reg() {
        let c = "\r\nHKEY_LOCAL_MACHINE\\SOFTWARE\\LibreOffice\\UNO\\InstallPath\r\n    (Varsayılan)    REG_SZ    C:\\Program Files\\LibreOffice\\program\r\n\r\n";
        assert_eq!(reg_degeri(c).as_deref(), Some(r"C:\Program Files\LibreOffice\program"));
        assert_eq!(reg_degeri("HKEY_X\r\n    (Default)    REG_SZ    \r\n"), None);
    }

    /// Bilgisayarda LibreOffice varsa gerçek bir dönüşüm dener.
    #[test]
    fn donusum() {
        let Some(soffice) = bul() else {
            eprintln!("atlandı: LibreOffice yok");
            return;
        };
        let k = std::env::temp_dir().join(format!("chameleon-lo-test-{}", std::process::id()));
        std::fs::create_dir_all(&k).unwrap();
        let kaynak = k.join("deneme belge ç.csv");
        std::fs::write(&kaynak, "ad,sayi\nelma,3\narmut,5\n").unwrap();
        let hedef = k.join("sonuç.pdf");
        cevir(&soffice, &kaynak, &hedef).unwrap();
        let bayt = std::fs::read(&hedef).unwrap();
        assert!(bayt.starts_with(b"%PDF"));
        let bozuk = k.join("bozuk.docx");
        std::fs::write(&bozuk, b"PK\x03\x04 bu bir docx degil").unwrap();
        assert!(cevir(&soffice, &bozuk, &k.join("bozuk.pdf")).is_err());
        let _ = std::fs::remove_dir_all(&k);
    }
}
