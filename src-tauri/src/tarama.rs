//! Fotoğraftaki belgeyi bulur, perspektifini düzeltir ve taranmış gibi temizler.
//!
//! Köşe bulma iki yolla denenir, model gerektirmez:
//! 1. Kenar: Canny + genişletme → en büyük dışbükey dörtgen kontur.
//! 2. Parlaklık: Otsu eşiği → en büyük açık renkli bölge → uç noktalar.
//! Kenar yolu güvenilir bir dörtgen bulursa o, bulamazsa parlaklık yolu kullanılır.

use image::{imageops, DynamicImage, GrayImage, ImageDecoder, Luma, RgbImage};
use imageproc::{
    contours::find_contours,
    distance_transform::Norm,
    edges::canny,
    geometric_transformations::{warp_into, Border, Interpolation, Projection},
    geometry::{approximate_polygon_dp, arc_length, convex_hull},
    point::Point,
};
use std::path::Path;

/// Sol üst, sağ üst, sağ alt, sol alt; 0..1 aralığında (yönü düzeltilmiş görsele göre).
pub type Kose = [[f32; 2]; 4];

/// EXIF yönünü uygulayarak açar (telefon fotoğrafları çoğu zaman yan kaydedilir).
pub fn ac(yol: &Path) -> Result<DynamicImage, String> {
    let ad = yol.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let mut cozucu = image::ImageReader::open(yol)
        .and_then(|r| r.with_guessed_format())
        .map_err(|e| format!("{ad} açılamadı: {e}"))?
        .into_decoder()
        .map_err(|e| format!("{ad} okunamadı: {e}"))?;
    let yon = cozucu.orientation().unwrap_or(image::metadata::Orientation::NoTransforms);
    let mut img = DynamicImage::from_decoder(cozucu).map_err(|e| format!("{ad} çözülemedi: {e}"))?;
    img.apply_orientation(yon);
    Ok(img)
}

fn alan(q: &[(f32, f32); 4]) -> f32 {
    let mut s = 0.0;
    for i in 0..4 {
        let (a, b) = (q[i], q[(i + 1) % 4]);
        s += a.0 * b.1 - b.0 * a.1;
    }
    s.abs() / 2.0
}

/// Nokta kümesinin dört uç köşesi: x+y en küçük/büyük, x−y en büyük/küçük.
fn uc_koseler(noktalar: &[(f32, f32)]) -> Option<[(f32, f32); 4]> {
    let by = |f: fn(&(f32, f32)) -> f32, en_buyuk: bool| {
        noktalar
            .iter()
            .copied()
            .max_by(|a, b| {
                let o = f(a).total_cmp(&f(b));
                if en_buyuk { o } else { o.reverse() }
            })
    };
    Some([
        by(|p| p.0 + p.1, false)?,
        by(|p| p.0 - p.1, true)?,
        by(|p| p.0 + p.1, true)?,
        by(|p| p.0 - p.1, false)?,
    ])
}

fn disbukey_mi(q: &[(f32, f32); 4]) -> bool {
    let mut isaret = 0.0f32;
    for i in 0..4 {
        let (a, b, c) = (q[i], q[(i + 1) % 4], q[(i + 2) % 4]);
        let z = (b.0 - a.0) * (c.1 - b.1) - (b.1 - a.1) * (c.0 - b.0);
        if z.abs() < 1e-6 {
            return false;
        }
        if isaret == 0.0 {
            isaret = z.signum();
        } else if z.signum() != isaret {
            return false;
        }
    }
    true
}

/// Kenar yolu: en büyük, yeterince geniş ve dışbükey dörtgen kontur.
fn kenarla(gri: &GrayImage) -> Option<[(f32, f32); 4]> {
    let toplam = (gri.width() * gri.height()) as f32;
    let mut en_iyi: Option<([(f32, f32); 4], f32)> = None;
    for (alt, ust) in [(20.0, 60.0), (40.0, 120.0), (10.0, 35.0)] {
        let kenar = imageproc::morphology::dilate(&canny(gri, alt, ust), Norm::LInf, 2);
        for kontur in find_contours::<i32>(&kenar) {
            if kontur.points.len() < 20 {
                continue;
            }
            let govde = convex_hull(kontur.points.clone());
            let cevre = arc_length(&govde, true);
            for oran in [0.02, 0.035, 0.05] {
                let yaklasik = approximate_polygon_dp(&govde, cevre * oran, true);
                if yaklasik.len() != 4 {
                    continue;
                }
                let n: Vec<(f32, f32)> = yaklasik.iter().map(|p| (p.x as f32, p.y as f32)).collect();
                let q = uc_koseler(&n)?;
                let a = alan(&q);
                if disbukey_mi(&q) && a > toplam * 0.15 && en_iyi.is_none_or(|(_, b)| a > b) {
                    en_iyi = Some((q, a));
                }
                break;
            }
        }
        if en_iyi.is_some() {
            break;
        }
    }
    en_iyi.map(|(q, _)| q)
}

