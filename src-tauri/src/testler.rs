//! Gerçek dosyalarla uçtan uca denemeler. CHAMELEON_TEST klasöründe en az iki PDF ve bir görsel olmalı:
//! `CHAMELEON_TEST=<klasör> cargo test -- --nocapture --test-threads=1`

use crate::{duzen, gorsel, model, pdf, tarama};
use std::path::PathBuf;

fn klasor() -> Option<PathBuf> {
    std::env::var_os("CHAMELEON_TEST").map(PathBuf::from)
}

fn dosyalar(uzantilar: &[&str]) -> Vec<PathBuf> {
    let Some(k) = klasor() else { return vec![] };
    let mut v: Vec<PathBuf> = std::fs::read_dir(k)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|e| uzantilar.contains(&e.to_string_lossy().to_lowercase().as_str())))
        .collect();
    v.sort();
    v
}

fn cikti(ad: &str) -> PathBuf {
    let k = klasor().unwrap().join("cikti");
    std::fs::create_dir_all(&k).unwrap();
    k.join(ad)
}

#[test]
fn aralik() {
    assert_eq!(pdf::aralik_coz("1-3, 5, 8-", 9).unwrap(), vec![1, 2, 3, 5, 8, 9]);
    assert_eq!(pdf::aralik_coz("4-2", 9).unwrap(), vec![4, 3, 2]);
    assert!(pdf::aralik_coz("12", 9).is_err());
    assert!(pdf::aralik_coz("", 9).is_err());
}

#[test]
fn birlestir_bol_cikar_dondur() {
    let pdfler = dosyalar(&["pdf"]);
    if pdfler.len() < 2 {
        eprintln!("atlandı: CHAMELEON_TEST'te iki PDF yok");
        return;
    }
    let (a, b) = (pdf::sayfa_sayisi(&pdfler[0]).unwrap(), pdf::sayfa_sayisi(&pdfler[1]).unwrap());

    let t = std::time::Instant::now();
    let hedef = cikti("birlesik.pdf");
    let n = pdf::birlestir(&pdfler[..2], &hedef).unwrap();
    eprintln!("birleştir: {a}+{b} → {n} sayfa, {:?}", t.elapsed());
    assert_eq!(n as u32, a + b);
    assert_eq!(pdf::sayfa_sayisi(&hedef).unwrap(), a + b);

    let t = std::time::Instant::now();
    let parcalar = pdf::bol(&hedef, 2, |bas, son, _| cikti(&format!("parca-{bas}-{son}.pdf"))).unwrap();
    eprintln!("böl (2'şer): {} parça, {:?}", parcalar.len(), t.elapsed());
    assert_eq!(parcalar.len() as u32, (a + b).div_ceil(2));
    assert_eq!(pdf::sayfa_sayisi(&parcalar[0]).unwrap(), 2.min(a + b));
    // Parça bağımsız bir PDF olarak Windows'un motorunda da açılabilmeli.
    let cizilen = gorsel::pdf_gorsel(&parcalar[0], &cikti("parca-gorsel"), 72, true, |_, _| {}).unwrap();
    assert_eq!(cizilen.len() as u32, 2.min(a + b));
    let boyut: u64 = parcalar.iter().map(|p| std::fs::metadata(p).unwrap().len()).sum();
    eprintln!("parçaların toplamı {} KB (kaynak {} KB)", boyut / 1024, std::fs::metadata(&hedef).unwrap().len() / 1024);

    let secim = format!("{}, 1", a + b);
    let t = std::time::Instant::now();
    let n = pdf::sayfa_cikar(&hedef, &secim, &cikti("cikarilan.pdf")).unwrap();
    eprintln!("sayfa çıkar ({secim}): {:?}", t.elapsed());
    assert_eq!(n, 2);
    assert_eq!(pdf::sayfa_sayisi(&cikti("cikarilan.pdf")).unwrap(), 2);

    let n = pdf::dondur(&pdfler[0], 90, "", &cikti("donmus.pdf")).unwrap();
    assert_eq!(n as u32, a);
    let d = pdf::yukle(&cikti("donmus.pdf")).unwrap();
    for id in d.get_pages().values() {
        let r = d.get_dictionary(*id).unwrap().get(b"Rotate").unwrap().as_i64().unwrap();
        assert_eq!(r.rem_euclid(90), 0);
    }
}

