// Arayüz dili. Kaynak metinler Türkçe; İngilizcede bu sözlükten çevrilir. Windows dili Türkçe ise
// Türkçe, değilse İngilizce açılır; yan menüdeki TR/EN düğmesi seçimi kalıcı yapar.
const DIL = (() => {
  try {
    const s = localStorage.getItem('dil');
    if (s === 'tr' || s === 'en') return s;
  } catch (_) {}
  return (navigator.language || '').toLowerCase().startsWith('tr') ? 'tr' : 'en';
})();

const SOZLUK = {
  // Araçlar ve gruplar
  'Dönüştür': 'Convert', 'PDF araçları': 'PDF tools', 'Düzenle': 'Edit', 'Güvenlik': 'Security',
  "PDF'e çevir": 'Office → PDF', 'Belge tara': 'Scan document', 'Görselden PDF': 'Images → PDF', "PDF'ten görsele": 'PDF → images',
  'Birleştir': 'Merge', 'Böl': 'Split', 'Sayfa çıkar': 'Extract pages', 'Döndür': 'Rotate', 'Sıkıştır': 'Compress',
  'Metni düzenle': 'Edit text', 'Sayfa numarası': 'Page numbers', 'Filigran': 'Watermark', 'Parola': 'Password',
  'PowerPoint, Word ve Excel dosyalarını bilgisayarındaki Office ile PDF yapar.': 'Turns PowerPoint, Word and Excel files into PDF with the Office installed on your PC.',
  'PPTX, PPT, DOCX, XLSX… bırakır bırakmaz çevrilir': 'PPTX, PPT, DOCX, XLSX… converted the moment you drop them',
  'Telefonla çektiğin belge fotoğraflarını düzeltir, temizler ve tek PDF yapar.': 'Straightens and cleans up phone photos of documents and makes one PDF.',
  'Fotoğrafları bırak — köşeler kendiliğinden bulunur': 'Drop photos — corners are found automatically',
  "Fotoğrafları sırayla tek bir PDF yapar. JPEG'ler yeniden sıkıştırılmaz.": 'Puts photos into one PDF in order. JPEGs are not recompressed.',
  'JPG, PNG, WEBP… sırayı aşağıda ayarla': 'JPG, PNG, WEBP… set the order below',
  'PDF oluştur': 'Create PDF',
  'Her sayfayı ayrı bir görsel olarak kaydeder.': 'Saves every page as a separate image.',
  'PDF bırak': 'Drop a PDF',
  "Birden fazla PDF'i listedeki sırayla tek dosyada birleştirir.": 'Joins several PDFs into one file, in the order of the list.',
  "PDF'leri bırak, sırayı aşağıda ayarla": 'Drop PDFs, set the order below',
  "PDF'i parçalara ayırır; parçalar yeni bir klasöre yazılır.": 'Splits a PDF into parts, written to a new folder.',
  "Seçtiğin sayfaları, yazdığın sırayla yeni bir PDF'e kopyalar.": 'Copies the pages you choose, in the order you type, into a new PDF.',
  'PDF bırak, sonra sayfaları yaz': 'Drop a PDF, then type the pages',
  'Sayfaları çıkar': 'Extract pages',
  'Sayfaları döndürüp yeni bir kopya kaydeder.': 'Rotates pages and saves a new copy.',
  'İçindeki görselleri küçültüp yeniden sıkıştırır. Yazılar ve vektörler aynı kalır.': 'Downsizes and recompresses embedded images. Text and vector graphics stay as they are.',
  'Sayfadaki bir yazıya tıklayıp değiştir ya da boş bir yere yeni yazı ekle.': 'Click any text on a page to change it, or click empty space to add new text.',
  'PDF bırak, düzenleyici açılır': 'Drop a PDF to open the editor',
  'Her sayfaya numara ekler.': 'Adds a number to every page.',
  'Her sayfaya çapraz, yarı saydam bir yazı ekler.': 'Adds a diagonal, semi-transparent text to every page.',
  "PDF'e AES-256 parola koyar ya da bildiğin parolayı kaldırır.": 'Adds an AES-256 password to a PDF, or removes a password you know.',
  'Önce parolayı yaz, sonra PDF bırak': 'Type the password first, then drop a PDF',
  // Seçenekler
  'Her': 'Every', 'sayfada bir dosya': 'pages per file', 'Sayfalar': 'Pages', 'örn. 1-3, 5, 8-': 'e.g. 1-3, 5, 8-',
  'Yön': 'Direction', '↻ Sağa 90°': '↻ 90° right', '↺ Sola 90°': '↺ 90° left', 'boş = hepsi': 'empty = all',
  'Görünüm': 'Look', 'Taranmış': 'Scanned', 'Renkli': 'Color', 'Orijinal': 'Original', 'Köşe bulucu': 'Corner detector',
  'Seviye': 'Level', 'Hafif': 'Light', 'Orta': 'Medium', 'Güçlü': 'Strong',
  'Parola koy': 'Add password', 'Parolayı kaldır': 'Remove password',
  'Konum': 'Position', 'Alt orta': 'Bottom center', 'Alt sağ': 'Bottom right', 'Üst sağ': 'Top right',
  'Biçim': 'Format', 'Sayfa 1': 'Page 1', 'Başlangıç': 'Start at', 'Metin': 'Text', 'Belirginlik': 'Strength', 'Belirgin': 'Bold',
  'Kalite': 'Quality', 'Ekran': 'Screen', 'Normal': 'Normal', 'Baskı': 'Print',
  // Liste ve işler
  'Yukarı': 'Up', 'Aşağı': 'Down', 'Listeden çıkar': 'Remove from list', 'Listeyi temizle': 'Clear list',
  'Hazırlanıyor…': 'Preparing…', 'Sırada…': 'Queued…', 'Aç': 'Open', 'Klasörde göster': 'Show in folder',
  'Henüz işlem yok. Bıraktığın dosyalar kaynakla aynı klasöre kaydedilir.': 'Nothing yet. Results are saved next to the files you drop.',
  'Son işlemler': 'Recent', 'Temizle': 'Clear', 'Dosya seç': 'Choose files', 'Dosyaları buraya sürükle': 'Drag files here',
  'Bırak': 'Drop', "PDF'e çevrilecek": 'Will be converted to PDF',
  '{arac} bu dosya türünü almıyor ({tur}…)': "{arac} doesn't take this file type ({tur}…)",
  'Bölünüyor…': 'Splitting…', 'Döndürülüyor…': 'Rotating…', 'Sayfalar çiziliyor…': 'Rendering pages…',
  'Görseller küçültülüyor…': 'Downsizing images…', 'Şifreleniyor…': 'Encrypting…', 'Parola kaldırılıyor…': 'Removing password…',
  'Önce yukarıya parolayı yaz': 'Type the password above first', 'Numaralanıyor…': 'Numbering…', 'Filigran ekleniyor…': 'Adding watermark…',
  '{n} PDF birleştiriliyor': 'Merging {n} PDFs', 'Birleştiriliyor…': 'Merging…', '{n} görsel → PDF': '{n} images → PDF',
  'Oluşturuluyor…': 'Creating…', 'Sayfa {a} çıkarılıyor…': 'Extracting pages {a}…', '{n} dosya': '{n} files', '{n} sayfa': '{n} pages',
  // Office ve model
  'Office: hazır': 'Office: ready',
  "Office bulunamadı — PDF'e çevirme çalışmaz": 'Office not found — Office → PDF will not work',
  'Office: yalnızca {x}': 'Office: only {x}',
  'Klasik (model yok)': 'Classic (no model)', 'İndiriliyor… %{p}': 'Downloading… {p}%', 'İndir ({mb} MB)': 'Download ({mb} MB)',
  'Hazır': 'Ready', 'Sil': 'Delete', 'Model indirme': 'Model download',
  'Model gerektirmez; dağınık zeminde ve gölgede yanılabilir.': 'Needs no model; can be wrong on cluttered backgrounds and in shadows.',
  '{a}. DocsaidLab/DocAligner projesinden, kendi kaynağından indirilir (kod Apache-2.0). Doğruluğu ve lisansı modelin sahibine aittir.':
    "{a}. Downloaded from the DocsaidLab/DocAligner project's own source (code: Apache-2.0). Its accuracy and license belong to its authors.",
  'Köşe bulucu: {a}': 'Corners: {a}', 'Köşe bulucu: klasik': 'Corners: classic',
  // Tarama
  'Sıralamak için kartları sürükle · köşeleri düzeltmek için karta tıkla': 'Drag cards to reorder · click a card to adjust corners',
  'Köşeleri ayarla': 'Adjust corners', 'Sağa döndür': 'Rotate right', 'Çıkar': 'Remove',
  '{n} sayfalık tarama': '{n}-page scan', 'Sayfalar düzeltiliyor…': 'Correcting pages…',
  'Yuvarlakları belgenin köşelerine sürükle': "Drag the circles onto the document's corners",
  'Otomatik bul': 'Auto detect', 'Tüm fotoğraf': 'Whole photo', 'Vazgeç': 'Cancel', 'Uygula': 'Apply',
  'Fotoğraf': 'Photo', 'Sonuç — perspektif düzeltilmiş': 'Result — perspective corrected',
  // Metin düzenleyici
  'Yazıya tıkla ve değiştir · boş yere tıklayınca yeni yazı eklenir · Enter onaylar, Esc vazgeçer':
    'Click text to change it · click empty space to add text · Enter confirms, Esc cancels',
  'Önceki sayfa': 'Previous page', 'Sonraki sayfa': 'Next page', 'Kapat': 'Close', 'Kaydet': 'Save',
  'Açılamadı: {h}': 'Could not open: {h}', '{n} değişiklik': '{n} changes',
  'Kaydedilmemiş değişiklikler silinsin mi?': 'Discard unsaved changes?', '{n} değişiklik yazılıyor…': 'Writing {n} changes…',
};

/// Çeviri: d('Kaynak metin {x}', { x: 1 })
function d(metin, degerler) {
  let s = DIL === 'en' ? SOZLUK[metin] ?? metin : metin;
  if (degerler) for (const [k, v] of Object.entries(degerler)) s = s.split(`{${k}}`).join(String(v));
  return s;
}

/// index.html'deki sabit metinleri ve title/placeholder özniteliklerini çevirir.
function htmlCevir() {
  document.documentElement.lang = DIL;
  if (DIL === 'tr') return;
  const yuru = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
  for (let n = yuru.nextNode(); n; n = yuru.nextNode()) {
    const k = n.nodeValue.trim();
    if (k && SOZLUK[k]) n.nodeValue = n.nodeValue.replace(k, SOZLUK[k]);
  }
  for (const el of document.querySelectorAll('[title],[placeholder]')) {
    for (const a of ['title', 'placeholder']) if (el.getAttribute(a) && SOZLUK[el.getAttribute(a)]) el.setAttribute(a, SOZLUK[el.getAttribute(a)]);
  }
}
