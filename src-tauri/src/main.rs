// Chameleon PDF masaüstü — Office belgelerini yüklü Office ile PDF'e çevirir, PDF araçlarını
// (birleştir, böl, sayfa çıkar, döndür, görsel ↔ PDF) yerelde çalıştırır.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod duzen;
mod gorsel;
mod model;
mod pdf;
mod tarama;
mod yazi;
#[cfg(test)]
mod testler;

use serde::Serialize;
use std::collections::HashSet;
use std::io::{BufRead, BufReader, Write};
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tauri::{AppHandle, Emitter};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Clone, Serialize)]
struct Ilerleme {
    /// İşin kimliği: Office işlerinde kaynak dosya yolu, diğerlerinde arayüzün verdiği kimlik.
    anahtar: String,
    durum: &'static str,
    ileti: String,
    ciktilar: Vec<String>,
}

fn bildir(app: &AppHandle, anahtar: &str, durum: &'static str, ileti: impl Into<String>, ciktilar: Vec<String>) {
    let _ = app.emit("ilerleme", Ilerleme { anahtar: anahtar.into(), durum, ileti: ileti.into(), ciktilar });
}

/// `klasor/govde.uzanti` var ya da bu işte ayrıldıysa "govde (2).uzanti" gibi boş bir ad bulur.
fn benzersiz(klasor: &Path, govde: &str, uzanti: &str, ayrilan: &mut HashSet<PathBuf>) -> PathBuf {
    let ad = |ek: &str| {
        if uzanti.is_empty() {
            klasor.join(format!("{govde}{ek}"))
        } else {
            klasor.join(format!("{govde}{ek}.{uzanti}"))
        }
    };
    let mut yol = ad("");
    let mut i = 2;
    while yol.exists() || ayrilan.contains(&yol) {
        yol = ad(&format!(" ({i})"));
        i += 1;
    }
    ayrilan.insert(yol.clone());
    yol
}

fn govde(yol: &Path) -> String {
    yol.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "belge".into())
}

fn klasoru(yol: &Path) -> PathBuf {
    yol.parent().map(Path::to_path_buf).unwrap_or_default()
}

fn yazi(yollar: &[PathBuf]) -> Vec<String> {
    yollar.iter().map(|p| p.to_string_lossy().into_owned()).collect()
}

async fn arka_planda<T: Send + 'static>(is: impl FnOnce() -> Result<T, String> + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(is).await.map_err(|e| e.to_string())?
}

#[tauri::command]
async fn office(app: AppHandle, dosyalar: Vec<String>) -> Result<(), String> {
    arka_planda(move || {
        let mut ayrilan = HashSet::new();
        let mut liste = String::new();
        for d in &dosyalar {
            let k = PathBuf::from(d);
            let hedef = benzersiz(&klasoru(&k), &govde(&k), "pdf", &mut ayrilan);
            liste.push_str(&format!("{}\t{}\n", d, hedef.display()));
        }
        let gecici = std::env::temp_dir().join(format!("chameleon-{}", std::process::id()));
        std::fs::create_dir_all(&gecici).map_err(|e| e.to_string())?;
        let betik = gecici.join("office.ps1");
        let liste_yolu = gecici.join(format!("liste-{}.txt", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()));
        // Windows PowerShell 5.1, BOM'suz betiği ANSI okur; Türkçe iletiler için BOM ekle.
        std::fs::write(&betik, format!("\u{FEFF}{}", include_str!("office.ps1"))).map_err(|e| e.to_string())?;
        std::fs::write(&liste_yolu, liste).map_err(|e| e.to_string())?;

        let mut cocuk = Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-ExecutionPolicy", "Bypass", "-File"])
            .arg(&betik)
            .arg(&liste_yolu)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("PowerShell başlatılamadı: {e}"))?;
        let mut biten = HashSet::new();
        if let Some(cikti) = cocuk.stdout.take() {
            for satir in BufReader::new(cikti).lines().map_while(Result::ok) {
                let mut p = satir.trim_start_matches('\u{FEFF}').splitn(3, '\t');
                let (tur, kaynak, ileti) = (p.next().unwrap_or(""), p.next().unwrap_or(""), p.next().unwrap_or(""));
                match tur {
                    "BASLA" => bildir(&app, kaynak, "calisiyor", "Office ile çevriliyor…", vec![]),
                    "OK" => {
                        biten.insert(kaynak.to_string());
                        bildir(&app, kaynak, "tamam", "PDF hazır", vec![ileti.to_string()]);
                    }
                    "HATA" => {
                        biten.insert(kaynak.to_string());
                        bildir(&app, kaynak, "hata", office_hatasi(ileti), vec![]);
                    }
                    _ => {}
                }
            }
        }
        let mut hata_metni = String::new();
        if let Some(mut e) = cocuk.stderr.take() {
            use std::io::Read;
            let _ = e.read_to_string(&mut hata_metni);
        }
        let _ = cocuk.wait();
        let _ = std::fs::remove_file(&liste_yolu);
        for d in dosyalar.iter().filter(|d| !biten.contains(*d)) {
            let ileti = hata_metni.lines().find(|l| !l.trim().is_empty()).unwrap_or("Office yanıt vermedi");
            bildir(&app, d, "hata", office_hatasi(ileti), vec![]);
        }
        Ok(())
    })
    .await
}

