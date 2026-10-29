//! lopdf ile saf Rust PDF işlemleri: birleştir, böl, sayfa çıkar, döndür, görsellerden PDF.

use lopdf::{dictionary, Dictionary, Document, Object, ObjectId, Stream};
use std::path::{Path, PathBuf};

pub type Sonuc<T> = Result<T, String>;

fn hata<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

pub fn yukle(yol: &Path) -> Sonuc<Document> {
    let mut d = Document::load(yol).map_err(|e| format!("{} açılamadı: {e}", ad(yol)))?;
    if d.is_encrypted() {
        // Parolasız (yalnızca izin kısıtlı) dosyalar boş parolayla açılabilir.
        d.decrypt("").map_err(|_| format!("{} parola korumalı", ad(yol)))?;
    }
    Ok(d)
}

pub fn ad(yol: &Path) -> String {
    yol.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
}

pub(crate) fn kaydet(mut d: Document, cikti: &Path) -> Sonuc<()> {
    d.prune_objects();
    d.renumber_objects();
    d.compress();
    d.save(cikti).map(|_| ()).map_err(|e| format!("{} yazılamadı: {e}", ad(cikti)))
}

/// Sayfaya üst düğümlerden miras kalan (sayfada olmayan) öznitelikler.
pub(crate) fn miras(d: &Document, sayfa: ObjectId) -> Dictionary {
    const ANAHTARLAR: [&[u8]; 4] = [b"Resources", b"MediaBox", b"CropBox", b"Rotate"];
    let mut sonuc = Dictionary::new();
    let Ok(dict) = d.get_dictionary(sayfa) else { return sonuc };
    let mut ust = dict.get(b"Parent").and_then(Object::as_reference).ok();
    let mut adim = 0;
    while let (Some(id), true) = (ust, adim < 64) {
        let Ok(u) = d.get_dictionary(id) else { break };
        for k in ANAHTARLAR {
            if !dict.has(k) && !sonuc.has(k) {
                if let Ok(v) = u.get(k) {
                    sonuc.set(k, v.clone());
                }
            }
        }
        ust = u.get(b"Parent").and_then(Object::as_reference).ok();
        adim += 1;
    }
    sonuc
}

fn kok_sayfalar(d: &Document) -> Sonuc<ObjectId> {
    d.catalog()
        .and_then(|c| c.get(b"Pages"))
        .and_then(Object::as_reference)
        .map_err(|_| "PDF'in sayfa ağacı bozuk".to_string())
}

pub fn birlestir(yollar: &[PathBuf], cikti: &Path) -> Sonuc<usize> {
    let (ilk, kalan) = yollar.split_first().ok_or("Birleştirilecek dosya yok")?;
    let mut hedef = yukle(ilk)?;
    let kok = kok_sayfalar(&hedef)?;
    let mut eklenen = Vec::new();

    for yol in kalan {
        let mut d = yukle(yol)?;
        d.renumber_objects_with(hedef.max_id + 1);
        let sayfalar: Vec<ObjectId> = d.get_pages().into_values().collect();
        // Sayfalar yeni ağaca taşınınca eski üst düğümlerden gelen öznitelikler kaybolmasın.
        for &s in &sayfalar {
            let ek = miras(&d, s);
            if let Ok(dict) = d.get_dictionary_mut(s) {
                for (k, v) in ek {
                    dict.set(k, v);
                }
                dict.set("Parent", kok);
            }
        }
        hedef.max_id = hedef.max_id.max(d.max_id);
        hedef.objects.extend(std::mem::take(&mut d.objects));
        eklenen.extend(sayfalar);
    }

    let kok_dict = hedef.get_dictionary_mut(kok).map_err(hata)?;
    let mut cocuklar = kok_dict.get(b"Kids").and_then(Object::as_array).cloned().unwrap_or_default();
    let mut sayi = kok_dict.get(b"Count").and_then(Object::as_i64).unwrap_or(cocuklar.len() as i64);
    sayi += eklenen.len() as i64;
    cocuklar.extend(eklenen.into_iter().map(Object::Reference));
    kok_dict.set("Kids", cocuklar);
    kok_dict.set("Count", sayi);

    let toplam = hedef.get_pages().len();
    kaydet(hedef, cikti)?;
    Ok(toplam)
}

/// "1-3, 5, 8-" gibi bir aralığı 1 tabanlı sayfa listesine çevirir (sırayı ve tekrarları korur).
pub fn aralik_coz(metin: &str, toplam: u32) -> Sonuc<Vec<u32>> {
    let mut sonuc = Vec::new();
    for parca in metin.split([',', ';', ' ']).map(str::trim).filter(|p| !p.is_empty()) {
        let sayi = |s: &str, bos: u32| -> Sonuc<u32> {
            if s.trim().is_empty() {
                return Ok(bos);
            }
            s.trim().parse::<u32>().map_err(|_| format!("'{parca}' anlaşılmadı"))
        };
        let (a, b) = match parca.split_once('-') {
            Some((a, b)) => (sayi(a, 1)?, sayi(b, toplam)?),
            None => {
                let n = sayi(parca, 0)?;
                (n, n)
            }
        };
        if a == 0 || b == 0 || a > toplam || b > toplam {
            return Err(format!("'{parca}' 1–{toplam} aralığının dışında"));
        }
        if a <= b {
            sonuc.extend(a..=b);
        } else {
            sonuc.extend((b..=a).rev());
        }
    }
    if sonuc.is_empty() {
        return Err("Sayfa aralığı boş".into());
    }
    Ok(sonuc)
}

