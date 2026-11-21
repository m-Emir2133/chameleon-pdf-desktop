//! Windows kabuğuna kısa yollar: masaüstüne "bırak-çevir" simgeleri, Gönder menüsü ve sağ tık menüsü.
//!
//! Windows'ta bir dosyayı bir .exe'nin ya da kısayolun (.lnk) üzerine sürükleyip bırakmak, programı o
//! dosyaların yollarıyla başlatır. Kısayolların argümanları (ör. `--eylem pdf --masaustu`) bu yolların
//! önüne eklenir; uygulama açılışta bunları okuyup işi kendisi yapar. Hepsi HKCU ve kullanıcı
//! klasörlerine yazılır; yönetici izni gerekmez ve her şey uygulamanın içinden geri alınabilir.

use crate::dil;
use serde::Serialize;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use windows::core::{GUID, HSTRING, Interface};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, IPersistFile, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Shell::{
    IShellLinkW, SHChangeNotify, SHGetKnownFolderPath, ShellLink, FOLDERID_Desktop, FOLDERID_SendTo, KNOWN_FOLDER_FLAG,
    SHCNE_ASSOCCHANGED, SHCNF_IDLIST,
};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Kısayolla yapılabilen işler: (kimlik, Türkçe ad, İngilizce ad).
pub const EYLEMLER: [(&str, &str, &str); 3] = [
    ("pdf", "PDF'e çevir", "Convert to PDF"),
    ("birlestir", "PDF birleştir", "Merge PDFs"),
    ("sikistir", "PDF sıkıştır", "Compress PDF"),
];

/// Sağ tık menüsünde "PDF'e çevir" çıkacak dosya türleri.
const CEVRILEBILIR: [&str; 26] = [
    "pptx", "ppt", "pptm", "ppsx", "pps", "potx", "odp", "docx", "doc", "docm", "odt", "rtf", "xlsx", "xls", "xlsm", "xlsb",
    "ods", "csv", "jpg", "jpeg", "png", "webp", "bmp", "gif", "tif", "tiff",
];
const ANAHTAR: &str = r"Software\Classes\SystemFileAssociations";

fn simge_verisi(eylem: &str) -> &'static [u8] {
    match eylem {
        "birlestir" => include_bytes!("../icons/kisayol-birlestir.ico"),
        "sikistir" => include_bytes!("../icons/kisayol-sikistir.ico"),
        _ => include_bytes!("../icons/kisayol-pdf.ico"),
    }
}

fn ad(eylem: &str, en: bool) -> &'static str {
    EYLEMLER.iter().find(|e| e.0 == eylem).map(|e| if en { e.2 } else { e.1 }).unwrap_or("Chameleon PDF")
}

fn bilinen_klasor(kimlik: &GUID) -> Option<PathBuf> {
    unsafe {
        let p = SHGetKnownFolderPath(kimlik, KNOWN_FOLDER_FLAG(0), None).ok()?;
        let yol = p.to_string().ok().map(PathBuf::from);
        CoTaskMemFree(Some(p.0 as *const _));
        yol
    }
}

/// Kullanıcının masaüstü (OneDrive'a taşınmışsa oradaki).
pub fn masaustu() -> Option<PathBuf> {
    bilinen_klasor(&FOLDERID_Desktop)
}

fn exe() -> Result<PathBuf, String> {
    std::env::current_exe().map_err(|e| e.to_string())
}

/// Simgeler uygulama verisinde durur; kısayol exe taşınsa bile simgesini kaybetmez.
fn simge_yolu(eylem: &str) -> Result<PathBuf, String> {
    let kok = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    let klasor = kok.join("com.bukalemun.masaustu").join("simgeler");
    std::fs::create_dir_all(&klasor).map_err(|e| e.to_string())?;
    let yol = klasor.join(format!("kisayol-{eylem}.ico"));
    std::fs::write(&yol, simge_verisi(eylem)).map_err(|e| e.to_string())?;
    Ok(yol)
}

fn lnk_yaz(lnk: &Path, argumanlar: &str, simge: &Path, aciklama: &str) -> Result<(), String> {
    let hedef = exe()?;
    let e = |e: windows::core::Error| e.message().to_string();
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let k: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER).map_err(e)?;
        k.SetPath(&HSTRING::from(hedef.as_os_str())).map_err(e)?;
        k.SetArguments(&HSTRING::from(argumanlar)).map_err(e)?;
        if let Some(k2) = hedef.parent() {
            k.SetWorkingDirectory(&HSTRING::from(k2.as_os_str())).map_err(e)?;
        }
        k.SetIconLocation(&HSTRING::from(simge.as_os_str()), 0).map_err(e)?;
        k.SetDescription(&HSTRING::from(aciklama)).map_err(e)?;
        let dosya: IPersistFile = k.cast().map_err(e)?;
        dosya.Save(&HSTRING::from(lnk.as_os_str()), true).map_err(e)?;
    }
    Ok(())
}