/// COM hatalarını kullanıcının anlayacağı bir cümleye çevirir.
fn office_hatasi(ham: &str) -> String {
    let k = ham.to_lowercase();
    if k.contains("80040154") || k.contains("class not registered") || k.contains("sınıf kayıtlı değil") {
        "Bu dosya türü için Office uygulaması kurulu değil".into()
    } else if k.contains("password") || k.contains("parola") {
        "Dosya parola korumalı".into()
    } else if ham.is_empty() {
        "Çevrilemedi".into()
    } else {
        format!("Çevrilemedi: {ham}")
    }
}

#[tauri::command]
async fn birlestir(dosyalar: Vec<String>) -> Result<Vec<String>, String> {
    arka_planda(move || {
        let yollar: Vec<PathBuf> = dosyalar.iter().map(PathBuf::from).collect();
        let ilk = yollar.first().ok_or("Dosya yok")?;
        let hedef = benzersiz(&klasoru(ilk), &format!("{} - birleştirilmiş", govde(ilk)), "pdf", &mut HashSet::new());
        pdf::birlestir(&yollar, &hedef)?;
        Ok(yazi(&[hedef]))
    })
    .await
}

#[tauri::command]
async fn bol(dosya: String, adim: u32) -> Result<Vec<String>, String> {
    arka_planda(move || {
        let k = PathBuf::from(&dosya);
        let klasor = benzersiz(&klasoru(&k), &format!("{} - bölünmüş", govde(&k)), "", &mut HashSet::new());
        std::fs::create_dir_all(&klasor).map_err(|e| e.to_string())?;
        let g = govde(&k);
        let parcalar = pdf::bol(&k, adim, |bas, son, toplam| {
            let w = toplam.to_string().len();
            if bas == son {
                klasor.join(format!("{g} - s{bas:0w$}.pdf"))
            } else {
                klasor.join(format!("{g} - s{bas:0w$}-{son:0w$}.pdf"))
            }
        })?;
        let mut sonuc = vec![klasor.to_string_lossy().into_owned()];
        sonuc.push(format!("{} dosya", parcalar.len()));
        Ok(sonuc)
    })
    .await
}

#[tauri::command]
async fn sayfa_cikar(dosya: String, aralik: String) -> Result<Vec<String>, String> {
    arka_planda(move || {
        let k = PathBuf::from(&dosya);
        let etiket: String = aralik.chars().filter(|c| c.is_ascii_digit() || *c == '-' || *c == ',').collect();
        let hedef = benzersiz(&klasoru(&k), &format!("{} - sayfa {}", govde(&k), etiket.replace(',', "_")), "pdf", &mut HashSet::new());
        pdf::sayfa_cikar(&k, &aralik, &hedef)?;
        Ok(yazi(&[hedef]))
    })
    .await
}