/// Parlaklık yolu: kâğıt genelde zeminden açıktır.
fn parlaklikla(gri: &GrayImage) -> Option<[(f32, f32); 4]> {
    let esik = imageproc::contrast::otsu_level(gri);
    let ikili = imageproc::contrast::threshold(gri, esik, imageproc::contrast::ThresholdType::Binary);
    let etiket = imageproc::region_labelling::connected_components(&ikili, imageproc::region_labelling::Connectivity::Eight, Luma([0u8]));
    let mut sayac = std::collections::HashMap::<u32, u32>::new();
    for p in etiket.pixels() {
        if p[0] != 0 {
            *sayac.entry(p[0]).or_default() += 1;
        }
    }
    let (&en_buyuk, &adet) = sayac.iter().max_by_key(|(_, &n)| n)?;
    if (adet as f32) < (gri.width() * gri.height()) as f32 * 0.15 {
        return None;
    }
    let noktalar: Vec<Point<i32>> = etiket
        .enumerate_pixels()
        .filter(|(_, _, p)| p[0] == en_buyuk)
        .map(|(x, y, _)| Point::new(x as i32, y as i32))
        .collect();
    let govde: Vec<(f32, f32)> = convex_hull(noktalar).iter().map(|p| (p.x as f32, p.y as f32)).collect();
    uc_koseler(&govde)
}

/// Belgenin köşelerini bulur: önce DocAligner modeli, o yoksa ya da köşe çıkmazsa klasik yöntem.
pub fn koseleri_bul(img: &DynamicImage) -> (Kose, &'static str) {
    if let Some(k) = crate::model::koseler(img) {
        return (k, "model");
    }
    klasik_koseler(img)
}

/// Model gerektirmeyen yol. Bulamazsa kenarlardan %4 içeride bir dikdörtgen döner.
pub fn klasik_koseler(img: &DynamicImage) -> (Kose, &'static str) {
    let kucuk = img.resize(560, 560, imageops::FilterType::Triangle).to_luma8();
    let gri = imageproc::filter::gaussian_blur_f32(&kucuk, 1.6);
    let (w, h) = (gri.width() as f32, gri.height() as f32);
    let (q, yol) = match kenarla(&gri) {
        Some(q) => (q, "kenar"),
        None => match parlaklikla(&gri) {
            Some(q) => (q, "parlaklık"),
            None => {
                let q = [(0.04 * w, 0.04 * h), (0.96 * w, 0.04 * h), (0.96 * w, 0.96 * h), (0.04 * w, 0.96 * h)];
                return (q.map(|(x, y)| [x / w, y / h]), "yok");
            }
        },
    };
    let kaba = q.map(|(x, y)| [(x / w).clamp(0.0, 1.0), (y / h).clamp(0.0, 1.0)]);
    (hassaslastir(img, kaba), yol)
}

