//! Windows yazı tipini (Arial) PDF'e alt küme olarak gömer: yalnızca kullanılan harfler girer,
//! Türkçe karakterler çalışır, dosya birkaç KB büyür.

use lopdf::{dictionary, Document, Object, ObjectId, Stream};
use std::collections::BTreeMap;
use subsetter::GlyphRemapper;

pub struct Yazi {
    pub id: ObjectId,
    /// karakter → (yeni glif kimliği, em başına genişlik)
    harita: BTreeMap<char, (u16, f32)>,
}

impl Yazi {
    /// Metni içerik akışında kullanılacak onaltılık dizgeye (<...>) ve em biriminde genişliğe çevirir.
    pub fn kodla(&self, metin: &str) -> (String, f32) {
        let mut bayt = Vec::with_capacity(metin.len() * 2);
        let mut gen = 0.0;
        for c in metin.chars() {
            let (g, w) = self.harita.get(&c).or_else(|| self.harita.get(&'?')).copied().unwrap_or((0, 0.5));
            bayt.extend_from_slice(&g.to_be_bytes());
            gen += w;
        }
        let hex: String = bayt.iter().map(|b| format!("{b:02X}")).collect();
        (format!("<{hex}>"), gen)
    }
}

fn dosya(kalin: bool) -> Result<Vec<u8>, String> {
    let klasor = std::env::var("WINDIR").unwrap_or_else(|_| r"C:\Windows".into()) + r"\Fonts\";
    let adaylar: &[&str] = if kalin { &["arialbd.ttf", "segoeuib.ttf", "arial.ttf"] } else { &["arial.ttf", "segoeui.ttf"] };
    adaylar
        .iter()
        .find_map(|a| std::fs::read(format!("{klasor}{a}")).ok())
        .ok_or_else(|| "Windows yazı tipi (Arial) bulunamadı".to_string())
}

/// `metinler`deki bütün karakterleri içeren bir Type0 yazı tipi ekler.
pub fn gom<'a>(d: &mut Document, kalin: bool, metinler: impl IntoIterator<Item = &'a str>) -> Result<Yazi, String> {
    let veri = dosya(kalin)?;
    let yuz = ttf_parser::Face::parse(&veri, 0).map_err(|e| format!("Yazı tipi okunamadı: {e}"))?;
    let em = yuz.units_per_em() as f32;

    let mut karakterler: Vec<char> = metinler.into_iter().flat_map(str::chars).collect();
    karakterler.push('?');
    karakterler.sort_unstable();
    karakterler.dedup();

    let mut yeniden = GlyphRemapper::new();
    let mut eski = BTreeMap::new();
    for &c in &karakterler {
        if let Some(g) = yuz.glyph_index(c) {
            yeniden.remap(g.0);
            eski.insert(c, g);
        }
    }
    let alt = subsetter::subset(&veri, 0, &yeniden).map_err(|e| format!("Yazı tipi küçültülemedi: {e:?}"))?;

    let mut harita = BTreeMap::new();
    let mut genislikler = BTreeMap::new();
    let mut unicode = BTreeMap::new();
    for (&c, &g) in &eski {
        let yeni = yeniden.get(g.0).unwrap_or(0);
        let w = yuz.glyph_hor_advance(g).unwrap_or(0) as f32 / em;
        harita.insert(c, (yeni, w));
        genislikler.insert(yeni, (w * 1000.0).round() as i64);
        unicode.entry(yeni).or_insert(c);
    }

    let olc = |v: i16| (v as f32 * 1000.0 / em).round() as i64;
    let kutu = yuz.global_bounding_box();
    let ad = if kalin { "CHMLNB+Arial-Bold" } else { "CHMLNA+ArialMT" };
    let mut akis = Stream::new(dictionary! { "Length1" => alt.len() as i64 }, alt);
    akis.compress().ok();
    let dosya_id = d.add_object(akis);
    let tanim = d.add_object(dictionary! {
        "Type" => "FontDescriptor", "FontName" => Object::Name(ad.into()),
        "Flags" => 32, "ItalicAngle" => 0,
        "FontBBox" => vec![olc(kutu.x_min).into(), olc(kutu.y_min).into(), olc(kutu.x_max).into(), olc(kutu.y_max).into()],
        "Ascent" => olc(yuz.ascender()), "Descent" => olc(yuz.descender()),
        "CapHeight" => olc(yuz.capital_height().unwrap_or(yuz.ascender())),
        "StemV" => if kalin { 120 } else { 80 },
        "FontFile2" => dosya_id,
    });

    let mut w = Vec::new();
    for (g, gen) in &genislikler {
        w.push(Object::Integer(*g as i64));
        w.push(Object::Array(vec![Object::Integer(*gen)]));
    }
    let cid = d.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "CIDFontType2", "BaseFont" => Object::Name(ad.into()),
        "CIDSystemInfo" => dictionary! { "Registry" => Object::string_literal("Adobe"), "Ordering" => Object::string_literal("Identity"), "Supplement" => 0 },
        "FontDescriptor" => tanim, "W" => w, "DW" => 500, "CIDToGIDMap" => "Identity",
    });

    // Metin seçilip kopyalanabilsin diye glif → Unicode eşlemesi.
    let mut cmap = String::from(
        "/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n/CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n/CMapName /Adobe-Identity-UCS def\n/CMapType 2 def\n1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n",
    );
    let girdiler: Vec<_> = unicode.iter().collect();
    for parca in girdiler.chunks(100) {
        cmap.push_str(&format!("{} beginbfchar\n", parca.len()));
        for (g, c) in parca {
            let mut u = [0u16; 2];
            let hex: String = c.encode_utf16(&mut u).iter().map(|x| format!("{x:04X}")).collect();
            cmap.push_str(&format!("<{g:04X}> <{hex}>\n"));
        }
        cmap.push_str("endbfchar\n");
    }
    cmap.push_str("endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n");
    let tu = d.add_object(Stream::new(dictionary! {}, cmap.into_bytes()));

    let id = d.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "Type0", "BaseFont" => Object::Name(ad.into()),
        "Encoding" => "Identity-H", "DescendantFonts" => vec![Object::Reference(cid)], "ToUnicode" => tu,
    });
    Ok(Yazi { id, harita })
}

/// Rastgele 32 bayt (AES-256 dosya anahtarı için).
pub fn rastgele32() -> Result<[u8; 32], String> {
    let mut b = [0u8; 32];
    getrandom::fill(&mut b).map_err(|e| e.to_string())?;
    Ok(b)
}