#[test]
fn gorselden_pdf() {
    let gorseller = dosyalar(&["jpg", "jpeg", "png"]);
    if gorseller.is_empty() {
        eprintln!("atlandı: görsel yok");
        return;
    }
    let t = std::time::Instant::now();
    let n = pdf::gorsellerden(&gorseller, &cikti("gorseller.pdf")).unwrap();
    eprintln!("görselden PDF: {n} sayfa, {:?}", t.elapsed());
    assert_eq!(n, gorseller.len());
    assert_eq!(pdf::sayfa_sayisi(&cikti("gorseller.pdf")).unwrap() as usize, gorseller.len());
}

#[test]
fn pdf_gorsele() {
    let pdfler = dosyalar(&["pdf"]);
    let Some(p) = pdfler.first() else { return };
    let t = std::time::Instant::now();
    let yazilan = gorsel::pdf_gorsel(p, &cikti("gorseller"), 150, false, |_, _| {}).unwrap();
    eprintln!("PDF → PNG (150 dpi): {} sayfa, {:?}", yazilan.len(), t.elapsed());
    assert_eq!(yazilan.len() as u32, pdf::sayfa_sayisi(p).unwrap());
    let ilk = image::open(&yazilan[0]).unwrap();
    eprintln!("ilk sayfa {}x{}", ilk.width(), ilk.height());
    assert!(ilk.width() > 500);
}

/// CHAMELEON_TARAMA klasöründeki fotoğraflarda bulunan köşeleri gercek.json ile karşılaştırır.
#[test]
fn belge_tespiti() {
    let Some(k) = std::env::var_os("CHAMELEON_TARAMA").map(PathBuf::from) else { return };
    let gercek: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(k.join("gercek.json")).unwrap()).unwrap();
    let cikti = k.join("duzeltilmis");
    std::fs::create_dir_all(&cikti).unwrap();
    let mut kotu = 0;
    let mut bulunanlar = serde_json::Map::new();
    for (ad, deger) in &gercek {
        let img = tarama::ac(&k.join(ad)).unwrap();
        let t = std::time::Instant::now();
        let (bulunan, yol) = tarama::koseleri_bul(&img);
        let sure = t.elapsed();
        bulunanlar.insert(ad.clone(), serde_json::json!(bulunan));
        let (w, h) = (img.width() as f32, img.height() as f32);
        let hata = (0..4)
            .map(|i| {
                let g = &deger[i];
                let (gx, gy) = (g[0].as_f64().unwrap() as f32 * w, g[1].as_f64().unwrap() as f32 * h);
                ((bulunan[i][0] * w - gx).powi(2) + (bulunan[i][1] * h - gy).powi(2)).sqrt()
            })
            .fold(0.0f32, f32::max);
        // Köşe hatası, görsel köşegeninin %1,5'inden büyükse başarısız say.
        let esik = (w * w + h * h).sqrt() * 0.015;
        if hata > esik {
            kotu += 1;
        }
        eprintln!("{ad}: yol={yol} en büyük köşe hatası {hata:.0} px (eşik {esik:.0}) {:?}", sure);
        let t = std::time::Instant::now();
        let d = tarama::duzelt(&img, &bulunan, 0, tarama::Filtre::Tarama, 2000);
        eprintln!("   düzelt+filtre {}x{} {:?}", d.width(), d.height(), t.elapsed());
        d.save(cikti.join(ad)).unwrap();
    }
    std::fs::write(k.join("bulunan.json"), serde_json::Value::Object(bulunanlar).to_string()).unwrap();

    // Uçtan uca: üç fotoğraf → düzelt → tek PDF → Windows motorunda aç.
    let t = std::time::Instant::now();
    let sayfalar: Vec<pdf::JpegSayfa> = gercek
        .keys()
        .take(3)
        .map(|ad| {
            let img = tarama::ac(&k.join(ad)).unwrap();
            let (kose, _) = tarama::koseleri_bul(&img);
            let d = tarama::duzelt(&img, &kose, 0, tarama::Filtre::Tarama, 2400);
            pdf::JpegSayfa { veri: tarama::jpeg(&d, 80), gen: d.width(), yuk: d.height(), gri: true }
        })
        .collect();
    let hedef = cikti.join("tarama.pdf");
    assert_eq!(pdf::jpeglerden(sayfalar, &hedef).unwrap(), 3);
    eprintln!("3 fotoğraf → PDF {:?}, {} KB", t.elapsed(), std::fs::metadata(&hedef).unwrap().len() / 1024);
    let cizilen = gorsel::pdf_gorsel(&hedef, &cikti.join("tarama-kontrol"), 72, true, |_, _| {}).unwrap();
    assert_eq!(cizilen.len(), 3);
    eprintln!("başarısız: {kotu}/{}", gercek.len());
}