/// Kaba köşeleri yüksek çözünürlükte iyileştirir: her kenar boyunca, kenara dik doğrultuda en keskin
/// parlaklık geçişini arar, bu noktalara doğru oturtur ve komşu doğruların kesişimini köşe alır.
pub(crate) fn hassaslastir(img: &DynamicImage, kaba: Kose) -> Kose {
    let gri = imageproc::filter::gaussian_blur_f32(&img.resize(1600, 1600, imageops::FilterType::Triangle).to_luma8(), 1.2);
    let (w, h) = (gri.width() as f32, gri.height() as f32);
    let deger = |x: f32, y: f32| -> f32 {
        let (xi, yi) = (x.round(), y.round());
        if xi < 0.0 || yi < 0.0 || xi >= w || yi >= h {
            return f32::NAN;
        }
        gri.get_pixel(xi as u32, yi as u32)[0] as f32
    };
    let k = kaba.map(|[x, y]| (x * w, y * h));
    let arama = (w * w + h * h).sqrt() * 0.025;

    // Her kenar için doğru: nokta + birim yön.
    let mut dogrular = Vec::with_capacity(4);
    for i in 0..4 {
        let (a, b) = (k[i], k[(i + 1) % 4]);
        let (dx, dy) = (b.0 - a.0, b.1 - a.1);
        let boy = (dx * dx + dy * dy).sqrt().max(1.0);
        let (ux, uy) = (dx / boy, dy / boy);
        let (nx, ny) = (-uy, ux);
        let mut noktalar = Vec::new();
        for s in 0..40 {
            let t = 0.08 + 0.84 * s as f32 / 39.0;
            let (px, py) = (a.0 + dx * t, a.1 + dy * t);
            let mut en_iyi = (0.0f32, 0.0f32);
            let mut d = -arama;
            while d <= arama {
                let g = deger(px + nx * (d + 1.5), py + ny * (d + 1.5)) - deger(px + nx * (d - 1.5), py + ny * (d - 1.5));
                if g.is_finite() && g.abs() > en_iyi.1 {
                    en_iyi = (d, g.abs());
                }
                d += 1.0;
            }
            if en_iyi.1 > 12.0 {
                noktalar.push((px + nx * en_iyi.0, py + ny * en_iyi.0));
            }
        }
        dogrular.push(dogru_oturt(&noktalar).unwrap_or(((a.0 + b.0) / 2.0, (a.1 + b.1) / 2.0, ux, uy)));
    }

    let mut sonuc = kaba;
    for i in 0..4 {
        // i. köşe, (i-1). ve i. kenarların kesişimi.
        let (p, q) = (dogrular[(i + 3) % 4], dogrular[i]);
        if let Some((x, y)) = kesisim(p, q) {
            let (ox, oy) = k[i];
            if ((x - ox).powi(2) + (y - oy).powi(2)).sqrt() < arama * 1.5 {
                sonuc[i] = [(x / w).clamp(0.0, 1.0), (y / h).clamp(0.0, 1.0)];
            }
        }
    }
    sonuc
}

/// En küçük kareler doğrusu (toplam en küçük kareler); 3 px'ten uzak noktaları atıp bir kez daha oturtur.
fn dogru_oturt(noktalar: &[(f32, f32)]) -> Option<(f32, f32, f32, f32)> {
    let oturt = |n: &[(f32, f32)]| -> Option<(f32, f32, f32, f32)> {
        if n.len() < 6 {
            return None;
        }
        let m = n.len() as f32;
        let (cx, cy) = n.iter().fold((0.0, 0.0), |s, p| (s.0 + p.0 / m, s.1 + p.1 / m));
        let (mut sxx, mut syy, mut sxy) = (0.0, 0.0, 0.0);
        for p in n {
            let (x, y) = (p.0 - cx, p.1 - cy);
            sxx += x * x;
            syy += y * y;
            sxy += x * y;
        }
        let aci = 0.5 * (2.0 * sxy).atan2(sxx - syy);
        Some((cx, cy, aci.cos(), aci.sin()))
    };
    let ilk = oturt(noktalar)?;
    let uzaklik = |p: &(f32, f32), l: (f32, f32, f32, f32)| ((p.0 - l.0) * -l.3 + (p.1 - l.1) * l.2).abs();
    let iyi: Vec<(f32, f32)> = noktalar.iter().copied().filter(|p| uzaklik(p, ilk) < 3.0).collect();
    oturt(&iyi).or(Some(ilk))
}

fn kesisim(p: (f32, f32, f32, f32), q: (f32, f32, f32, f32)) -> Option<(f32, f32)> {
    let det = p.2 * q.3 - p.3 * q.2;
    if det.abs() < 1e-4 {
        return None;
    }
    let t = ((q.0 - p.0) * q.3 - (q.1 - p.1) * q.2) / det;
    Some((p.0 + p.2 * t, p.1 + p.3 * t))
}

#[derive(Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Filtre {
    /// Gölgesi alınmış, kontrastı artırılmış gri: klasik "taranmış" görünüm.
    Tarama,
    /// Gölgesi alınmış renkli.
    Renkli,
    /// Yalnızca perspektif düzeltmesi.
    Orijinal,
}

