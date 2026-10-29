//! Var olan sayfalara içerik ekleyen ve belgeyi yeniden yazan işlemler:
//! sayfa numarası, filigran, metin üst katmanı, sıkıştırma, parola.

use crate::dil;
use crate::pdf::{self, Sonuc};
use crate::yazi;
use lopdf::{dictionary, Dictionary, Document, Object, ObjectId, Stream};
use std::path::Path;

fn sayilar(o: &Object) -> Option<[f32; 4]> {
    let a = o.as_array().ok()?;
    if a.len() != 4 {
        return None;
    }
    let mut v = [0.0; 4];
    for (i, x) in a.iter().enumerate() {
        v[i] = x.as_float().ok()?;
    }
    Some([v[0].min(v[2]), v[1].min(v[3]), v[0].max(v[2]), v[1].max(v[3])])
}

/// Görünen kutu (CropBox, yoksa MediaBox) ve /Rotate.
fn kutu_ve_donus(d: &Document, id: ObjectId) -> ([f32; 4], i64) {
    let sayfa = d.get_dictionary(id).cloned().unwrap_or_default();
    let miras = pdf::miras(d, id);
    let bak = |k: &[u8]| sayfa.get(k).ok().or_else(|| miras.get(k).ok()).map(|o| match o {
        Object::Reference(r) => d.get_object(*r).cloned().unwrap_or(Object::Null),
        x => x.clone(),
    });
    let kutu = bak(b"CropBox").and_then(|o| sayilar(&o)).or_else(|| bak(b"MediaBox").and_then(|o| sayilar(&o))).unwrap_or([0.0, 0.0, 595.0, 842.0]);
    let donus = bak(b"Rotate").and_then(|o| o.as_i64().ok()).unwrap_or(0).rem_euclid(360);
    (kutu, donus)
}

/// Okuyucunun gösterdiği haliyle sayfa (sol alt 0,0; döndürme uygulanmış) → kullanıcı uzayı
/// dönüşümü ve görünen genişlik/yükseklik. Böylece yazılar döndürülmüş sayfada da düz durur.
fn gorunur(d: &Document, id: ObjectId) -> ([f32; 6], f32, f32) {
    let ([x0, y0, x1, y1], r) = kutu_ve_donus(d, id);
    let (w, h) = (x1 - x0, y1 - y0);
    match r {
        90 => ([0.0, 1.0, -1.0, 0.0, x0 + w, y0], h, w),
        180 => ([-1.0, 0.0, 0.0, -1.0, x0 + w, y0 + h], w, h),
        270 => ([0.0, -1.0, 1.0, 0.0, x0, y0 + h], h, w),
        _ => ([1.0, 0.0, 0.0, 1.0, x0, y0], w, h),
    }
}

fn matris(m: [f32; 6]) -> String {
    m.iter().map(|v| format!("{v:.4}")).collect::<Vec<_>>().join(" ")
}

fn cozumle(d: &Document, o: &Object) -> Option<Dictionary> {
    match o {
        Object::Reference(r) => d.get_dictionary(*r).ok().cloned(),
        Object::Dictionary(x) => Some(x.clone()),
        _ => None,
    }
}