/// Bir PDF sayfasını Windows motoruyla çizip verilen bölgedeki koyu piksel oranını döner.
fn koyu_oran(yol: &std::path::Path, bolge: (f32, f32, f32, f32)) -> f32 {
    let k = std::env::temp_dir().join(format!("chm-kontrol-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&k);
    let s = gorsel::pdf_gorsel(yol, &k, 72, false, |_, _| {}).unwrap();
    let img = image::open(&s[0]).unwrap().to_luma8();
    let (w, h) = (img.width() as f32, img.height() as f32);
    let (x0, y0, x1, y1) = ((bolge.0 * w) as u32, (bolge.1 * h) as u32, (bolge.2 * w) as u32, (bolge.3 * h) as u32);
    let mut koyu = 0;
    let mut hepsi = 0;
    for y in y0..y1 {
        for x in x0..x1 {
            hepsi += 1;
            if img.get_pixel(x, y)[0] < 128 {
                koyu += 1;
            }
        }
    }
    koyu as f32 / hepsi.max(1) as f32
}

#[test]
fn duzen_islemleri() {
    let pdfler = dosyalar(&["pdf"]);
    let Some(kaynak) = pdfler.first() else { return };
    let n = pdf::sayfa_sayisi(kaynak).unwrap() as usize;

    // Sayfa numarası: alt ortada yazı belirmeli.
    let alt = (0.4, 0.955, 0.6, 1.0);
    let once = koyu_oran(kaynak, alt);
    let t = std::time::Instant::now();
    assert_eq!(duzen::sayfa_numarasi(kaynak, &cikti("numarali.pdf"), "alt-orta", "Sayfa {n} / {N} — ğüşıöç", 1, 11.0).unwrap(), n);
    let sonra = koyu_oran(&cikti("numarali.pdf"), alt);
    eprintln!("sayfa numarası {:?}: alt-orta koyu oran {once:.4} → {sonra:.4}, boyut {} KB", t.elapsed(), std::fs::metadata(cikti("numarali.pdf")).unwrap().len() / 1024);
    assert!(sonra > once + 0.003);

    // Filigran.
    duzen::filigran(kaynak, &cikti("filigranli.pdf"), "GİZLİ ÖRNEK", 0.25).unwrap();
    assert_eq!(pdf::sayfa_sayisi(&cikti("filigranli.pdf")).unwrap() as usize, n);

    // Metin üst katmanı: sol üstte bir alanı beyaza örtüp yeni yazı yaz.
    let d = pdf::yukle(kaynak).unwrap();
    let ([x0, _, _, y1], _) = (([0.0f32, 0.0, 0.0, 842.0]), 0);
    let _ = (x0, y1, d);
    let degisiklik = vec![duzen::MetinDegisiklik {
        sayfa: 0, ort: Some([40.0, 700.0, 300.0, 60.0]), zemin: [1.0, 1.0, 1.0],
        x: 50.0, y: 720.0, boyut: 24.0, renk: [0.8, 0.1, 0.1], kalin: true, metin: "Değiştirildi: şğüİı".into(),
    }];
    duzen::metinleri_yaz(kaynak, &cikti("metinli.pdf"), &degisiklik).unwrap();
    let metin_d = pdf::yukle(&cikti("metinli.pdf")).unwrap();
    let sayfa = *metin_d.get_pages().values().next().unwrap();
    let metin = metin_d.extract_text(&[1]).unwrap_or_default();
    eprintln!("metin katmanı: çıkarılan metinde yeni yazı var mı: {}", metin.contains("Değiştirildi"));
    let _ = sayfa;

    // Sıkıştır.
    let t = std::time::Instant::now();
    let (a, b, g) = duzen::sikistir(&pdfler[pdfler.len() - 1], &cikti("sikistirilmis.pdf"), 2).unwrap();
    eprintln!("sıkıştır {:?}: {} KB → {} KB, {g} görsel", t.elapsed(), a / 1024, b / 1024);
    assert_eq!(pdf::sayfa_sayisi(&cikti("sikistirilmis.pdf")).unwrap(), pdf::sayfa_sayisi(&pdfler[pdfler.len() - 1]).unwrap());
    gorsel::pdf_gorsel(&cikti("sikistirilmis.pdf"), &cikti("sikistirilmis-kontrol"), 50, true, |_, _| {}).unwrap();

    // Parola: koy, parolasız açılmasın, parolayla açılsın, kaldır.
    duzen::parola_koy(kaynak, &cikti("parolali.pdf"), "çilek123").unwrap();
    assert!(pdf::yukle(&cikti("parolali.pdf")).is_err());
    assert!(gorsel::pdf_gorsel(&cikti("parolali.pdf"), &cikti("parolali-kontrol"), 50, true, |_, _| {}).is_err());
    assert!(duzen::parola_kaldir(&cikti("parolali.pdf"), &cikti("parolasiz.pdf"), "yanlis").is_err());
    duzen::parola_kaldir(&cikti("parolali.pdf"), &cikti("parolasiz.pdf"), "çilek123").unwrap();
    assert_eq!(pdf::sayfa_sayisi(&cikti("parolasiz.pdf")).unwrap() as usize, n);
    eprintln!("parola: koy/kaldır tamam");
}

/// duzen_islemleri çıktılarının ilk sayfalarını göz kontrolü için PNG'ye çizer.
#[test]
fn duzen_gorsel_kontrol() {
    let Some(_) = klasor() else { return };
    for ad in ["numarali", "filigranli", "metinli"] {
        let p = cikti(&format!("{ad}.pdf"));
        if p.exists() {
            let _ = std::fs::remove_dir_all(cikti(&format!("k_{ad}")));
            gorsel::pdf_gorsel(&p, &cikti(&format!("k_{ad}")), 60, false, |_, _| {}).unwrap();
        }
    }
}

/// Arayüzün metin düzenleyicisinden gelen gerçek bir değişiklik listesini uygular ve ilk sayfayı çizer.
#[test]
fn arayuz_metin_degisikligi() {
    let Some(kaynak) = dosyalar(&["pdf"]).into_iter().next() else { return };
    let json = r#"[{"sayfa":0,"ort":[83.5,718,184.8,10.6],"zemin":[0.99,0.99,0.99],"x":84,"y":720.1,"boyut":9,"renk":[0,0,0],"kalin":true,"metin":"YENİ BAŞLIK şğüİı"},
                   {"sayfa":0,"ort":null,"zemin":[1,1,1],"x":297.6,"y":49.3,"boyut":12,"renk":[0,0,0],"kalin":false,"metin":"Not: kontrol edildi"}]"#;
    let degisiklik: Vec<duzen::MetinDegisiklik> = serde_json::from_str(json).unwrap();
    let hedef = cikti("arayuz-metin.pdf");
    duzen::metinleri_yaz(&kaynak, &hedef, &degisiklik).unwrap();
    let _ = std::fs::remove_dir_all(cikti("k_arayuz"));
    gorsel::pdf_gorsel(&hedef, &cikti("k_arayuz"), 110, false, |_, _| {}).unwrap();
}

/// Klasik, model ve model+hassaslaştırma yollarını CHAMELEON_TARAMA setinde karşılaştırır.
#[test]
fn tespit_karsilastir() {
    let Some(k) = std::env::var_os("CHAMELEON_TARAMA").map(PathBuf::from) else { return };
    assert!(model::var_mi(), "model dosyası bulunamadı");
    let gercek: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(&std::fs::read_to_string(k.join("gercek.json")).unwrap()).unwrap();
    let mut basari = [0; 3];
    let mut sure = [std::time::Duration::ZERO; 3];
    for (ad, deger) in &gercek {
        let img = tarama::ac(&k.join(ad)).unwrap();
        let (w, h) = (img.width() as f32, img.height() as f32);
        let esik = (w * w + h * h).sqrt() * 0.015;
        let hata = |b: &tarama::Kose| {
            (0..4)
                .map(|i| {
                    let (gx, gy) = (deger[i][0].as_f64().unwrap() as f32 * w, deger[i][1].as_f64().unwrap() as f32 * h);
                    ((b[i][0] * w - gx).powi(2) + (b[i][1] * h - gy).powi(2)).sqrt()
                })
                .fold(0.0f32, f32::max)
        };
        let t = std::time::Instant::now();
        let klasik = tarama::klasik_koseler(&img).0;
        sure[0] += t.elapsed();
        let t = std::time::Instant::now();
        let m = model::koseler(&img).unwrap();
        sure[1] += t.elapsed();
        let t = std::time::Instant::now();
        let mh = tarama::hassaslastir(&img, m);
        sure[2] += t.elapsed();
        let hatalar = [hata(&klasik), hata(&m), hata(&mh)];
        for (i, e) in hatalar.iter().enumerate() {
            if *e <= esik {
                basari[i] += 1;
            }
        }
        eprintln!("{ad}: klasik {:5.0}  model {:5.0}  model+hassas {:5.0}  (eşik {esik:.0})", hatalar[0], hatalar[1], hatalar[2]);
    }
    let n = gercek.len() as u32;
    eprintln!("başarılı: klasik {}/{n}, model {}/{n}, model+hassas {}/{n}", basari[0], basari[1], basari[2]);
    eprintln!("ort. süre: klasik {:?}, model {:?}, hassaslaştırma (ek) {:?}", sure[0] / n, sure[1] / n, sure[2] / n);
}
