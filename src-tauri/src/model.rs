//! Belge köşesi modelleri. Uygulamayla gelmezler: kullanıcı listeden birini seçer ve model,
//! sahibi olan projenin kendi kaynağından indirilir. Şu an hepsi DocsaidLab/DocAligner'ın
//! (Apache-2.0) ısı haritası modelleri; aynı ön/son işlemi paylaşırlar.

use image::DynamicImage;
use ort::{environment::Environment, session::Session, value::Tensor};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, OnceLock};

pub struct Tanim {
    pub kimlik: &'static str,
    pub ad: &'static str,
    pub aciklama: &'static str,
    pub aciklama_en: &'static str,
    dosya: &'static str,
    boyut: u64,
    sha256: &'static str,
    drive_id: &'static str,
}

/// Doğruluk notları, 16 zor ve 10 kolay sentetik fotoğrafla ölçüldü (bkz. README).
pub const KATALOG: &[Tanim] = &[
    Tanim {
        kimlik: "docaligner-sa24",
        ad: "DocAligner FastViT-SA24",
        aciklama: "En doğru; testlerde zor fotoğrafların 16/16'sı. 83 MB, ~0,1 sn",
        aciklama_en: "Most accurate; 16/16 hard photos in tests. 83 MB, ~0.1 s",
        dosya: "docaligner-fastvit_sa24.onnx",
        boyut: 83_084_930,
        sha256: "7f9f5a8935b2eb22b3ee0245d34996063f54562df390d34714af2d76928695bc",
        drive_id: "14vUH77v6yGg7zFctUgcT6BzV5Iisg4Dl",
    },
    Tanim {
        kimlik: "docaligner-lcnet100",
        ad: "DocAligner LCNet100",
        aciklama: "Çok küçük ve hızlı; zor fotoğraflarda iyi, düz taramalarda daha zayıf. 5 MB",
        aciklama_en: "Very small and fast; good on hard photos, weaker on flat scans. 5 MB",
        dosya: "docaligner-lcnet100.onnx",
        boyut: 4_767_987,
        sha256: "f4117b786e3a18470f3865c93f3c2bd69d9b998edd60f385574a5c665e79594e",
        drive_id: "1IlbLPkCv-TdaBLOPh_4J97P1_KHYYJ7a",
    },
    Tanim {
        kimlik: "docaligner-t8",
        ad: "DocAligner FastViT-T8",
        aciklama: "Orta boy; sonuçları fotoğrafa göre değişken. 13 MB",
        aciklama_en: "Medium size; results vary from photo to photo. 13 MB",
        dosya: "docaligner-fastvit_t8.onnx",
        boyut: 13_167_814,
        sha256: "950070fd4a46e25d46db7c35d0ed77612d9063f70d4543c7c504f0da6a943a89",
        drive_id: "1W88oOkHjMza6xXbDxrG8k_C-Tf2aYKJ4",
    },
];

const GIRDI: u32 = 256;
const HARITA: usize = 128;
/// DocAligner infer.py'deki heatmap_threshold.
const ESIK: f32 = 0.3;

fn tanim(kimlik: &str) -> Option<&'static Tanim> {
    KATALOG.iter().find(|t| t.kimlik == kimlik)
}

/// İndirilen modellerin klasörü: %LOCALAPPDATA%\com.bukalemun.masaustu\modeller
fn indirme_klasoru() -> PathBuf {
    let kok = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    kok.join("com.bukalemun.masaustu").join("modeller")
}

/// Modelin diskteki yeri; önce indirme klasörü, sonra exe'nin yanı (1.0.0 zip'i) ve geliştirme ağacı.
fn bul(t: &Tanim) -> Option<PathBuf> {
    let mut adaylar = vec![indirme_klasoru().join(t.dosya)];
    if let Some(k) = std::env::current_exe().ok().and_then(|e| e.parent().map(PathBuf::from)) {
        for alt in ["modeller", "../../modeller", "../../../modeller"] {
            adaylar.push(k.join(alt).join(t.dosya));
        }
    }
    adaylar.into_iter().find(|p| std::fs::metadata(p).is_ok_and(|m| m.len() == t.boyut))
}

#[derive(Serialize)]
pub struct Durum {
    kimlik: &'static str,
    ad: &'static str,
    aciklama: &'static str,
    mb: f32,
    indirildi: bool,
}

pub fn durumlar() -> Vec<Durum> {
    KATALOG
        .iter()
        .map(|t| Durum { kimlik: t.kimlik, ad: t.ad, aciklama: if crate::dil::en() { t.aciklama_en } else { t.aciklama }, mb: t.boyut as f32 / 1_000_000.0, indirildi: bul(t).is_some() })
        .collect()
}