/// Sayfanın mevcut içeriğini q/Q ile sarar (kalan grafik durumu bizi etkilemesin), sonuna `icerik`
/// ekler ve verilen kaynakları sayfanın Font/ExtGState sözlüklerine katar.
pub fn ust_katman_ekle(d: &mut Document, sayfa: ObjectId, icerik: String, fontlar: &[(&str, ObjectId)], durumlar: &[(&str, ObjectId)]) -> Sonuc<()> {
    let s = d.get_dictionary(sayfa).map_err(|e| e.to_string())?.clone();
    let mut kaynak = s
        .get(b"Resources")
        .ok()
        .and_then(|o| cozumle(d, o))
        .or_else(|| pdf::miras(d, sayfa).get(b"Resources").ok().and_then(|o| cozumle(d, o)))
        .unwrap_or_default();
    for (anahtar, eklenecek) in [(&b"Font"[..], fontlar), (&b"ExtGState"[..], durumlar)] {
        if eklenecek.is_empty() {
            continue;
        }
        let mut alt = kaynak.get(anahtar).ok().and_then(|o| cozumle(d, o)).unwrap_or_default();
        for (ad, id) in eklenecek {
            alt.set(*ad, Object::Reference(*id));
        }
        kaynak.set(anahtar, Object::Dictionary(alt));
    }
    let mut parcalar = match s.get(b"Contents") {
        Ok(Object::Reference(r)) => vec![Object::Reference(*r)],
        Ok(Object::Array(a)) => a.clone(),
        _ => vec![],
    };
    let bas = d.add_object(Stream::new(dictionary! {}, b"q\n".to_vec()));
    let son = d.add_object(Stream::new(dictionary! {}, format!("Q\n{icerik}\n").into_bytes()));
    parcalar.insert(0, Object::Reference(bas));
    parcalar.push(Object::Reference(son));
    let sd = d.get_dictionary_mut(sayfa).map_err(|e| e.to_string())?;
    sd.set("Resources", Object::Dictionary(kaynak));
    sd.set("Contents", Object::Array(parcalar));
    Ok(())
}

pub fn sayfa_numarasi(yol: &Path, cikti: &Path, konum: &str, bicim: &str, baslangic: u32, boyut: f32) -> Sonuc<usize> {
    let mut d = pdf::yukle(yol)?;
    let sayfalar: Vec<ObjectId> = d.get_pages().into_values().collect();
    let toplam = sayfalar.len() as u32;
    let metinler: Vec<String> = (0..toplam)
        .map(|i| bicim.replace("{n}", &(baslangic + i).to_string()).replace("{N}", &(baslangic + toplam - 1).to_string()))
        .collect();
    let yazi = yazi::gom(&mut d, false, metinler.iter().map(String::as_str))?;
    for (id, metin) in sayfalar.iter().zip(&metinler) {
        let (m, w, h) = gorunur(&d, *id);
        let (hex, gen) = yazi.kodla(metin);
        let tw = gen * boyut;
        let kenar = 28.0f32.min(w * 0.06);
        let x = if konum.ends_with("sag") { w - kenar - tw } else if konum.ends_with("sol") { kenar } else { (w - tw) / 2.0 };
        let y = if konum.starts_with("ust") { h - kenar - boyut * 0.75 } else { kenar * 0.8 };
        let icerik = format!("q {} cm BT /ChmF0 {boyut} Tf 0.2 0.2 0.25 rg {x:.2} {y:.2} Td {hex} Tj ET Q", matris(m));
        ust_katman_ekle(&mut d, *id, icerik, &[("ChmF0", yazi.id)], &[])?;
    }
    pdf::kaydet(d, cikti)?;
    Ok(toplam as usize)
}