/// Bir nesnenin gösterdiği başvuruları toplar; /Parent izlenmez (sayfa ağacına geri tırmanmasın).
fn baglar(o: &Object, cikis: &mut Vec<ObjectId>) {
    match o {
        Object::Reference(id) => cikis.push(*id),
        Object::Array(a) => a.iter().for_each(|x| baglar(x, cikis)),
        Object::Dictionary(d) => d.iter().filter(|(k, _)| k.as_slice() != b"Parent").for_each(|(_, v)| baglar(v, cikis)),
        Object::Stream(s) => baglar(&Object::Dictionary(s.dict.clone()), cikis),
        _ => {}
    }
}

/// Yalnızca verilen sayfaları (verilen sırayla) içeren yeni bir PDF yazar. Belgenin tamamını
/// kopyalamak yerine yalnızca bu sayfalardan ulaşılan nesneleri taşır; akışlar yeniden sıkıştırılmaz.
fn sayfalari_yaz(kaynak: &Document, sayfalar: &[u32], cikti: &Path) -> Sonuc<()> {
    let tum = kaynak.get_pages();
    let mut d = Document::with_version(kaynak.version.clone());
    d.max_id = kaynak.max_id;
    let kok = d.new_object_id();

    let secili: Vec<ObjectId> = sayfalar.iter().map(|n| tum[n]).collect();
    let mut yigin = secili.clone();
    let mut gorulen = std::collections::HashSet::new();
    while let Some(id) = yigin.pop() {
        if !gorulen.insert(id) {
            continue;
        }
        let Ok(nesne) = kaynak.get_object(id) else { continue };
        let mut nesne = nesne.clone();
        if secili.contains(&id) {
            if let Object::Dictionary(dict) = &mut nesne {
                for (k, v) in miras(kaynak, id) {
                    dict.set(k, v);
                }
                dict.set("Parent", kok);
            }
        }
        baglar(&nesne, &mut yigin);
        d.objects.insert(id, nesne);
    }

    let cocuklar: Vec<Object> = secili.iter().map(|&id| Object::Reference(id)).collect();
    d.objects.insert(
        kok,
        Object::Dictionary(dictionary! { "Type" => "Pages", "Count" => cocuklar.len() as i64, "Kids" => cocuklar }),
    );
    let katalog = d.add_object(dictionary! { "Type" => "Catalog", "Pages" => kok });
    d.trailer.set("Root", katalog);
    d.save(cikti).map(|_| ()).map_err(|e| format!("{} yazılamadı: {e}", ad(cikti)))
}

pub fn sayfa_sayisi(yol: &Path) -> Sonuc<u32> {
    Ok(yukle(yol)?.get_pages().len() as u32)
}

pub fn sayfa_cikar(yol: &Path, aralik: &str, cikti: &Path) -> Sonuc<usize> {
    let d = yukle(yol)?;
    let sayfalar = aralik_coz(aralik, d.get_pages().len() as u32)?;
    sayfalari_yaz(&d, &sayfalar, cikti)?;
    Ok(sayfalar.len())
}

/// Her `adim` sayfayı ayrı bir dosyaya yazar; `cikti_adi(i)` i. parçanın yolunu verir.
pub fn bol(yol: &Path, adim: u32, cikti_adi: impl Fn(u32, u32, u32) -> PathBuf) -> Sonuc<Vec<PathBuf>> {
    let d = yukle(yol)?;
    let toplam = d.get_pages().len() as u32;
    let adim = adim.max(1);
    let mut yazilan = Vec::new();
    let mut bas = 1;
    while bas <= toplam {
        let son = (bas + adim - 1).min(toplam);
        let hedef = cikti_adi(bas, son, toplam);
        sayfalari_yaz(&d, &(bas..=son).collect::<Vec<_>>(), &hedef)?;
        yazilan.push(hedef);
        bas = son + 1;
    }
    Ok(yazilan)
}

pub fn dondur(yol: &Path, derece: i64, aralik: &str, cikti: &Path) -> Sonuc<usize> {
    let mut d = yukle(yol)?;
    let tum = d.get_pages();
    let secili = if aralik.trim().is_empty() {
        tum.keys().copied().collect()
    } else {
        aralik_coz(aralik, tum.len() as u32)?
    };
    for n in &secili {
        let id = tum[n];
        let simdiki = d
            .get_dictionary(id)
            .ok()
            .and_then(|s| s.get(b"Rotate").and_then(Object::as_i64).ok())
            .or_else(|| miras(&d, id).get(b"Rotate").and_then(Object::as_i64).ok())
            .unwrap_or(0);
        if let Ok(s) = d.get_dictionary_mut(id) {
            s.set("Rotate", (simdiki + derece).rem_euclid(360));
        }
    }
    kaydet(d, cikti)?;
    Ok(secili.len())
}