/// Modeli kaynağından indirir, boyut ve SHA-256 ile doğrular. `ilerle(inen, toplam)` yaklaşık her MB'ta çağrılır.
pub fn indir(kimlik: &str, ilerle: impl Fn(u64, u64)) -> Result<(), String> {
    let t = tanim(kimlik).ok_or("Bilinmeyen model")?;
    if bul(t).is_some() {
        return Ok(());
    }
    let klasor = indirme_klasoru();
    std::fs::create_dir_all(&klasor).map_err(|e| e.to_string())?;
    let gecici = klasor.join(format!("{}.indiriliyor", t.dosya));
    let url = format!("https://drive.usercontent.google.com/download?id={}&export=download&confirm=t", t.drive_id);
    let yanit = ureq::get(&url).call().map_err(|e| crate::dil::s(format!("İndirilemedi: {e}"), format!("Download failed: {e}")))?;
    let mut okuyucu = yanit.into_reader();
    let mut dosya = std::fs::File::create(&gecici).map_err(|e| e.to_string())?;
    let mut ozet = Sha256::new();
    let mut tampon = vec![0u8; 256 * 1024];
    let (mut inen, mut son_bildirim) = (0u64, 0u64);
    loop {
        let n = okuyucu.read(&mut tampon).map_err(|e| crate::dil::s(format!("İndirme kesildi: {e}"), format!("Download interrupted: {e}")))?;
        if n == 0 {
            break;
        }
        dosya.write_all(&tampon[..n]).map_err(|e| e.to_string())?;
        ozet.update(&tampon[..n]);
        inen += n as u64;
        if inen - son_bildirim >= 1_048_576 {
            son_bildirim = inen;
            ilerle(inen, t.boyut);
        }
    }
    drop(dosya);
    let hex: String = ozet.finalize().iter().map(|b| format!("{b:02x}")).collect();
    if inen != t.boyut || hex != t.sha256 {
        let _ = std::fs::remove_file(&gecici);
        return Err(crate::dil::s(format!("İndirilen dosya beklenen model değil ({} bayt). Kaynak değişmiş olabilir.", inen), format!("The downloaded file is not the expected model ({} bytes). The source may have changed.", inen)));
    }
    std::fs::rename(&gecici, klasor.join(t.dosya)).map_err(|e| e.to_string())?;
    ilerle(t.boyut, t.boyut);
    Ok(())
}

pub fn sil(kimlik: &str) -> Result<(), String> {
    let t = tanim(kimlik).ok_or("Bilinmeyen model")?;
    if let Some(h) = OTURUMLAR.lock().map_err(|e| e.to_string())?.as_mut() {
        h.remove(kimlik);
    }
    let p = indirme_klasoru().join(t.dosya);
    if p.exists() {
        std::fs::remove_file(p).map_err(|e| e.to_string())?;
    }
    Ok(())
}

static ORTAM: OnceLock<Option<Environment>> = OnceLock::new();
static OTURUMLAR: Mutex<Option<HashMap<String, Arc<Mutex<Session>>>>> = Mutex::new(None);

/// İlk kullanımda yüklenir (~0,4 sn) ve bellekte kalır.
fn oturum(kimlik: &str) -> Option<Arc<Mutex<Session>>> {
    let mut kilit = OTURUMLAR.lock().ok()?;
    let harita = kilit.get_or_insert_with(HashMap::new);
    if let Some(o) = harita.get(kimlik) {
        return Some(o.clone());
    }
    let yol = bul(tanim(kimlik)?)?;
    let ortam = ORTAM.get_or_init(|| ort::init().with_name("chameleon").build().ok()).as_ref()?;
    let o = Arc::new(Mutex::new(Session::builder(ortam).ok()?.commit_from_file(yol).ok()?));
    harita.insert(kimlik.to_string(), o.clone());
    Some(o)
}

#[cfg(test)]
pub fn var_mi(kimlik: &str) -> bool {
    oturum(kimlik).is_some()
}

/// Dört köşe (sol üst, sağ üst, sağ alt, sol alt; 0..1). Model indirilmemişse ya da bir köşe bulunamazsa None.
pub fn koseler(img: &DynamicImage, kimlik: &str) -> Option<[[f32; 2]; 4]> {
    let o = oturum(kimlik)?;
    let girdi = Tensor::from_array(([1usize, 3, GIRDI as usize, GIRDI as usize], girdi_hazirla(img))).ok()?;
    let mut oturum = o.lock().ok()?;
    let cikti = oturum.run(ort::inputs!["img" => girdi]).ok()?;
    let (_, isi) = cikti["heatmap"].try_extract_tensor::<f32>().ok()?;

    // DocAligner'ın kendi kuralı: dört köşenin dördü de eşiği aşmazsa sonuç yok. Fotoğrafta köşesi
    // kadrajın dışında kalan perde/slayt fotoğraflarında bu sık oluyor; o yüzden zayıf köşede önce
    // haritanın tepe noktasına bakılır, o da yoksa köşe fotoğrafın o köşesine konur (en az iki köşe şartıyla).
    let mut bulunan: [Option<[f32; 2]>; 4] = [None; 4];
    for (c, kose) in bulunan.iter_mut().enumerate() {
        let harita = &isi[c * HARITA * HARITA..(c + 1) * HARITA * HARITA];
        *kose = merkez(harita).or_else(|| tepe(harita));
    }
    if bulunan.iter().filter(|k| k.is_some()).count() < 2 {
        return None;
    }
    const KENAR: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    Some(std::array::from_fn(|i| bulunan[i].unwrap_or(KENAR[i])))
}