fn masaustu_lnk(eylem: &str, en: bool) -> Option<PathBuf> {
    masaustu().map(|m| m.join(format!("{}.lnk", ad(eylem, en))))
}

fn gonder_lnk(eylem: &str, en: bool) -> Option<PathBuf> {
    bilinen_klasor(&FOLDERID_SendTo).map(|k| k.join(format!("{} (Chameleon).lnk", ad(eylem, en))))
}

/// Kısayol iki dilden birinin adıyla duruyor olabilir.
fn var_mi(yol: impl Fn(bool) -> Option<PathBuf>) -> bool {
    [false, true].into_iter().any(|en| yol(en).is_some_and(|p| p.exists()))
}

fn ikisini_sil(yol: impl Fn(bool) -> Option<PathBuf>) {
    for en in [false, true] {
        if let Some(p) = yol(en) {
            let _ = std::fs::remove_file(p);
        }
    }
}

#[derive(Serialize)]
pub struct Durum {
    /// Masaüstünde simgesi olan eylemler.
    masaustu: Vec<String>,
    gonder: bool,
    sagtik: bool,
}

pub fn durum() -> Durum {
    Durum {
        masaustu: EYLEMLER.iter().filter(|e| var_mi(|en| masaustu_lnk(e.0, en))).map(|e| e.0.to_string()).collect(),
        gonder: EYLEMLER.iter().any(|e| var_mi(|en| gonder_lnk(e.0, en))),
        sagtik: Command::new("reg")
            .args(["query", &format!(r"HKCU\{ANAHTAR}\.pptx\shell\ChameleonPDF")])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .is_ok_and(|s| s.success()),
    }
}

/// `tur`: "masaustu" (tek eylem), "gonder" ya da "sagtik"; `acik` false ise kaldırır.
pub fn ayarla(tur: &str, eylem: &str, acik: bool) -> Result<(), String> {
    let en = dil::en();
    match tur {
        "masaustu" => {
            ikisini_sil(|en| masaustu_lnk(eylem, en));
            if acik {
                let lnk = masaustu_lnk(eylem, en).ok_or(dil::t("Masaüstü bulunamadı", "Desktop folder not found"))?;
                let aciklama = dil::t(
                    "Dosyaları bu simgenin üzerine bırak; sonuç masaüstüne kaydedilir.",
                    "Drop files on this icon; the result is saved to the desktop.",
                );
                lnk_yaz(&lnk, &format!("--eylem {eylem} --masaustu"), &simge_yolu(eylem)?, aciklama)?;
            }
        }
        "gonder" => {
            for (e, ..) in EYLEMLER {
                ikisini_sil(|en| gonder_lnk(e, en));
                if acik {
                    let lnk = gonder_lnk(e, en).ok_or(dil::t("Gönder klasörü bulunamadı", "Send To folder not found"))?;
                    lnk_yaz(&lnk, &format!("--eylem {e}"), &simge_yolu(e)?, ad(e, en))?;
                }
            }
        }
        "sagtik" => sag_tik(acik)?,
        _ => return Err("?".into()),
    }
    Ok(())
}

/// .reg dosyası içinde bir dizeyi tırnaklar.
fn reg_dize(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Sağ tık menüsü girdilerini oluşturan (ya da silen) .reg metni.
pub fn reg_metni(exe: &Path, simgeler: &[(&str, PathBuf)], acik: bool, en: bool) -> String {
    let mut s = String::from("Windows Registry Editor Version 5.00\r\n\r\n");
    let simge = |e: &str| simgeler.iter().find(|s| s.0 == e).map(|s| s.1.to_string_lossy().into_owned()).unwrap_or_default();
    let exe = exe.to_string_lossy();
    let mut girdi = |uzanti: &str, fiil: &str, eylem: &str| {
        let kok = format!(r"HKEY_CURRENT_USER\{ANAHTAR}\.{uzanti}\shell\{fiil}");
        if !acik {
            s.push_str(&format!("[-{kok}]\r\n\r\n"));
            return;
        }
        let komut = format!("\"{exe}\" --eylem {eylem} \"%1\"");
        s.push_str(&format!(
            "[{kok}]\r\n@={}\r\n\"Icon\"={}\r\n\"MultiSelectModel\"=\"Player\"\r\n\r\n[{kok}\\command]\r\n@={}\r\n\r\n",
            reg_dize(&format!("{} (Chameleon)", ad(eylem, en))),
            reg_dize(&simge(eylem)),
            reg_dize(&komut)
        ));
    };
    for u in CEVRILEBILIR {
        girdi(u, "ChameleonPDF", "pdf");
    }
    girdi("pdf", "ChameleonPDFSikistir", "sikistir");
    girdi("pdf", "ChameleonPDFBirlestir", "birlestir");
    s
}

fn sag_tik(acik: bool) -> Result<(), String> {
    let simgeler = if acik {
        EYLEMLER.iter().map(|e| Ok((e.0, simge_yolu(e.0)?))).collect::<Result<Vec<_>, String>>()?
    } else {
        vec![]
    };
    let metin = reg_metni(&exe()?, &simgeler, acik, dil::en());
    // reg import UTF-16 LE (BOM'lu) dosyayı Türkçe karakterleriyle doğru okur.
    let mut bayt = vec![0xFF, 0xFE];
    bayt.extend(metin.encode_utf16().flat_map(u16::to_le_bytes));
    let yol = std::env::temp_dir().join(format!("chameleon-sagtik-{}.reg", std::process::id()));
    std::fs::write(&yol, bayt).map_err(|e| e.to_string())?;
    let c = Command::new("reg")
        .arg("import")
        .arg(&yol)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW)
        .output();
    let _ = std::fs::remove_file(&yol);
    let c = c.map_err(|e| e.to_string())?;
    if !c.status.success() {
        return Err(String::from_utf8_lossy(&c.stderr).trim().to_string());
    }
    unsafe { SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None) };
    Ok(())
}