pub fn filigran(yol: &Path, cikti: &Path, metin: &str, opaklik: f32) -> Sonuc<usize> {
    let metin = metin.trim();
    if metin.is_empty() {
        return Err(dil::t("Filigran metni boş", "The watermark text is empty").into());
    }
    let mut d = pdf::yukle(yol)?;
    let sayfalar: Vec<ObjectId> = d.get_pages().into_values().collect();
    let yazi = yazi::gom(&mut d, true, [metin])?;
    let durum = d.add_object(dictionary! { "Type" => "ExtGState", "ca" => opaklik, "CA" => opaklik });
    let (hex, gen) = yazi.kodla(metin);
    for id in &sayfalar {
        let (m, w, h) = gorunur(&d, *id);
        let kosegen = (w * w + h * h).sqrt();
        // Köşegenin ~%70'ini kaplasın, ama kısa metinler dev olmasın.
        let boyut = (kosegen * 0.7 / gen.max(0.1)).min(h * 0.22);
        let aci = h.atan2(w);
        let (c, s) = (aci.cos(), aci.sin());
        let icerik = format!(
            "q {} cm /ChmGS0 gs BT /ChmF1 {boyut:.2} Tf 0.45 0.47 0.55 rg {c:.4} {s:.4} {:.4} {c:.4} {:.2} {:.2} Tm {:.2} {:.2} Td {hex} Tj ET Q",
            matris(m),
            -s,
            w / 2.0,
            h / 2.0,
            -gen * boyut / 2.0,
            -boyut * 0.33
        );
        ust_katman_ekle(&mut d, *id, icerik, &[("ChmF1", yazi.id)], &[("ChmGS0", durum)])?;
    }
    pdf::kaydet(d, cikti)?;
    Ok(sayfalar.len())
}

/// Sayfaya eklenecek bir metin: eski yazının üstü `zemin` rengiyle örtülür, yeni metin yazılır.
#[derive(serde::Deserialize)]
pub struct MetinDegisiklik {
    /// 0 tabanlı sayfa sırası.
    pub sayfa: usize,
    /// Örtülecek alan, kullanıcı uzayında [x, y, genişlik, yükseklik]; boşsa örtme yapılmaz.
    pub ort: Option<[f32; 4]>,
    pub zemin: [f32; 3],
    /// Yazının taban çizgisinin başlangıcı (kullanıcı uzayı).
    pub x: f32,
    pub y: f32,
    pub boyut: f32,
    pub renk: [f32; 3],
    pub kalin: bool,
    pub metin: String,
}

pub fn metinleri_yaz(yol: &Path, cikti: &Path, degisiklikler: &[MetinDegisiklik]) -> Sonuc<usize> {
    let mut d = pdf::yukle(yol)?;
    let sayfalar: Vec<ObjectId> = d.get_pages().into_values().collect();
    let duz = yazi::gom(&mut d, false, degisiklikler.iter().filter(|g| !g.kalin).map(|g| g.metin.as_str()))?;
    let kalin = yazi::gom(&mut d, true, degisiklikler.iter().filter(|g| g.kalin).map(|g| g.metin.as_str()))?;
    for (i, id) in sayfalar.iter().enumerate() {
        let burada: Vec<&MetinDegisiklik> = degisiklikler.iter().filter(|g| g.sayfa == i).collect();
        if burada.is_empty() {
            continue;
        }
        let mut icerik = String::new();
        for g in burada {
            if let Some([x, y, w, h]) = g.ort {
                let [r, gg, b] = g.zemin;
                icerik.push_str(&format!("{r:.3} {gg:.3} {b:.3} rg {x:.2} {y:.2} {w:.2} {h:.2} re f\n"));
            }
            for (satir_no, satir) in g.metin.lines().enumerate() {
                let (yz, ad) = if g.kalin { (&kalin, "ChmF3") } else { (&duz, "ChmF2") };
                let (hex, _) = yz.kodla(satir);
                let [r, gg, b] = g.renk;
                let y = g.y - satir_no as f32 * g.boyut * 1.2;
                icerik.push_str(&format!("BT /{ad} {:.2} Tf {r:.3} {gg:.3} {b:.3} rg {:.2} {y:.2} Td {hex} Tj ET\n", g.boyut, g.x));
            }
        }
        ust_katman_ekle(&mut d, *id, icerik, &[("ChmF2", duz.id), ("ChmF3", kalin.id)], &[])?;
    }
    pdf::kaydet(d, cikti)?;
    Ok(degisiklikler.len())
}