/// infer.py'deki gibi: en-boy korunmadan 256x256, BGR, /255, NCHW. Küçültme OpenCV'nin
/// INTER_LINEAR'ı gibi yapılır (yumuşatmasız iki doğrusal örnekleme). Model bununla eğitildiği için
/// yumuşatan bir filtre (Triangle) bazı fotoğraflarda köşe haritasını belirgin biçimde değiştiriyordu.
fn girdi_hazirla(img: &DynamicImage) -> Vec<f32> {
    let rgb = img.to_rgb8();
    let (w, h) = (rgb.width() as f32, rgb.height() as f32);
    let (sx, sy) = (w / GIRDI as f32, h / GIRDI as f32);
    let alan = (GIRDI * GIRDI) as usize;
    let mut veri = vec![0f32; 3 * alan];
    let al = |x: u32, y: u32| rgb.get_pixel(x.min(rgb.width() - 1), y.min(rgb.height() - 1));
    for oy in 0..GIRDI {
        let fy = ((oy as f32 + 0.5) * sy - 0.5).max(0.0);
        let (y0, ty) = (fy.floor() as u32, fy.fract());
        for ox in 0..GIRDI {
            let fx = ((ox as f32 + 0.5) * sx - 0.5).max(0.0);
            let (x0, tx) = (fx.floor() as u32, fx.fract());
            let (a, b, c, d) = (al(x0, y0), al(x0 + 1, y0), al(x0, y0 + 1), al(x0 + 1, y0 + 1));
            let i = (oy * GIRDI + ox) as usize;
            for k in 0..3 {
                let v = (a[k] as f32 * (1.0 - tx) + b[k] as f32 * tx) * (1.0 - ty) + (c[k] as f32 * (1.0 - tx) + d[k] as f32 * tx) * ty;
                // BGR sırası: kanal 0 = mavi.
                veri[(2 - k) * alan + i] = v.round() / 255.0;
            }
        }
    }
    veri
}

/// Eşiğin altında kalan zayıf ama belirgin bir köşe: en yüksek değer 0,08'i geçiyorsa onun yeri.
fn tepe(harita: &[f32]) -> Option<[f32; 2]> {
    let (i, en) = harita.iter().enumerate().fold((0, 0f32), |a, (i, &v)| if v > a.1 { (i, v) } else { a });
    (en >= 0.08).then(|| [((i % HARITA) as f32 + 0.5) / HARITA as f32, ((i / HARITA) as f32 + 0.5) / HARITA as f32])
}

/// Eşiği aşan bağlı bölgelerden (8 komşulu) tepe değeri en yüksek olanın, değerle ağırlıklı merkezi
/// (0..1). DocAligner en büyük alanlı bölgeyi seçer; gölgeli fotoğraflarda yanlış yerde daha geniş
/// ama daha zayıf bir bölge çıkabildiği için burada tepe değeri esas alınır.
fn merkez(harita: &[f32]) -> Option<[f32; 2]> {
    let n = HARITA;
    let mut etiket = vec![0u32; n * n];
    let mut en_iyi: Option<(f32, f32, f32, f32)> = None; // (tepe, Σv·x, Σv·y, Σv)
    let mut sira = 0;
    let mut yigin = Vec::new();
    for bas in 0..n * n {
        if harita[bas] < ESIK || etiket[bas] != 0 {
            continue;
        }
        sira += 1;
        etiket[bas] = sira;
        yigin.push(bas);
        let (mut tepe, mut sx, mut sy, mut sv) = (0f32, 0f32, 0f32, 0f32);
        while let Some(i) = yigin.pop() {
            let (x, y) = (i % n, i / n);
            let v = harita[i];
            tepe = tepe.max(v);
            sx += v * (x as f32 + 0.5);
            sy += v * (y as f32 + 0.5);
            sv += v;
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                    if nx < 0 || ny < 0 || nx >= n as i32 || ny >= n as i32 {
                        continue;
                    }
                    let j = ny as usize * n + nx as usize;
                    if harita[j] >= ESIK && etiket[j] == 0 {
                        etiket[j] = sira;
                        yigin.push(j);
                    }
                }
            }
        }
        if en_iyi.is_none_or(|e| tepe > e.0) {
            en_iyi = Some((tepe, sx, sy, sv));
        }
    }
    let (_, sx, sy, sv) = en_iyi?;
    Some([sx / sv / n as f32, sy / sv / n as f32])
}