/// Komut satırı: `[--eylem pdf|birlestir|sikistir] [--masaustu] dosya...`. Klasörler bir düzey açılır.
#[derive(Clone, Serialize, Debug, PartialEq)]
pub struct Istek {
    pub dosyalar: Vec<String>,
    pub eylem: String,
    pub masaustu: bool,
    /// Uygulama yalnızca bu iş için mi açıldı (bitince kendiliğinden kapanabilir).
    pub yeni: bool,
}

pub fn arguman_coz(argumanlar: &[String], yeni: bool) -> Option<Istek> {
    let mut i = Istek { dosyalar: vec![], eylem: "pdf".into(), masaustu: false, yeni };
    let mut it = argumanlar.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--eylem" => {
                if let Some(e) = it.next() {
                    i.eylem = e.clone();
                }
            }
            "--masaustu" => i.masaustu = true,
            _ if a.starts_with("--") => {}
            _ => {
                let p = Path::new(a);
                if p.is_dir() {
                    let mut icindekiler: Vec<String> = std::fs::read_dir(p)
                        .into_iter()
                        .flatten()
                        .filter_map(|e| e.ok().map(|e| e.path()))
                        .filter(|p| p.is_file())
                        .map(|p| p.to_string_lossy().into_owned())
                        .collect();
                    icindekiler.sort();
                    i.dosyalar.extend(icindekiler);
                } else if p.is_file() {
                    i.dosyalar.push(a.clone());
                }
            }
        }
    }
    (!i.dosyalar.is_empty()).then_some(i)
}

#[cfg(test)]
mod testler {
    use super::*;

    #[test]
    fn argumanlar() {
        let k = std::env::temp_dir().join(format!("chameleon-arg-{}", std::process::id()));
        std::fs::create_dir_all(k.join("alt")).unwrap();
        let a = k.join("b sunum.pptx");
        let b = k.join("alt").join("a.docx");
        std::fs::write(&a, b"x").unwrap();
        std::fs::write(&b, b"x").unwrap();
        let s = |p: &Path| p.to_string_lossy().into_owned();
        let i = arguman_coz(&["--eylem".into(), "birlestir".into(), "--masaustu".into(), s(&a), s(&k.join("alt")), s(&k.join("yok.pdf"))], true).unwrap();
        assert_eq!(i, Istek { dosyalar: vec![s(&a), s(&b)], eylem: "birlestir".into(), masaustu: true, yeni: true });
        let i = arguman_coz(&[s(&a)], false).unwrap();
        assert_eq!((i.eylem.as_str(), i.masaustu, i.yeni), ("pdf", false, false));
        assert!(arguman_coz(&["--masaustu".into()], true).is_none());
        let _ = std::fs::remove_dir_all(&k);
    }

    #[test]
    fn reg() {
        let simgeler: Vec<(&str, PathBuf)> = EYLEMLER.iter().map(|e| (e.0, PathBuf::from(format!(r"C:\S\{}.ico", e.0)))).collect();
        let m = reg_metni(Path::new(r"C:\Program Files\Chameleon\chameleon-pdf.exe"), &simgeler, true, false);
        assert!(m.starts_with("Windows Registry Editor Version 5.00\r\n"));
        assert!(m.contains("[HKEY_CURRENT_USER\\Software\\Classes\\SystemFileAssociations\\.pptx\\shell\\ChameleonPDF]\r\n@=\"PDF'e çevir (Chameleon)\"\r\n\"Icon\"=\"C:\\\\S\\\\pdf.ico\""));
        assert!(m.contains("@=\"\\\"C:\\\\Program Files\\\\Chameleon\\\\chameleon-pdf.exe\\\" --eylem pdf \\\"%1\\\"\""));
        assert!(m.contains(".pdf\\shell\\ChameleonPDFBirlestir]"));
        let sil = reg_metni(Path::new("x.exe"), &[], false, true);
        assert!(sil.contains("[-HKEY_CURRENT_USER\\Software\\Classes\\SystemFileAssociations\\.docx\\shell\\ChameleonPDF]"));
        assert!(!sil.contains("command"));
    }
}