#[tauri::command]
async fn dondur(dosya: String, derece: i64, aralik: String) -> Result<Vec<String>, String> {
    arka_planda(move || {
        let k = PathBuf::from(&dosya);
        let hedef = benzersiz(&klasoru(&k), &format!("{} - döndürülmüş", govde(&k)), "pdf", &mut HashSet::new());
        pdf::dondur(&k, derece, &aralik, &hedef)?;
        Ok(yazi(&[hedef]))
    })
    .await
}

#[tauri::command]
async fn gorsellerden_pdf(dosyalar: Vec<String>) -> Result<Vec<String>, String> {
    arka_planda(move || {
        let yollar: Vec<PathBuf> = dosyalar.iter().map(PathBuf::from).collect();
        let ilk = yollar.first().ok_or("Görsel yok")?;
        let ad = if yollar.len() == 1 { govde(ilk) } else { format!("{} ve {} görsel", govde(ilk), yollar.len() - 1) };
        let hedef = benzersiz(&klasoru(ilk), &ad, "pdf", &mut HashSet::new());
        pdf::gorsellerden(&yollar, &hedef)?;
        Ok(yazi(&[hedef]))
    })
    .await
}

#[tauri::command]
async fn pdf_gorsel(app: AppHandle, anahtar: String, dosya: String, dpi: u32, jpg: bool) -> Result<Vec<String>, String> {
    arka_planda(move || {
        let k = PathBuf::from(&dosya);
        let klasor = benzersiz(&klasoru(&k), &format!("{} - görseller", govde(&k)), "", &mut HashSet::new());
        let sayfalar = gorsel::pdf_gorsel(&k, &klasor, dpi.clamp(36, 600), jpg, |i, n| {
            bildir(&app, &anahtar, "calisiyor", format!("Sayfa {i}/{n}"), vec![]);
        })?;
        Ok(vec![klasor.to_string_lossy().into_owned(), format!("{} görsel", sayfalar.len())])
    })
    .await
}

fn veri_url(jpeg: &[u8]) -> String {
    use base64::Engine;
    format!("data:image/jpeg;base64,{}", base64::engine::general_purpose::STANDARD.encode(jpeg))
}

#[derive(Serialize)]
struct TaramaOnizleme {
    /// Yönü düzeltilmiş fotoğrafın küçük hali (köşe düzenleyici için).
    foto: String,
    gen: u32,
    yuk: u32,
    kose: tarama::Kose,
    yol: &'static str,
}

/// Fotoğrafı açar, köşeleri bulur ve düzenleyici için küçük bir önizleme döner.
#[tauri::command]
async fn tarama_hazirla(dosya: String) -> Result<TaramaOnizleme, String> {
    arka_planda(move || {
        let img = tarama::ac(Path::new(&dosya))?;
        let (kose, yol) = tarama::koseleri_bul(&img);
        let kucuk = img.resize(1400, 1400, image::imageops::FilterType::Triangle);
        Ok(TaramaOnizleme { foto: veri_url(&tarama::jpeg(&kucuk, 82)), gen: kucuk.width(), yuk: kucuk.height(), kose, yol })
    })
    .await
}

#[derive(serde::Deserialize)]
struct TaramaSayfa {
    dosya: String,
    kose: tarama::Kose,
    donus: u32,
    filtre: tarama::Filtre,
}

/// Düzeltilmiş sayfanın küçük önizlemesi (kart görseli).
#[tauri::command]
async fn tarama_onizle(sayfa: TaramaSayfa) -> Result<String, String> {
    arka_planda(move || {
        let img = tarama::ac(Path::new(&sayfa.dosya))?;
        let kucuk = img.resize(1000, 1000, image::imageops::FilterType::Triangle);
        let d = tarama::duzelt(&kucuk, &sayfa.kose, sayfa.donus, sayfa.filtre, 520);
        Ok(veri_url(&tarama::jpeg(&d, 80)))
    })
    .await
}