/// Görselleri küçültüp yeniden sıkıştırır. Dönüş: (önceki bayt, sonraki bayt, değişen görsel).
pub fn sikistir(yol: &Path, cikti: &Path, seviye: u8) -> Sonuc<(u64, u64, usize)> {
    let (en_buyuk, kalite) = match seviye {
        1 => (2400, 82),
        3 => (1100, 55),
        _ => (1600, 70),
    };
    let once = std::fs::metadata(yol).map(|m| m.len()).unwrap_or(0);
    let mut d = pdf::yukle(yol)?;
    let gorseller: Vec<ObjectId> = d
        .objects
        .iter()
        .filter(|(_, o)| matches!(o, Object::Stream(s) if s.dict.get(b"Subtype").and_then(Object::as_name).is_ok_and(|n| n == b"Image")))
        .map(|(id, _)| *id)
        .collect();
    let mut degisen = 0;
    for id in gorseller {
        let Ok(Object::Stream(akis)) = d.get_object(id) else { continue };
        if let Some(yeni) = gorsel_kucult(&d, akis, en_buyuk, kalite) {
            d.objects.insert(id, Object::Stream(yeni));
            degisen += 1;
        }
    }
    d.prune_objects();
    d.renumber_objects();
    d.compress();
    let mut dosya = std::io::BufWriter::new(std::fs::File::create(cikti).map_err(|e| e.to_string())?);
    d.save_with_options(&mut dosya, lopdf::SaveOptions::builder().use_object_streams(true).use_xref_streams(true).build())
        .map_err(|e| format!("{} yazılamadı: {e}", pdf::ad(cikti)))?;
    drop(dosya);
    let sonra = std::fs::metadata(cikti).map(|m| m.len()).unwrap_or(0);
    Ok((once, sonra, degisen))
}

fn gorsel_kucult(d: &Document, akis: &Stream, en_buyuk: u32, kalite: u8) -> Option<Stream> {
    let s = &akis.dict;
    if s.get(b"ImageMask").and_then(Object::as_bool).unwrap_or(false) || s.has(b"Decode") {
        return None;
    }
    if s.get(b"BitsPerComponent").and_then(Object::as_i64).ok()? != 8 {
        return None;
    }
    let w = s.get(b"Width").and_then(Object::as_i64).ok()? as u32;
    let h = s.get(b"Height").and_then(Object::as_i64).ok()? as u32;
    // Kanal sayısı: DeviceRGB/DeviceGray ya da N'si 1/3 olan ICCBased. Diğerlerine dokunma.
    let kanal = match s.get(b"ColorSpace").ok()? {
        Object::Name(n) if n == b"DeviceRGB" => 3,
        Object::Name(n) if n == b"DeviceGray" => 1,
        Object::Array(a) if a.first().and_then(|x| x.as_name().ok()) == Some(b"ICCBased") => {
            let r = a.get(1)?.as_reference().ok()?;
            match d.get_object(r).ok()? {
                Object::Stream(icc) => icc.dict.get(b"N").and_then(Object::as_i64).ok()? as u32,
                _ => return None,
            }
        }
        Object::Reference(r) => match d.get_object(*r).ok()? {
            Object::Name(n) if n == b"DeviceRGB" => 3,
            Object::Name(n) if n == b"DeviceGray" => 1,
            _ => return None,
        },
        _ => return None,
    };
    if kanal != 1 && kanal != 3 {
        return None;
    }
    let filtre = match s.get(b"Filter").ok() {
        Some(Object::Name(n)) => n.clone(),
        Some(Object::Array(a)) if a.len() == 1 => a[0].as_name().ok()?.to_vec(),
        None => b"".to_vec(),
        _ => return None,
    };
    let img = match filtre.as_slice() {
        b"DCTDecode" => image::load_from_memory_with_format(&akis.content, image::ImageFormat::Jpeg).ok()?,
        b"FlateDecode" | b"" => {
            let ham = if filtre.is_empty() { akis.content.clone() } else { akis.decompressed_content().ok()? };
            if ham.len() < (w * h * kanal) as usize {
                return None;
            }
            // Küçük düz renkli görseller (logo, simge) JPEG'de bozulur; dokunma.
            if w.max(h) < 300 {
                return None;
            }
            if kanal == 3 {
                image::DynamicImage::ImageRgb8(image::RgbImage::from_raw(w, h, ham[..(w * h * 3) as usize].to_vec())?)
            } else {
                image::DynamicImage::ImageLuma8(image::GrayImage::from_raw(w, h, ham[..(w * h) as usize].to_vec())?)
            }
        }
        _ => return None,
    };
    let img = if img.width().max(img.height()) > en_buyuk {
        img.resize(en_buyuk, en_buyuk, image::imageops::FilterType::Triangle)
    } else {
        img
    };
    let gri = kanal == 1 || matches!(img, image::DynamicImage::ImageLuma8(_));
    let img = if gri { image::DynamicImage::ImageLuma8(img.to_luma8()) } else { image::DynamicImage::ImageRgb8(img.to_rgb8()) };
    let veri = crate::tarama::jpeg(&img, kalite);
    // Kazanç %10'dan azsa eskisi kalsın.
    if veri.len() as f32 > akis.content.len() as f32 * 0.9 {
        return None;
    }
    let mut yeni = s.clone();
    yeni.remove(b"DecodeParms");
    yeni.remove(b"Length");
    yeni.set("Filter", "DCTDecode");
    yeni.set("Width", img.width() as i64);
    yeni.set("Height", img.height() as i64);
    if gri && kanal == 3 {
        yeni.set("ColorSpace", "DeviceGray");
    }
    let mut akis = Stream::new(yeni, veri);
    akis.allows_compression = false;
    Some(akis)
}