/// Köşelere göre düzeltir, döndürür ve filtreyi uygular. `uzun_kenar` çıktının en uzun kenarı (piksel).
pub fn duzelt(img: &DynamicImage, kose: &Kose, donus: u32, filtre: Filtre, uzun_kenar: u32) -> DynamicImage {
    let (iw, ih) = (img.width() as f32, img.height() as f32);
    let k = kose.map(|[x, y]| (x * iw, y * ih));
    let uzunluk = |a: (f32, f32), b: (f32, f32)| ((a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)).sqrt();
    let mut gen = uzunluk(k[0], k[1]).max(uzunluk(k[3], k[2]));
    let mut yuk = uzunluk(k[0], k[3]).max(uzunluk(k[1], k[2]));
    // Kaynağın çözünürlüğünü aşma; çok büyükse uzun kenarı sınırla.
    let olcek = (uzun_kenar as f32 / gen.max(yuk)).min(1.0);
    gen = (gen * olcek).round().max(16.0);
    yuk = (yuk * olcek).round().max(16.0);

    let rgb = img.to_rgb8();
    let hedef = [(0.0, 0.0), (gen, 0.0), (gen, yuk), (0.0, yuk)];
    let mut cikti = RgbImage::new(gen as u32, yuk as u32);
    if let Some(p) = Projection::from_control_points(k, hedef) {
        warp_into(&rgb, p, Interpolation::Bilinear, Border::Replicate, &mut cikti);
    }
    // Köşe birkaç piksel taşarsa zemin kenarda siyah şerit bırakır; her kenardan %0,6 kırp.
    let (pw, ph) = ((gen * 0.006).round() as u32, (yuk * 0.006).round() as u32);
    let cikti = imageops::crop_imm(&cikti, pw, ph, cikti.width() - 2 * pw, cikti.height() - 2 * ph).to_image();
    let mut sonuc = match filtre {
        Filtre::Orijinal => DynamicImage::ImageRgb8(cikti),
        Filtre::Renkli => DynamicImage::ImageRgb8(golge_al_renkli(&cikti)),
        Filtre::Tarama => DynamicImage::ImageLuma8(golge_al_gri(&cikti)).unsharpen(0.8, 3),
    };
    sonuc = match donus % 360 {
        90 => sonuc.rotate90(),
        180 => sonuc.rotate180(),
        270 => sonuc.rotate270(),
        _ => sonuc,
    };
    sonuc
}

/// Kâğıdın aydınlanmasını tahmin eder: küçült, en açık değerleri yay (yazıyı sil), bulanıklaştır, büyüt.
fn zemin(gri: &GrayImage) -> GrayImage {
    let (w, h) = gri.dimensions();
    let kw = (w / 16).max(8);
    let kh = (h / 16).max(8);
    let kucuk = imageops::resize(gri, kw, kh, imageops::FilterType::Triangle);
    // Yazı koyu olduğundan genişletme (max) onu siler, kâğıdın parlaklığı kalır.
    let acik = imageproc::morphology::dilate(&kucuk, Norm::LInf, 2);
    let yumusak = imageproc::filter::gaussian_blur_f32(&acik, 2.0);
    imageops::resize(&yumusak, w, h, imageops::FilterType::Triangle)
}

fn golge_al_gri(rgb: &RgbImage) -> GrayImage {
    let gri = DynamicImage::ImageRgb8(rgb.clone()).to_luma8();
    let z = zemin(&gri);
    let mut c = GrayImage::new(gri.width(), gri.height());
    for ((p, b), o) in gri.pixels().zip(z.pixels()).zip(c.pixels_mut()) {
        let oran = p[0] as f32 / (b[0] as f32).max(24.0);
        // 0.9 üstü kâğıt kabul edilip beyaza, altı hafifçe koyulaştırılarak yayılır.
        let v = ((oran - 0.25) / (0.9 - 0.25)).clamp(0.0, 1.0);
        o[0] = (v.powf(1.1) * 255.0) as u8;
    }
    c
}

fn golge_al_renkli(rgb: &RgbImage) -> RgbImage {
    let gri = DynamicImage::ImageRgb8(rgb.clone()).to_luma8();
    let z = zemin(&gri);
    let mut c = RgbImage::new(rgb.width(), rgb.height());
    for ((p, b), o) in rgb.pixels().zip(z.pixels()).zip(c.pixels_mut()) {
        let bolen = (b[0] as f32).max(24.0) / 255.0;
        for k in 0..3 {
            let v = (p[k] as f32 / bolen / 255.0 - 0.06) / 0.9;
            o[k] = (v.clamp(0.0, 1.0) * 255.0) as u8;
        }
    }
    c
}

pub fn jpeg(img: &DynamicImage, kalite: u8) -> Vec<u8> {
    let mut veri = Vec::new();
    let mut k = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut veri, kalite);
    let _ = match img {
        DynamicImage::ImageLuma8(g) => k.encode_image(g),
        diger => k.encode_image(&diger.to_rgb8()),
    };
    veri
}