#[tauri::command]
async fn tarama_pdf(sayfalar: Vec<TaramaSayfa>) -> Result<Vec<String>, String> {
    use rayon::prelude::*;
    arka_planda(move || {
        let ilk = PathBuf::from(&sayfalar.first().ok_or("Sayfa yok")?.dosya);
        let jpegler = sayfalar
            .par_iter()
            .map(|s| {
                let img = tarama::ac(Path::new(&s.dosya))?;
                // 2400 px uzun kenar A4'te ~200 dpi eder.
                let d = tarama::duzelt(&img, &s.kose, s.donus, s.filtre, 2400);
                let gri = matches!(d, image::DynamicImage::ImageLuma8(_));
                Ok(pdf::JpegSayfa { veri: tarama::jpeg(&d, if gri { 80 } else { 85 }), gen: d.width(), yuk: d.height(), gri })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let hedef = benzersiz(&klasoru(&ilk), &format!("{} - tarama", govde(&ilk)), "pdf", &mut HashSet::new());
        let n = pdf::jpeglerden(jpegler, &hedef)?;
        Ok(vec![hedef.to_string_lossy().into_owned(), format!("{n} sayfa")])
    })
    .await
}

#[tauri::command]
async fn sayfa_sayisi(dosya: String) -> Result<u32, String> {
    arka_planda(move || pdf::sayfa_sayisi(Path::new(&dosya))).await
}

/// Çıktıyı kaynağın yanına "<ad> - <ek>.pdf" olarak yazan tek dosyalık işler için ortak kalıp.
fn yanina(dosya: &str, ek: &str) -> (PathBuf, PathBuf) {
    let k = PathBuf::from(dosya);
    let hedef = benzersiz(&klasoru(&k), &format!("{} - {ek}", govde(&k)), "pdf", &mut HashSet::new());
    (k, hedef)
}

fn kb(b: u64) -> String {
    if b >= 1024 * 1024 { format!("{:.1} MB", b as f64 / 1048576.0) } else { format!("{} KB", b / 1024) }
}

#[tauri::command]
async fn sikistir(dosya: String, seviye: u8) -> Result<Vec<String>, String> {
    arka_planda(move || {
        let (k, hedef) = yanina(&dosya, "sıkıştırılmış");
        let (once, sonra, _) = duzen::sikistir(&k, &hedef, seviye)?;
        if sonra >= once {
            let _ = std::fs::remove_file(&hedef);
            return Err(format!("Daha fazla küçültülemedi ({}). Dosya zaten sıkıştırılmış görünüyor.", kb(once)));
        }
        let oran = 100 - sonra * 100 / once.max(1);
        Ok(vec![hedef.to_string_lossy().into_owned(), format!("{} → {} (%{oran} küçük)", kb(once), kb(sonra))])
    })
    .await
}

#[tauri::command]
async fn parola_koy(dosya: String, parola: String) -> Result<Vec<String>, String> {
    arka_planda(move || {
        let (k, hedef) = yanina(&dosya, "parolalı");
        duzen::parola_koy(&k, &hedef, &parola)?;
        Ok(vec![hedef.to_string_lossy().into_owned(), "AES-256 ile şifrelendi".into()])
    })
    .await
}

#[tauri::command]
async fn parola_kaldir(dosya: String, parola: String) -> Result<Vec<String>, String> {
    arka_planda(move || {
        let (k, hedef) = yanina(&dosya, "parolasız");
        duzen::parola_kaldir(&k, &hedef, &parola)?;
        Ok(vec![hedef.to_string_lossy().into_owned(), "Parola kaldırıldı".into()])
    })
    .await
}

#[tauri::command]
async fn sayfa_numarasi(dosya: String, konum: String, bicim: String, baslangic: u32) -> Result<Vec<String>, String> {
    arka_planda(move || {
        let (k, hedef) = yanina(&dosya, "numaralı");
        let n = duzen::sayfa_numarasi(&k, &hedef, &konum, &bicim, baslangic.max(1), 10.5)?;
        Ok(vec![hedef.to_string_lossy().into_owned(), format!("{n} sayfa numaralandı")])
    })
    .await
}

#[tauri::command]
async fn filigran(dosya: String, metin: String, opaklik: f32) -> Result<Vec<String>, String> {
    arka_planda(move || {
        let (k, hedef) = yanina(&dosya, "filigranlı");
        let n = duzen::filigran(&k, &hedef, &metin, opaklik.clamp(0.05, 0.9))?;
        Ok(vec![hedef.to_string_lossy().into_owned(), format!("{n} sayfa")])
    })
    .await
}

/// Metin düzenleyicinin pdf.js'e vereceği ham dosya baytları.
#[tauri::command]
async fn dosya_oku(dosya: String) -> Result<tauri::ipc::Response, String> {
    arka_planda(move || std::fs::read(&dosya).map(tauri::ipc::Response::new).map_err(|e| e.to_string())).await
}

#[tauri::command]
async fn metin_kaydet(dosya: String, degisiklikler: Vec<duzen::MetinDegisiklik>) -> Result<Vec<String>, String> {
    arka_planda(move || {
        let (k, hedef) = yanina(&dosya, "düzenlenmiş");
        let n = duzen::metinleri_yaz(&k, &hedef, &degisiklikler)?;
        Ok(vec![hedef.to_string_lossy().into_owned(), format!("{n} değişiklik")])
    })
    .await
}

#[tauri::command]
fn ac(yol: String) {
    let _ = Command::new("explorer").arg(yol).spawn();
}

#[tauri::command]
fn klasorde_goster(yol: String) {
    let _ = Command::new("explorer").raw_arg(format!("/select,\"{yol}\"")).spawn();
}

/// Belge köşesi modeli yüklenebildi mi (ilk çağrıda ~0,4 sn'de yüklenir ve bellekte kalır).
#[tauri::command]
async fn model_durumu() -> Result<bool, String> {
    arka_planda(|| Ok(model::var_mi())).await
}

/// Office uygulamalarının kayıtlı olup olmadığı (arayüz uyarısı için).
#[tauri::command]
fn office_durumu() -> Vec<String> {
    ["PowerPoint", "Word", "Excel"]
        .into_iter()
        .filter(|a| {
            Command::new("reg")
                .args(["query", &format!("HKCR\\{a}.Application\\CLSID")])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .creation_flags(CREATE_NO_WINDOW)
                .status()
                .is_ok_and(|s| s.success())
        })
        .map(String::from)
        .collect()
}

fn main() {
    // Konsol çıktısı yok; panik olursa en azından geçici klasöre not düş.
    std::panic::set_hook(Box::new(|bilgi| {
        if let Ok(mut f) = std::fs::File::create(std::env::temp_dir().join("chameleon-panik.txt")) {
            let _ = writeln!(f, "{bilgi}");
        }
    }));
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            use tauri::Manager;
            // Gizli bir süreçten başlatılınca Windows ilk pencereyi simge durumunda açabiliyor.
            if let Some(p) = app.get_webview_window("main") {
                if p.is_minimized().unwrap_or(false) {
                    let _ = p.unminimize();
                }
                let _ = p.set_focus();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            office,
            birlestir,
            bol,
            sayfa_cikar,
            dondur,
            gorsellerden_pdf,
            pdf_gorsel,
            tarama_hazirla,
            tarama_onizle,
            tarama_pdf,
            sikistir,
            parola_koy,
            parola_kaldir,
            sayfa_numarasi,
            filigran,
            dosya_oku,
            metin_kaydet,
            sayfa_sayisi,
            ac,
            klasorde_goster,
            office_durumu,
            model_durumu
        ])
        .run(tauri::generate_context!())
        .expect("Chameleon PDF başlatılamadı");
}
