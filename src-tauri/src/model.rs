//! DocAligner (DocsaidLab, Apache-2.0) ısı haritası modeliyle belge köşesi bulma.
//! Model dosyası exe'nin yanındaki `modeller/` klasöründe durur; yoksa tarama klasik yönteme düşer.

use image::{imageops, DynamicImage};
use ort::{session::Session, value::Tensor};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

const DOSYA: &str = "docaligner-fastvit_sa24.onnx";
const GIRDI: u32 = 256;
const HARITA: usize = 128;
/// infer.py'deki heatmap_threshold.
const ESIK: f32 = 0.3;

static OTURUM: OnceLock<Option<Mutex<Session>>> = OnceLock::new();

fn model_yolu() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let klasor = exe.parent()?;
    // Kurulu uygulama: exe'nin yanı. Geliştirme: target/<profil>/ → src-tauri/modeller.
    [klasor.join("modeller").join(DOSYA), klasor.join("../../modeller").join(DOSYA), klasor.join("../../../modeller").join(DOSYA)]
        .into_iter()
        .find(|p| p.exists())
}

fn oturum() -> Option<&'static Mutex<Session>> {
    OTURUM
        .get_or_init(|| {
            let yol = model_yolu()?;
            let ortam = ort::init().with_name("chameleon").build().ok()?;
            Session::builder(&ortam).ok()?.commit_from_file(yol).ok().map(Mutex::new)
        })
        .as_ref()
}

pub fn var_mi() -> bool {
    oturum().is_some()
}

/// Dört köşe (sol üst, sağ üst, sağ alt, sol alt; 0..1). Model yoksa ya da bir köşe bulunamazsa None.
pub fn koseler(img: &DynamicImage) -> Option<[[f32; 2]; 4]> {
    let kilit = oturum()?;
    // infer.py: en-boy korunmadan 256x256'ya, BGR, /255, NCHW.
    let k = imageops::resize(&img.to_rgb8(), GIRDI, GIRDI, imageops::FilterType::Triangle);
    let alan = (GIRDI * GIRDI) as usize;
    let mut veri = vec![0f32; 3 * alan];
    for (x, y, p) in k.enumerate_pixels() {
        let i = (y * GIRDI + x) as usize;
        for c in 0..3 {
            veri[c * alan + i] = p[2 - c] as f32 / 255.0;
        }
    }
    let girdi = Tensor::from_array(([1usize, 3, GIRDI as usize, GIRDI as usize], veri)).ok()?;
    let mut oturum = kilit.lock().ok()?;
    let cikti = oturum.run(ort::inputs!["img" => girdi]).ok()?;
    let (_, isi) = cikti["heatmap"].try_extract_tensor::<f32>().ok()?;

    let mut sonuc = [[0f32; 2]; 4];
    for (c, kose) in sonuc.iter_mut().enumerate() {
        *kose = merkez(&isi[c * HARITA * HARITA..(c + 1) * HARITA * HARITA])?;
    }
    Some(sonuc)
}

/// Eşiği aşan en büyük bağlı bölgenin (8 komşulu) değerle ağırlıklı merkezi, 0..1 aralığında.
fn merkez(harita: &[f32]) -> Option<[f32; 2]> {
    let n = HARITA;
    let mut etiket = vec![0u32; n * n];
    let mut en_iyi: Option<(usize, f32, f32, f32)> = None; // (alan, Σv·x, Σv·y, Σv)
    let mut sira = 0;
    let mut yigin = Vec::new();
    for bas in 0..n * n {
        if harita[bas] < ESIK || etiket[bas] != 0 {
            continue;
        }
        sira += 1;
        etiket[bas] = sira;
        yigin.push(bas);
        let (mut alan, mut sx, mut sy, mut sv) = (0usize, 0f32, 0f32, 0f32);
        while let Some(i) = yigin.pop() {
            let (x, y) = (i % n, i / n);
            let v = harita[i];
            alan += 1;
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
        if en_iyi.is_none_or(|e| alan > e.0) {
            en_iyi = Some((alan, sx, sy, sv));
        }
    }
    let (_, sx, sy, sv) = en_iyi?;
    Some([sx / sv / n as f32, sy / sv / n as f32])
}
