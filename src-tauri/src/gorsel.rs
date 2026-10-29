//! PDF sayfalarını Windows'un yerleşik PDF motoruyla (Windows.Data.Pdf) görsele çevirir.

use std::path::{Path, PathBuf};
use windows::{
    core::HSTRING,
    Data::Pdf::{PdfDocument, PdfPageRenderOptions},
    Graphics::Imaging::BitmapEncoder,
    Storage::{
        StorageFile,
        Streams::{DataReader, InMemoryRandomAccessStream},
    },
    Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED},
};

/// `klasor` içine sayfa-001.png gibi dosyalar yazar; her sayfadan sonra `ilerle(i, toplam)` çağrılır.
pub fn pdf_gorsel(
    yol: &Path,
    klasor: &Path,
    dpi: u32,
    jpg: bool,
    ilerle: impl Fn(u32, u32),
) -> Result<Vec<PathBuf>, String> {
    let e = |e: windows::core::Error| e.message().to_string();
    unsafe {
        // Bu iş parçacığında zaten başlatılmışsa S_FALSE döner; ikisi de sorun değil.
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
    let dosya = StorageFile::GetFileFromPathAsync(&HSTRING::from(yol.as_os_str()))
        .and_then(|a| a.join())
        .map_err(e)?;
    let belge = PdfDocument::LoadFromFileAsync(&dosya)
        .and_then(|a| a.join())
        .map_err(|_| crate::dil::t("PDF açılamadı (bozuk ya da parola korumalı olabilir)", "Could not open the PDF (it may be damaged or password protected)").to_string())?;
    let toplam = belge.PageCount().map_err(e)?;
    std::fs::create_dir_all(klasor).map_err(|e| e.to_string())?;
    let basamak = toplam.to_string().len().max(3);
    let uzanti = if jpg { "jpg" } else { "png" };
    let kodlayici = if jpg { BitmapEncoder::JpegEncoderId() } else { BitmapEncoder::PngEncoderId() }.map_err(e)?;
    let mut yazilan = Vec::new();

    for i in 0..toplam {
        let sayfa = belge.GetPage(i).map_err(e)?;
        // Size, 1/96 inç birimindedir.
        let boyut = sayfa.Size().map_err(e)?;
        let secenek = PdfPageRenderOptions::new().map_err(e)?;
        secenek.SetDestinationWidth((boyut.Width * dpi as f32 / 96.0).round() as u32).map_err(e)?;
        secenek.SetDestinationHeight((boyut.Height * dpi as f32 / 96.0).round() as u32).map_err(e)?;
        secenek.SetBitmapEncoderId(kodlayici).map_err(e)?;

        let akis = InMemoryRandomAccessStream::new().map_err(e)?;
        sayfa.RenderWithOptionsToStreamAsync(&akis, &secenek).and_then(|a| a.join()).map_err(e)?;
        let uzunluk = akis.Size().map_err(e)? as u32;
        let okuyucu = DataReader::CreateDataReader(&akis.GetInputStreamAt(0).map_err(e)?).map_err(e)?;
        okuyucu.LoadAsync(uzunluk).and_then(|a| a.join()).map_err(e)?;
        let mut veri = vec![0u8; uzunluk as usize];
        okuyucu.ReadBytes(&mut veri).map_err(e)?;

        let hedef = klasor.join(format!("sayfa-{:0basamak$}.{uzanti}", i + 1));
        std::fs::write(&hedef, &veri).map_err(|e| e.to_string())?;
        yazilan.push(hedef);
        ilerle(i + 1, toplam);
    }
    Ok(yazilan)
}