/// Görselleri sırayla birer sayfa yapar. JPEG'ler yeniden sıkıştırılmadan gömülür.
pub fn gorsellerden(yollar: &[PathBuf], cikti: &Path) -> Sonuc<usize> {
    use image::ImageDecoder;
    let mut sayfalar = Vec::new();

    for yol in yollar {
        let okuyucu = image::ImageReader::open(yol)
            .and_then(|r| r.with_guessed_format())
            .map_err(|e| format!("{} okunamadı: {e}", ad(yol)))?;
        let jpeg = okuyucu.format() == Some(image::ImageFormat::Jpeg);
        let mut cozucu = okuyucu.into_decoder().map_err(|e| format!("{} okunamadı: {e}", ad(yol)))?;
        let yon = cozucu.orientation().unwrap_or(image::metadata::Orientation::NoTransforms);
        let renk = cozucu.color_type();
        let (gen, yuk) = cozucu.dimensions();

        let (veri, w, h, uzay) = if jpeg
            && yon == image::metadata::Orientation::NoTransforms
            && matches!(renk, image::ColorType::Rgb8 | image::ColorType::L8)
        {
            let veri = std::fs::read(yol).map_err(hata)?;
            (veri, gen, yuk, if renk == image::ColorType::L8 { "DeviceGray" } else { "DeviceRGB" })
        } else {
            let mut img = image::DynamicImage::from_decoder(cozucu).map_err(|e| format!("{} çözülemedi: {e}", ad(yol)))?;
            img.apply_orientation(yon);
            // Saydam alanları beyaza oturt.
            let rgba = img.to_rgba8();
            let mut rgb = image::RgbImage::new(rgba.width(), rgba.height());
            for (p, q) in rgba.pixels().zip(rgb.pixels_mut()) {
                let a = p[3] as u32;
                for k in 0..3 {
                    q[k] = ((p[k] as u32 * a + 255 * (255 - a)) / 255) as u8;
                }
            }
            let mut veri = Vec::new();
            image::codecs::jpeg::JpegEncoder::new_with_quality(&mut veri, 92)
                .encode_image(&rgb)
                .map_err(hata)?;
            (veri, rgb.width(), rgb.height(), "DeviceRGB")
        };
        sayfalar.push(JpegSayfa { veri, gen: w, yuk: h, gri: uzay == "DeviceGray" });
    }
    jpeglerden(sayfalar, cikti)
}

/// Bir sayfaya gömülecek hazır JPEG.
pub struct JpegSayfa {
    pub veri: Vec<u8>,
    pub gen: u32,
    pub yuk: u32,
    pub gri: bool,
}

/// JPEG'leri yeniden sıkıştırmadan birer sayfa olarak gömer.
pub fn jpeglerden(sayfalar: Vec<JpegSayfa>, cikti: &Path) -> Sonuc<usize> {
    let mut d = Document::with_version("1.5");
    let kok = d.new_object_id();
    let mut cocuklar = Vec::new();

    for JpegSayfa { veri, gen: w, yuk: h, gri } in sayfalar {
        let uzay = if gri { "DeviceGray" } else { "DeviceRGB" };
        let mut gorsel = Stream::new(
            dictionary! {
                "Type" => "XObject", "Subtype" => "Image",
                "Width" => w as i64, "Height" => h as i64,
                "ColorSpace" => uzay, "BitsPerComponent" => 8, "Filter" => "DCTDecode",
            },
            veri,
        );
        gorsel.allows_compression = false;
        let gorsel_id = d.add_object(gorsel);

        // A4 genişliği (dikey) ya da A4 yüksekliği (yatay) kadar geniş, görsel oranında sayfa.
        let gen_pt = if h >= w { 595.0 } else { 842.0 };
        let yuk_pt = gen_pt * h as f32 / w as f32;
        let icerik = d.add_object(Stream::new(dictionary! {}, format!("q {gen_pt} 0 0 {yuk_pt} 0 0 cm /G0 Do Q").into_bytes()));
        let sayfa = d.add_object(dictionary! {
            "Type" => "Page", "Parent" => kok,
            "MediaBox" => vec![0.into(), 0.into(), gen_pt.into(), yuk_pt.into()],
            "Contents" => icerik,
            "Resources" => dictionary! { "XObject" => dictionary! { "G0" => gorsel_id } },
        });
        cocuklar.push(Object::Reference(sayfa));
    }

    let sayi = cocuklar.len();
    d.objects.insert(kok, Object::Dictionary(dictionary! { "Type" => "Pages", "Kids" => cocuklar, "Count" => sayi as i64 }));
    let katalog = d.add_object(dictionary! { "Type" => "Catalog", "Pages" => kok });
    d.trailer.set("Root", katalog);
    kaydet(d, cikti)?;
    Ok(sayi)
}
