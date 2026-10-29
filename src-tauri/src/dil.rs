//! Rust'tan arayüze giden metinlerin dili (dosya adı ekleri, ilerleme ve hata iletileri).
//! Arayüz açılışta `dil_ayarla` ile bildirir; varsayılan Türkçe.

use std::sync::atomic::{AtomicBool, Ordering};

static INGILIZCE: AtomicBool = AtomicBool::new(false);

pub fn ayarla(en: bool) {
    INGILIZCE.store(en, Ordering::Relaxed);
}

pub fn en() -> bool {
    INGILIZCE.load(Ordering::Relaxed)
}

/// Sabit metin: t("Türkçe", "English").
pub fn t(tr: &'static str, en: &'static str) -> &'static str {
    if self::en() { en } else { tr }
}

/// Biçimlendirilmiş metin: s(format!(..), format!(..)).
pub fn s(tr: String, en: String) -> String {
    if self::en() { en } else { tr }
}