pub fn parola_koy(yol: &Path, cikti: &Path, parola: &str) -> Sonuc<()> {
    if parola.is_empty() {
        return Err(dil::t("Parola boş olamaz", "The password cannot be empty").into());
    }
    let mut d = pdf::yukle(yol)?;
    let anahtar = yazi::rastgele32()?;
    let suzgec: std::sync::Arc<dyn lopdf::encryption::crypt_filters::CryptFilter> =
        std::sync::Arc::new(lopdf::encryption::crypt_filters::Aes256CryptFilter);
    let durum = lopdf::EncryptionState::try_from(lopdf::EncryptionVersion::V5 {
        encrypt_metadata: true,
        crypt_filters: std::collections::BTreeMap::from([(b"StdCF".to_vec(), suzgec)]),
        file_encryption_key: &anahtar,
        stream_filter: b"StdCF".to_vec(),
        string_filter: b"StdCF".to_vec(),
        owner_password: parola,
        user_password: parola,
        permissions: lopdf::Permissions::all(),
    })
    .map_err(|e| format!("Şifreleme hazırlanamadı: {e}"))?;
    d.encrypt(&durum).map_err(|e| format!("Şifrelenemedi: {e}"))?;
    d.save(cikti).map(|_| ()).map_err(|e| format!("{} yazılamadı: {e}", pdf::ad(cikti)))
}

pub fn parola_kaldir(yol: &Path, cikti: &Path, parola: &str) -> Sonuc<()> {
    let mut d = Document::load_with_password(yol, parola).map_err(|e| {
        let m = e.to_string().to_lowercase();
        if m.contains("password") || m.contains("decrypt") { dil::t("Parola yanlış", "Wrong password").to_string() } else { format!("Açılamadı: {e}") }
    })?;
    if !d.was_encrypted() && !d.is_encrypted() {
        return Err(dil::t("Bu PDF'te parola yok", "This PDF has no password").into());
    }
    if d.is_encrypted() {
        d.decrypt(parola).map_err(|_| dil::t("Parola yanlış", "Wrong password").to_string())?;
    }
    d.save(cikti).map(|_| ()).map_err(|e| format!("{} yazılamadı: {e}", pdf::ad(cikti)))
}
