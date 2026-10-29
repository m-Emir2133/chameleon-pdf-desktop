const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const { getCurrentWebview } = window.__TAURI__.webview;
const { open: dosyaSec } = window.__TAURI__.dialog;

const $ = (s) => document.querySelector(s);
const OFFICE = ['pptx', 'ppt', 'pptm', 'ppsx', 'pps', 'potx', 'odp', 'docx', 'doc', 'docm', 'odt', 'rtf', 'xlsx', 'xls', 'xlsm', 'xlsb', 'ods', 'csv'];
const PDF = ['pdf'];
const GORSEL = ['jpg', 'jpeg', 'png', 'webp', 'bmp', 'gif', 'tif', 'tiff'];

const ikon = (yol) => `<svg viewBox="0 0 24 24">${yol}</svg>`;
const IKON = {
  tara: ikon('<path d="M4 8V5a1 1 0 0 1 1-1h3M16 4h3a1 1 0 0 1 1 1v3M20 16v3a1 1 0 0 1-1 1h-3M8 20H5a1 1 0 0 1-1-1v-3"/><path d="m8 7 8 1-1 9-8-1z"/>'),
  office: ikon('<path d="M7 3h7l5 5v13H7z"/><path d="M14 3v5h5M10 13h6M10 17h4"/>'),
  birlestir: ikon('<path d="M8 4v6a4 4 0 0 0 4 4h0a4 4 0 0 1 4 4v2M16 4v6a4 4 0 0 1-4 4"/><path d="m13 18 3 3 3-3"/>'),
  bol: ikon('<path d="M6 3h12v7H6zM6 14h12v7H6z"/><path d="M3 12h18" stroke-dasharray="2 2"/>'),
  cikar: ikon('<path d="M6 3h9l3 3v15H6z"/><path d="m9 13 2.5 2.5L16 11"/>'),
  dondur: ikon('<path d="M20 12a8 8 0 1 1-2.3-5.6L20 8.6"/><path d="M20 3v5.6h-5.6"/>'),
  gorselPdf: ikon('<rect x="3" y="4" width="18" height="14" rx="2"/><path d="m3 15 5-5 4 4 3-3 6 6M15.5 8.5h.01"/>'),
  sikistir: ikon('<path d="M4 14h6v6M20 10h-6V4M14 10l7-7M3 21l7-7"/>'),
  parola: ikon('<rect x="5" y="11" width="14" height="10" rx="2"/><path d="M8 11V7a4 4 0 0 1 8 0v4"/>'),
  sayfaNo: ikon('<path d="M6 3h9l3 3v15H6z"/><path d="M10 17h4M12 13v4"/>'),
  filigran: ikon('<path d="M6 3h9l3 3v15H6z"/><path d="m8 16 8-6"/>'),
  metin: ikon('<path d="M4 20h4L19 9l-4-4L4 16z"/><path d="m14 6 4 4"/>'),
  pdfGorsel: ikon('<path d="M6 3h9l3 3v6"/><path d="M6 3v10"/><rect x="9" y="13" width="12" height="8" rx="1.5"/><path d="m9 19 3-3 3 3 2-2 4 4"/>'),
};

// Her araç: hangi dosyaları kabul ettiği, seçenekleri ve bırakınca hemen mi çalıştığı.
const ARACLAR = [
  { id: 'office', grup: 'Dönüştür', ad: "PDF'e çevir", ikon: IKON.office, kabul: OFFICE,
    aciklama: 'PowerPoint, Word ve Excel dosyalarını bilgisayarındaki Office ile PDF yapar.',
    birakAlt: 'PPTX, PPT, DOCX, XLSX… bırakır bırakmaz çevrilir', otomatik: true },
  { id: 'tara', grup: 'Dönüştür', ad: 'Belge tara', ikon: IKON.tara, kabul: GORSEL,
    aciklama: 'Telefonla çektiğin belge fotoğraflarını düzeltir, temizler ve tek PDF yapar.',
    birakAlt: 'Fotoğrafları bırak — köşeler kendiliğinden bulunur' },
  { id: 'gorselPdf', grup: 'Dönüştür', ad: "Görselden PDF", ikon: IKON.gorselPdf, kabul: GORSEL,
    aciklama: 'Fotoğrafları sırayla tek bir PDF yapar. JPEG\'ler yeniden sıkıştırılmaz.',
    birakAlt: 'JPG, PNG, WEBP… sırayı aşağıda ayarla', liste: true, buton: 'PDF oluştur' },
  { id: 'pdfGorsel', grup: 'Dönüştür', ad: "PDF'ten görsele", ikon: IKON.pdfGorsel, kabul: PDF,
    aciklama: 'Her sayfayı ayrı bir görsel olarak kaydeder.', birakAlt: 'PDF bırak', otomatik: true },
  { id: 'birlestir', grup: 'PDF araçları', ad: 'Birleştir', ikon: IKON.birlestir, kabul: PDF,
    aciklama: 'Birden fazla PDF\'i listedeki sırayla tek dosyada birleştirir.',
    birakAlt: 'PDF\'leri bırak, sırayı aşağıda ayarla', liste: true, buton: 'Birleştir' },
  { id: 'bol', grup: 'PDF araçları', ad: 'Böl', ikon: IKON.bol, kabul: PDF,
    aciklama: 'PDF\'i parçalara ayırır; parçalar yeni bir klasöre yazılır.', birakAlt: 'PDF bırak', otomatik: true },
  { id: 'cikar', grup: 'PDF araçları', ad: 'Sayfa çıkar', ikon: IKON.cikar, kabul: PDF,
    aciklama: 'Seçtiğin sayfaları, yazdığın sırayla yeni bir PDF\'e kopyalar.',
    birakAlt: 'PDF bırak, sonra sayfaları yaz', liste: true, buton: 'Sayfaları çıkar' },
  { id: 'dondur', grup: 'PDF araçları', ad: 'Döndür', ikon: IKON.dondur, kabul: PDF,
    aciklama: 'Sayfaları döndürüp yeni bir kopya kaydeder.', birakAlt: 'PDF bırak', otomatik: true },
  { id: 'sikistir', grup: 'PDF araçları', ad: 'Sıkıştır', ikon: IKON.sikistir, kabul: PDF,
    aciklama: 'İçindeki görselleri küçültüp yeniden sıkıştırır. Yazılar ve vektörler aynı kalır.', birakAlt: 'PDF bırak', otomatik: true },
  { id: 'metin', grup: 'Düzenle', ad: 'Metni düzenle', ikon: IKON.metin, kabul: PDF,
    aciklama: 'Sayfadaki bir yazıya tıklayıp değiştir ya da boş bir yere yeni yazı ekle.', birakAlt: 'PDF bırak, düzenleyici açılır' },
  { id: 'sayfaNo', grup: 'Düzenle', ad: 'Sayfa numarası', ikon: IKON.sayfaNo, kabul: PDF,
    aciklama: 'Her sayfaya numara ekler.', birakAlt: 'PDF bırak', otomatik: true },
  { id: 'filigran', grup: 'Düzenle', ad: 'Filigran', ikon: IKON.filigran, kabul: PDF,
    aciklama: 'Her sayfaya çapraz, yarı saydam bir yazı ekler.', birakAlt: 'PDF bırak', otomatik: true },
  { id: 'parola', grup: 'Güvenlik', ad: 'Parola', ikon: IKON.parola, kabul: PDF,
    aciklama: 'PDF\'e AES-256 parola koyar ya da bildiğin parolayı kaldırır.', birakAlt: 'Önce parolayı yaz, sonra PDF bırak', otomatik: true },
];
for (const a of ARACLAR) for (const k of ['grup', 'ad', 'aciklama', 'birakAlt', 'buton']) if (a[k]) a[k] = d(a[k]);

const ayar = {
  bolAdim: 1, aralik: '', derece: 90, donAralik: '', dpi: 150, jpg: false, taraFiltre: 'tarama', taraModel: 'klasik',
  seviye: 2, parolaKip: 'koy', konum: 'alt-orta', bicim: '{n} / {N}', baslangic: 1, filigranMetin: DIL === 'en' ? 'CONFIDENTIAL' : 'GİZLİ', opaklik: 0.25,
};
// Parola diske yazılmaz; yalnızca bu oturumda bellekte durur.
let parola = '';
try { Object.assign(ayar, JSON.parse(localStorage.getItem('ayar') || '{}')); } catch (_) {}
const kaydetAyar = () => { try { localStorage.setItem('ayar', JSON.stringify(ayar)); } catch (_) {} };

let arac = ARACLAR[0];
let liste = []; // { yol, sayfa? }

const ad = (yol) => yol.split(/[\\/]/).pop();
const uzanti = (yol) => (yol.split('.').pop() || '').toLowerCase();
const kacis = (s) => String(s).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));

// ---------- Yan çubuk ----------
function yanCiz() {
  let grup = '';
  $('#araclar').innerHTML = ARACLAR.map((a) => {
    const baslik = a.grup !== grup ? `<div class="grup">${(grup = a.grup)}</div>` : '';
    return `${baslik}<button data-id="${a.id}" class="${a === arac ? 'secili' : ''}">${a.ikon}<span>${a.ad}</span></button>`;
  }).join('');
}
$('#araclar').addEventListener('click', (e) => {
  const b = e.target.closest('button[data-id]');
  if (!b) return;
  aracSec(ARACLAR.find((a) => a.id === b.dataset.id));
});

function aracSec(a) {
  if (a !== arac) liste = [];
  arac = a;
  try { localStorage.setItem('arac', a.id); } catch (_) {}
  yanCiz();
  $('#baslik').textContent = a.ad;
  $('#aciklama').textContent = a.aciklama;
  $('#birak-alt').textContent = a.birakAlt;
  $('#tarama').hidden = a.id !== 'tara';
  seceneklerCiz();
  listeCiz();
  if (a.id === 'tara') kartlariCiz();
}

// ---------- Seçenekler ----------
function parcali(ad, secenekler, deger) {
  return `<span class="parcali" data-ad="${ad}">${secenekler
    .map(([v, e]) => `<button data-v="${v}" class="${String(v) === String(deger) ? 'secili' : ''}">${d(e)}</button>`)
    .join('')}</span>`;
}

function seceneklerCiz() {
  const s = $('#secenekler');
  if (arac.id === 'bol') {
    s.innerHTML = `<label class="secenek">${d('Her')} <input type="number" min="1" id="bolAdim" value="${ayar.bolAdim}"> ${d('sayfada bir dosya')}</label>`;
  } else if (arac.id === 'cikar') {
    s.innerHTML = `<label class="secenek">${d('Sayfalar')} <input type="text" id="aralik" placeholder="${d('örn. 1-3, 5, 8-')}" value="${kacis(ayar.aralik)}"></label>`;
  } else if (arac.id === 'dondur') {
    s.innerHTML = `<span class="secenek">${d('Yön')} ${parcali('derece', [[90, '↻ Sağa 90°'], [180, '180°'], [270, '↺ Sola 90°']], ayar.derece)}</span>
      <label class="secenek">${d('Sayfalar')} <input type="text" id="donAralik" placeholder="${d('boş = hepsi')}" value="${kacis(ayar.donAralik)}"></label>`;
  } else if (arac.id === 'tara') {
    s.innerHTML = `<span class="secenek">${d('Görünüm')} ${parcali('taraFiltre', FILTRELER, ayar.taraFiltre)}</span>
      <label class="secenek">${d('Köşe bulucu')} <select id="taraModel">${modelSecenekleri()}</select></label>
      <span class="secenek" id="model-eylem">${modelEylemi()}</span>
      <span class="ipucu model-not">${modelNotu()}</span>`;
  } else if (arac.id === 'sikistir') {
    s.innerHTML = `<span class="secenek">${d('Seviye')} ${parcali('seviye', [[1, 'Hafif'], [2, 'Orta'], [3, 'Güçlü']], ayar.seviye)}</span>`;
  } else if (arac.id === 'parola') {
    s.innerHTML = `<span class="secenek">${parcali('parolaKip', [['koy', 'Parola koy'], ['kaldir', 'Parolayı kaldır']], ayar.parolaKip)}</span>
      <label class="secenek">${d('Parola')} <input type="password" id="parola" autocomplete="off" value="${kacis(parola)}"></label>`;
  } else if (arac.id === 'sayfaNo') {
    s.innerHTML = `<span class="secenek">${d('Konum')} ${parcali('konum', [['alt-orta', 'Alt orta'], ['alt-sag', 'Alt sağ'], ['ust-sag', 'Üst sağ']], ayar.konum)}</span>
      <span class="secenek">${d('Biçim')} ${parcali('bicim', [['{n}', '1'], ['{n} / {N}', '1 / 10'], [DIL === 'en' ? 'Page {n}' : 'Sayfa {n}', 'Sayfa 1']], ayar.bicim)}</span>
      <label class="secenek">${d('Başlangıç')} <input type="number" min="1" id="baslangic" value="${ayar.baslangic}"></label>`;
  } else if (arac.id === 'filigran') {
    s.innerHTML = `<label class="secenek">${d('Metin')} <input type="text" id="filigranMetin" value="${kacis(ayar.filigranMetin)}"></label>
      <span class="secenek">${d('Belirginlik')} ${parcali('opaklik', [[0.12, 'Hafif'], [0.25, 'Orta'], [0.45, 'Belirgin']], ayar.opaklik)}</span>`;
  } else if (arac.id === 'pdfGorsel') {
    s.innerHTML = `<span class="secenek">${d('Kalite')} ${parcali('dpi', [[96, 'Ekran'], [150, 'Normal'], [300, 'Baskı']], ayar.dpi)}</span>
      <span class="secenek">${d('Biçim')} ${parcali('jpg', [[false, 'PNG'], [true, 'JPG']], ayar.jpg)}</span>`;
  } else {
    s.innerHTML = '';
  }
}
$('#secenekler').addEventListener('click', (e) => {
  const b = e.target.closest('.parcali button');
  if (!b) return;
  const alan = b.parentElement.dataset.ad;
  const v = b.dataset.v;
  ayar[alan] = v === 'true' ? true : v === 'false' ? false : v !== '' && !isNaN(Number(v)) ? Number(v) : v;
  kaydetAyar();
  seceneklerCiz();
  // Görünüm değişince tüm tarama sayfalarına uygula.
  if (alan === 'taraFiltre') sayfalar.forEach((s) => { s.filtre = ayar.taraFiltre; onizle(s); });
});
$('#secenekler').addEventListener('input', (e) => {
  const el = e.target;
  if (el.id === 'bolAdim') ayar.bolAdim = Math.max(1, parseInt(el.value, 10) || 1);
  if (el.id === 'aralik') ayar.aralik = el.value;
  if (el.id === 'donAralik') ayar.donAralik = el.value;
  if (el.id === 'baslangic') ayar.baslangic = Math.max(1, parseInt(el.value, 10) || 1);
  if (el.id === 'filigranMetin') ayar.filigranMetin = el.value;
  if (el.id === 'parola') { parola = el.value; return; }
  kaydetAyar();
  baslatDurumu();
});

// ---------- Sıralanabilir liste (birleştir, görselden PDF, sayfa çıkar) ----------
function listeCiz() {
  const alan = $('#liste-alani');
  $('#birak').classList.toggle('ince', !!arac.liste && liste.length > 0);
  if (!arac.liste || liste.length === 0) { alan.hidden = true; return; }
  alan.hidden = false;
  $('#liste-sayac').textContent = d('{n} dosya', { n: liste.length });
  $('#baslat').textContent = arac.buton;
  $('#liste').innerHTML = liste.map((o, i) => `
    <li>
      <span class="sira">${i + 1}</span>
      <span class="ad" title="${kacis(o.yol)}">${kacis(ad(o.yol))}</span>
      <span class="bilgi">${o.sayfa != null ? d('{n} sayfa', { n: o.sayfa }) : ''}</span>
      <button class="kucuk" data-i="${i}" data-e="yukari" ${i === 0 ? 'disabled' : ''} title="${d('Yukarı')}">▲</button>
      <button class="kucuk" data-i="${i}" data-e="asagi" ${i === liste.length - 1 ? 'disabled' : ''} title="${d('Aşağı')}">▼</button>
      <button class="kucuk" data-i="${i}" data-e="sil" title="${d('Listeden çıkar')}">✕</button>
    </li>`).join('');
  baslatDurumu();
}
function baslatDurumu() {
  const b = $('#baslat');
  if (!b) return;
  const eksik = (arac.id === 'birlestir' && liste.length < 2) || (arac.id === 'cikar' && !ayar.aralik.trim());
  b.disabled = liste.length === 0 || eksik;
}
$('#liste').addEventListener('click', (e) => {
  const b = e.target.closest('button[data-e]');
  if (!b) return;
  const i = Number(b.dataset.i);
  if (b.dataset.e === 'sil') liste.splice(i, 1);
  else {
    const j = b.dataset.e === 'yukari' ? i - 1 : i + 1;
    [liste[i], liste[j]] = [liste[j], liste[i]];
  }
  listeCiz();
});
$('#temizle').addEventListener('click', () => { liste = []; listeCiz(); });
$('#baslat').addEventListener('click', () => listeyiCalistir());

// ---------- İşler ----------
const isler = new Map(); // anahtar -> li
let isSayac = 0;
const IKON_TAMAM = ikon('<path d="m5 12 5 5 9-10"/>');
const IKON_HATA = ikon('<path d="M12 7v6M12 17h.01"/>');

function isEkle(anahtar, baslik, ileti) {
  const bos = $('#isler .bos');
  if (bos) bos.remove();
  $('#isleri-temizle').hidden = false;
  let li = isler.get(anahtar);
  if (!li) {
    li = document.createElement('li');
    isler.set(anahtar, li);
    $('#isler').prepend(li);
  }
  li.dataset.baslik = baslik;
  isGuncelle(anahtar, 'calisiyor', ileti || d('Hazırlanıyor…'), []);
  return anahtar;
}

function isGuncelle(anahtar, durum, ileti, ciktilar) {
  const li = isler.get(anahtar);
  if (!li) return;
  li.className = durum;
  const ilk = ciktilar && ciktilar[0];
  let eylem = '';
  if (durum === 'tamam' && ilk) {
    eylem = `<button class="eylem" data-ac="${kacis(ilk)}">${d('Aç')}</button>
             <button class="eylem" data-goster="${kacis(ilk)}">${d('Klasörde göster')}</button>`;
  }
  const alt = durum === 'tamam' && ilk ? `${ad(ilk)}${ciktilar[1] ? ` · ${ciktilar[1]}` : ''}` : ileti;
  li.innerHTML = `
    <span class="durum">${durum === 'tamam' ? IKON_TAMAM : durum === 'hata' ? IKON_HATA : ''}</span>
    <span class="metinler"><b>${kacis(li.dataset.baslik)}</b><span>${kacis(alt)}</span></span>${eylem}`;
}

$('#isler').addEventListener('click', (e) => {
  const b = e.target.closest('button');
  if (!b) return;
  if (b.dataset.ac) invoke('ac', { yol: b.dataset.ac });
  if (b.dataset.goster) invoke('klasorde_goster', { yol: b.dataset.goster });
});
$('#isleri-temizle').addEventListener('click', () => {
  for (const [k, li] of isler) if (!li.classList.contains('calisiyor')) { li.remove(); isler.delete(k); }
  if (isler.size === 0) {
    $('#isler').innerHTML = `<li class="bos">${d('Henüz işlem yok. Bıraktığın dosyalar kaynakla aynı klasöre kaydedilir.')}</li>`;
    $('#isleri-temizle').hidden = true;
  }
});

listen('ilerleme', ({ payload: p }) => isGuncelle(p.anahtar, p.durum, p.ileti, p.ciktilar));

async function calistir(baslik, ileti, is) {
  const anahtar = `is-${++isSayac}`;
  isEkle(anahtar, baslik, ileti);
  try {
    const ciktilar = await is(anahtar);
    isGuncelle(anahtar, 'tamam', '', ciktilar);
  } catch (h) {
    isGuncelle(anahtar, 'hata', String(h), []);
  }
}

// ---------- Dosya alma ----------
async function dosyalarGeldi(yollar) {
  const office = yollar.filter((y) => OFFICE.includes(uzanti(y)));
  const uygun = arac.id === 'office' ? [] : yollar.filter((y) => arac.kabul.includes(uzanti(y)));
  const diger = yollar.filter((y) => !office.includes(y) && !uygun.includes(y));

  // Office dosyaları hangi araçta bırakılırsa bırakılsın PDF'e çevrilir.
  if (office.length) {
    for (const y of office) isEkle(y, ad(y), d('Sırada…'));
    invoke('office', { dosyalar: office }).catch((h) => office.forEach((y) => isGuncelle(y, 'hata', String(h), [])));
  }
  for (const y of diger) {
    const anahtar = `is-${++isSayac}`;
    isEkle(anahtar, ad(y), '');
    isGuncelle(anahtar, 'hata', d('{arac} bu dosya türünü almıyor ({tur}…)', { arac: arac.ad, tur: arac.kabul.slice(0, 4).join(', ') }), []);
  }
  if (!uygun.length) return;

  if (arac.id === 'tara') { uygun.forEach(taramaEkle); return; }
  if (arac.id === 'metin') { mdAc(uygun[0]); return; }

  if (arac.liste) {
    const yeni = uygun.filter((y) => !liste.some((o) => o.yol === y)).map((yol) => ({ yol }));
    liste.push(...(arac.id === 'cikar' ? yeni.slice(0, 1) : yeni));
    if (arac.id === 'cikar') liste = liste.slice(-1);
    listeCiz();
    if (arac.kabul === PDF) {
      for (const o of liste.filter((o) => o.sayfa == null)) {
        invoke('sayfa_sayisi', { dosya: o.yol }).then((n) => { o.sayfa = n; listeCiz(); }).catch(() => {});
      }
    }
    if (arac.id === 'cikar') $('#aralik')?.focus();
    return;
  }

  for (const y of uygun) {
    if (arac.id === 'bol') {
      calistir(ad(y), d('Bölünüyor…'), () => invoke('bol', { dosya: y, adim: ayar.bolAdim }));
    } else if (arac.id === 'dondur') {
      calistir(ad(y), d('Döndürülüyor…'), () => invoke('dondur', { dosya: y, derece: ayar.derece, aralik: ayar.donAralik }));
    } else if (arac.id === 'pdfGorsel') {
      calistir(ad(y), d('Sayfalar çiziliyor…'), (anahtar) => invoke('pdf_gorsel', { anahtar, dosya: y, dpi: ayar.dpi, jpg: ayar.jpg }));
    } else if (arac.id === 'sikistir') {
      calistir(ad(y), d('Görseller küçültülüyor…'), () => invoke('sikistir', { dosya: y, seviye: ayar.seviye }));
    } else if (arac.id === 'parola') {
      const komut = ayar.parolaKip === 'koy' ? 'parola_koy' : 'parola_kaldir';
      calistir(ad(y), d(ayar.parolaKip === 'koy' ? 'Şifreleniyor…' : 'Parola kaldırılıyor…'), async () => {
        if (!parola) throw d('Önce yukarıya parolayı yaz');
        return invoke(komut, { dosya: y, parola });
      });
    } else if (arac.id === 'sayfaNo') {
      calistir(ad(y), d('Numaralanıyor…'), () => invoke('sayfa_numarasi', { dosya: y, konum: ayar.konum, bicim: ayar.bicim, baslangic: ayar.baslangic }));
    } else if (arac.id === 'filigran') {
      calistir(ad(y), d('Filigran ekleniyor…'), () => invoke('filigran', { dosya: y, metin: ayar.filigranMetin, opaklik: ayar.opaklik }));
    }
  }
}

function listeyiCalistir() {
  const yollar = liste.map((o) => o.yol);
  if (!yollar.length) return;
  if (arac.id === 'birlestir') {
    calistir(d('{n} PDF birleştiriliyor', { n: yollar.length }), d('Birleştiriliyor…'), () => invoke('birlestir', { dosyalar: yollar }));
  } else if (arac.id === 'gorselPdf') {
    calistir(d('{n} görsel → PDF', { n: yollar.length }), d('Oluşturuluyor…'), () => invoke('gorsellerden_pdf', { dosyalar: yollar }));
  } else if (arac.id === 'cikar') {
    calistir(ad(yollar[0]), d('Sayfa {a} çıkarılıyor…', { a: ayar.aralik }), () => invoke('sayfa_cikar', { dosya: yollar[0], aralik: ayar.aralik }));
  }
  liste = [];
  listeCiz();
}

$('#sec').addEventListener('click', secPenceresi);
$('#birak').addEventListener('click', secPenceresi);
async function secPenceresi() {
  const secim = await dosyaSec({
    multiple: !['cikar', 'metin'].includes(arac.id),
    filters: [{ name: arac.ad, extensions: arac.kabul }],
  });
  if (!secim) return;
  dosyalarGeldi(Array.isArray(secim) ? secim : [secim]);
}

// Tauri pencereye bırakılan dosyaların yollarını verir.
getCurrentWebview().onDragDropEvent(({ payload }) => {
  const perde = $('#perde');
  if (payload.type === 'enter' || payload.type === 'over') {
    perde.hidden = false;
    $('#perde-alt').textContent = arac.id === 'office' ? d("PDF'e çevrilecek") : arac.ad;
  } else {
    perde.hidden = true;
    if (payload.type === 'drop' && payload.paths?.length) dosyalarGeldi(payload.paths);
  }
});

// Office kurulu değilse yan çubukta söyle.
invoke('office_durumu').then((var_) => {
  const not = $('#office-notu');
  if (var_.length === 3) not.textContent = d('Office: hazır');
  else if (var_.length === 0) { not.textContent = d("Office bulunamadı — PDF'e çevirme çalışmaz"); not.classList.add('uyari'); }
  else { not.textContent = d('Office: yalnızca {x}', { x: var_.join(', ') }); not.classList.add('uyari'); }
  const s = document.createElement('div');
  s.id = 'model-notu';
  not.appendChild(s);
  modelleriYukle();
});

// ---------- Köşe bulma modelleri ----------
// Modeller uygulamayla gelmez; kullanıcı seçer ve model sahibinin kaynağından indirilir.
let modelListesi = []; // { kimlik, ad, aciklama, mb, indirildi }
const indirilen = new Map(); // kimlik → yüzde (indirme sürerken)

async function modelleriYukle() {
  try { modelListesi = await invoke('modeller'); } catch (_) { modelListesi = []; }
  if (ayar.taraModel !== 'klasik' && !modelListesi.some((m) => m.kimlik === ayar.taraModel)) ayar.taraModel = 'klasik';
  modelArayuzuGuncelle();
}

const seciliModel = () => modelListesi.find((m) => m.kimlik === ayar.taraModel);
/// Seçili model indirilmişse onun kimliği, değilse "klasik".
const kullanilacakModel = () => (seciliModel()?.indirildi ? ayar.taraModel : 'klasik');

function modelSecenekleri() {
  const s = [`<option value="klasik" ${ayar.taraModel === 'klasik' ? 'selected' : ''}>${d('Klasik (model yok)')}</option>`];
  for (const m of modelListesi) {
    s.push(`<option value="${m.kimlik}" ${m.kimlik === ayar.taraModel ? 'selected' : ''}>${kacis(m.ad)}${m.indirildi ? '' : ` · ${Math.round(m.mb)} MB`}</option>`);
  }
  return s.join('');
}

function modelEylemi() {
  const m = seciliModel();
  if (!m) return '';
  if (indirilen.has(m.kimlik)) return `<span class="ipucu">${d('İndiriliyor… %{p}', { p: indirilen.get(m.kimlik) })}</span>`;
  if (!m.indirildi) return `<button class="ikincil kucuk-dugme" data-model-indir="${m.kimlik}">${d('İndir ({mb} MB)', { mb: Math.round(m.mb) })}</button>`;
  return `<span class="ipucu">${d('Hazır')}</span> <button class="metin" data-model-sil="${m.kimlik}">${d('Sil')}</button>`;
}

function modelNotu() {
  const m = seciliModel();
  if (!m) return d('Model gerektirmez; dağınık zeminde ve gölgede yanılabilir.');
  return d('{a}. DocsaidLab/DocAligner projesinden, kendi kaynağından indirilir (kod Apache-2.0). Doğruluğu ve lisansı modelin sahibine aittir.', { a: kacis(m.aciklama) });
}

function modelArayuzuGuncelle() {
  const m = seciliModel();
  const not = $('#model-notu');
  if (not) {
    not.textContent = m?.indirildi ? d('Köşe bulucu: {a}', { a: m.ad }) : d('Köşe bulucu: klasik');
    not.className = m && !m.indirildi ? 'uyari' : '';
  }
  if (arac.id === 'tara') seceneklerCiz();
}

/// Elle düzeltilmemiş sayfalarda köşeleri yeni bulucuyla yeniden arar.
async function koseleriYenidenBul() {
  for (const s of sayfalar) {
    if (!s.foto || s.hata || JSON.stringify(s.kose) !== JSON.stringify(s.otoKose)) continue;
    try {
      const h = await invoke('tarama_hazirla', { dosya: s.dosya, model: kullanilacakModel() });
      Object.assign(s, { kose: h.kose, otoKose: h.kose, yol: h.yol });
      onizle(s);
    } catch (_) {}
  }
}

$('#secenekler').addEventListener('change', (e) => {
  if (e.target.id !== 'taraModel') return;
  ayar.taraModel = e.target.value;
  kaydetAyar();
  modelArayuzuGuncelle();
  koseleriYenidenBul();
});

$('#secenekler').addEventListener('click', async (e) => {
  const indir = e.target.closest('[data-model-indir]')?.dataset.modelIndir;
  const sil = e.target.closest('[data-model-sil]')?.dataset.modelSil;
  if (indir) {
    indirilen.set(indir, 0);
    modelArayuzuGuncelle();
    try {
      await invoke('model_indir', { kimlik: indir });
      indirilen.delete(indir);
      await modelleriYukle();
      koseleriYenidenBul();
    } catch (h) {
      indirilen.delete(indir);
      modelArayuzuGuncelle();
      const anahtar = `is-${++isSayac}`;
      isEkle(anahtar, d('Model indirme'), '');
      isGuncelle(anahtar, 'hata', String(h), []);
    }
  } else if (sil) {
    await invoke('model_sil', { kimlik: sil }).catch(() => {});
    await modelleriYukle();
  }
});

listen('model-ilerleme', ({ payload: p }) => {
  if (!indirilen.has(p.kimlik)) return;
  indirilen.set(p.kimlik, Math.floor((p.inen * 100) / p.toplam));
  const el = $('#model-eylem');
  if (el && seciliModel()?.kimlik === p.kimlik) el.innerHTML = modelEylemi();
});

// ---------- Belge tarama ----------
const FILTRELER = [['tarama', 'Taranmış'], ['renkli', 'Renkli'], ['orijinal', 'Orijinal']];
const sayfalar = []; // { id, dosya, foto, gen, yuk, kose, otoKose, donus, filtre, onizleme, hata }
let sayfaSayac = 0;

async function taramaEkle(dosya) {
  const s = { id: ++sayfaSayac, dosya, donus: 0, filtre: ayar.taraFiltre };
  sayfalar.push(s);
  kartlariCiz();
  try {
    const h = await invoke('tarama_hazirla', { dosya, model: kullanilacakModel() });
    Object.assign(s, { foto: h.foto, gen: h.gen, yuk: h.yuk, kose: h.kose, otoKose: h.kose, yol: h.yol });
    await onizle(s);
  } catch (e) {
    s.hata = String(e);
    kartlariCiz();
  }
}

// Aynı sayfa için üst üste istek gelirse yalnızca sonuncusu ekrana yazılır.
async function onizle(s) {
  if (!s.kose) return;
  const no = (s.istek = (s.istek || 0) + 1);
  try {
    const url = await invoke('tarama_onizle', { sayfa: taramaVerisi(s) });
    if (no === s.istek) { s.onizleme = url; kartGuncelle(s); }
  } catch (e) { s.hata = String(e); kartGuncelle(s); }
}

const taramaVerisi = (s) => ({ dosya: s.dosya, kose: s.kose, donus: s.donus, filtre: s.filtre });

function kartIcerik(s, i) {
  const govde = s.hata ? `<span class="kart-hata">${kacis(s.hata)}</span>`
    : s.onizleme ? `<img src="${s.onizleme}" alt="">` : '<span class="bekle"></span>';
  return `<span class="no">${i + 1}</span>${govde}
    <span class="kart-eylem">
      <button data-e="duzenle" title="${d('Köşeleri ayarla')}">⌗</button>
      <button data-e="dondur" title="${d('Sağa döndür')}">↻</button>
      <button data-e="sil" title="${d('Çıkar')}">✕</button>
    </span>`;
}

function kartlariCiz() {
  const kutu = $('#kartlar');
  kutu.innerHTML = sayfalar.map((s, i) => `<div class="kart" data-id="${s.id}" title="${kacis(ad(s.dosya))}">${kartIcerik(s, i)}</div>`).join('');
  $('#tarama-sayac').textContent = sayfalar.length ? d('{n} sayfa', { n: sayfalar.length }) : '';
  $('#tarama-pdf').disabled = !sayfalar.length;
  $('#birak').classList.toggle('ince', arac.id === 'tara' && sayfalar.length > 0);
}

function kartGuncelle(s) {
  const el = $(`#kartlar .kart[data-id="${s.id}"]`);
  if (el) el.innerHTML = kartIcerik(s, sayfalar.indexOf(s));
}

const sayfaBul = (el) => sayfalar.find((s) => s.id === Number(el.closest('.kart').dataset.id));

$('#kartlar').addEventListener('click', (e) => {
  if (suruklendi) return;
  const kart = e.target.closest('.kart');
  if (!kart) return;
  const s = sayfaBul(kart);
  const eylem = e.target.closest('button')?.dataset.e || 'duzenle';
  if (eylem === 'sil') { sayfalar.splice(sayfalar.indexOf(s), 1); kartlariCiz(); }
  else if (eylem === 'dondur') { s.donus = (s.donus + 90) % 360; onizle(s); }
  else if (s.foto) duzenleyiciAc(s);
});

// Kartları fareyle sürükleyerek sıralama. (Tauri dosya bırakmayı yakaladığı için HTML5 sürükle-bırak kullanılamıyor.)
let suruklendi = false;
$('#kartlar').addEventListener('pointerdown', (e) => {
  const kart = e.target.closest('.kart');
  if (!kart || e.button !== 0 || e.target.closest('button')) return;
  const kutu = $('#kartlar');
  const r = kart.getBoundingClientRect();
  const bas = { x: e.clientX, y: e.clientY };
  let hayalet = null;
  suruklendi = false;

  const hareket = (ev) => {
    const dx = ev.clientX - bas.x, dy = ev.clientY - bas.y;
    if (!hayalet) {
      if (Math.hypot(dx, dy) < 6) return;
      suruklendi = true;
      hayalet = kart.cloneNode(true);
      hayalet.classList.add('hayalet');
      Object.assign(hayalet.style, { left: `${r.left}px`, top: `${r.top}px`, width: `${r.width}px`, height: `${r.height}px` });
      document.body.appendChild(hayalet);
      kart.classList.add('yer-tutucu');
    }
    hayalet.style.transform = `translate(${dx}px, ${dy}px) rotate(2deg)`;
    const hedef = document.elementsFromPoint(ev.clientX, ev.clientY).map((el) => el.closest?.('.kart')).find((k) => k && k !== kart && !k.classList.contains('hayalet'));
    if (hedef) {
      const hr = hedef.getBoundingClientRect();
      const once = ev.clientX < hr.left + hr.width / 2;
      kutu.insertBefore(kart, once ? hedef : hedef.nextSibling);
    }
    // Kenara yaklaşınca listeyi kaydır.
    const kr = kutu.getBoundingClientRect();
    if (ev.clientY > kr.bottom - 40) kutu.scrollTop += 12;
    else if (ev.clientY < kr.top + 40) kutu.scrollTop -= 12;
  };
  const birak = () => {
    window.removeEventListener('pointermove', hareket);
    window.removeEventListener('pointerup', birak);
    window.removeEventListener('pointercancel', birak);
    if (!hayalet) return;
    hayalet.remove();
    kart.classList.remove('yer-tutucu');
    const sira = [...kutu.querySelectorAll('.kart')].map((k) => Number(k.dataset.id));
    sayfalar.sort((a, b) => sira.indexOf(a.id) - sira.indexOf(b.id));
    kartlariCiz();
    // Tıklama olayı bu sürüklemenin sonunda tetiklenmesin.
    setTimeout(() => { suruklendi = false; }, 0);
  };
  // Pencereyi dinle: kart sıralanırken DOM'da yer değiştirdiği için kendisine bağlanan olaylar kopar.
  window.addEventListener('pointermove', hareket);
  window.addEventListener('pointerup', birak);
  window.addEventListener('pointercancel', birak);
});

$('#tarama-temizle').addEventListener('click', () => { sayfalar.length = 0; kartlariCiz(); });
$('#tarama-pdf').addEventListener('click', () => {
  const hazir = sayfalar.filter((s) => s.kose && !s.hata);
  if (!hazir.length) return;
  calistir(d('{n} sayfalık tarama', { n: hazir.length }), d('Sayfalar düzeltiliyor…'), () => invoke('tarama_pdf', { sayfalar: hazir.map(taramaVerisi) }));
});

// ---------- Köşe düzenleyici ----------
let duzenlenen = null;
let taslak = null; // { kose, filtre }

function duzenleyiciAc(s) {
  duzenlenen = s;
  taslak = { kose: s.kose.map((k) => [...k]), filtre: s.filtre };
  $('#d-foto').src = s.foto;
  $('#d-sonuc').src = s.onizleme || '';
  $('#duzenleyici').hidden = false;
  dFiltreCiz();
  $('#d-foto').decode().catch(() => {}).then(() => { cokgenCiz(); sonucuGuncelle(); });
}

// Sağdaki "sonuç": perspektifi düzeltilmiş ve filtrelenmiş sayfa. Köşe bırakılınca yenilenir.
let sonucIstek = 0;
async function sonucuGuncelle() {
  if (!duzenlenen) return;
  const no = ++sonucIstek;
  $('#d-sonuc-kutu').classList.add('yenileniyor');
  try {
    const url = await invoke('tarama_onizle', {
      sayfa: { dosya: duzenlenen.dosya, kose: taslak.kose, donus: duzenlenen.donus, filtre: taslak.filtre }, boyut: 1100,
    });
    if (no === sonucIstek && duzenlenen) $('#d-sonuc').src = url;
  } catch (_) {}
  if (no === sonucIstek) $('#d-sonuc-kutu').classList.remove('yenileniyor');
}

function dFiltreCiz() { $('#d-filtre').innerHTML = parcali('dFiltre', FILTRELER, taslak.filtre); }

// Öğeler bir kez oluşturulur, sonra yalnızca koordinatlar güncellenir. (innerHTML ile yeniden
// kurmak sürüklenen yuvarlağı silip sürüklemeyi ilk harekette koparıyordu.)
function cokgenCiz() {
  const svg = $('#d-svg');
  if (!svg.querySelector('polygon')) {
    svg.innerHTML = `<path class="golge"/><polygon/>${[0, 1, 2, 3].map((i) => `<circle data-i="${i}" r="11"/>`).join('')}`;
  }
  const img = $('#d-foto');
  const w = img.clientWidth, h = img.clientHeight;
  const p = taslak.kose.map(([x, y]) => [x * w, y * h]);
  svg.querySelector('.golge').setAttribute('d', `M0,0H${w}V${h}H0Z M${p.map((q) => q.join(',')).join(' L')}Z`);
  svg.querySelector('polygon').setAttribute('points', p.map((q) => q.join(',')).join(' '));
  svg.querySelectorAll('circle').forEach((c, i) => { c.setAttribute('cx', p[i][0]); c.setAttribute('cy', p[i][1]); });
}

// Sürüklerken imlecin altını 3 kat büyüten yuvarlak pencere.
function buyutecGoster(x, y, nx, ny) {
  let b = $('#d-buyutec');
  if (!b) {
    b = document.createElement('div');
    b.id = 'd-buyutec';
    b.className = 'buyutec';
    document.body.appendChild(b);
  }
  const img = $('#d-foto');
  const K = 3, R = 70;
  Object.assign(b.style, {
    display: 'block',
    left: `${x - R}px`,
    // Parmağın/imlecin üstünde dursun, köşe görünür kalsın.
    top: `${y - R * 2 - 24}px`,
    backgroundImage: `url("${img.src}")`,
    backgroundSize: `${img.clientWidth * K}px ${img.clientHeight * K}px`,
    backgroundPosition: `${R - nx * img.clientWidth * K}px ${R - ny * img.clientHeight * K}px`,
  });
}
const buyutecGizle = () => { const b = $('#d-buyutec'); if (b) b.style.display = 'none'; };

$('#d-svg').addEventListener('pointerdown', (e) => {
  const c = e.target.closest('circle');
  if (!c || e.button !== 0) return;
  e.preventDefault();
  const i = Number(c.dataset.i);
  const img = $('#d-foto');
  c.classList.add('aktif');
  const hareket = (ev) => {
    const r = img.getBoundingClientRect();
    const nx = Math.min(1, Math.max(0, (ev.clientX - r.left) / r.width));
    const ny = Math.min(1, Math.max(0, (ev.clientY - r.top) / r.height));
    taslak.kose[i] = [nx, ny];
    cokgenCiz();
    buyutecGoster(ev.clientX, ev.clientY, nx, ny);
  };
  const birak = () => {
    window.removeEventListener('pointermove', hareket);
    window.removeEventListener('pointerup', birak);
    window.removeEventListener('pointercancel', birak);
    c.classList.remove('aktif');
    buyutecGizle();
    sonucuGuncelle();
  };
  window.addEventListener('pointermove', hareket);
  window.addEventListener('pointerup', birak);
  window.addEventListener('pointercancel', birak);
});

$('#d-filtre').addEventListener('click', (e) => {
  const b = e.target.closest('button[data-v]');
  if (b) { taslak.filtre = b.dataset.v; dFiltreCiz(); sonucuGuncelle(); }
});
$('#d-otomatik').addEventListener('click', () => { taslak.kose = duzenlenen.otoKose.map((k) => [...k]); cokgenCiz(); sonucuGuncelle(); });
$('#d-tum').addEventListener('click', () => { taslak.kose = [[0, 0], [1, 0], [1, 1], [0, 1]]; cokgenCiz(); sonucuGuncelle(); });
const duzenleyiciKapat = () => { $('#duzenleyici').hidden = true; duzenlenen = null; };
$('#d-iptal').addEventListener('click', duzenleyiciKapat);
$('#d-uygula').addEventListener('click', () => {
  Object.assign(duzenlenen, { kose: taslak.kose, filtre: taslak.filtre });
  onizle(duzenlenen);
  duzenleyiciKapat();
});
window.addEventListener('resize', () => { if (duzenlenen) cokgenCiz(); });
document.addEventListener('keydown', (e) => {
  if (!duzenlenen) return;
  if (e.key === 'Escape') duzenleyiciKapat();
  if (e.key === 'Enter') $('#d-uygula').click();
});

// ---------- Metin düzenleyici (pdf.js ile çizim, Rust ile yazma) ----------
let pdfjs = null;
async function pdfjsYukle() {
  if (!pdfjs) {
    pdfjs = await import('./pdfjs/pdf.min.mjs');
    pdfjs.GlobalWorkerOptions.workerSrc = './pdfjs/pdf.worker.min.mjs';
  }
  return pdfjs;
}

// md: açık belge; degisiklik: anahtar → { sayfa, ort, zemin, x, y, boyut, renk, kalin, metin, kutu: [sol, üst, gen, yük] (ekran) }
const md = { dosya: null, belge: null, no: 0, olcek: 1, vp: null, sayfa: null, ogeler: [], degisiklik: new Map(), yeniSayac: 0 };

async function mdAc(dosya) {
  try {
    const lib = await pdfjsYukle();
    const ham = await invoke('dosya_oku', { dosya });
    const belge = await lib.getDocument({
      data: new Uint8Array(ham), cMapUrl: './pdfjs/cmaps/', cMapPacked: true, standardFontDataUrl: './pdfjs/standard_fonts/',
    }).promise;
    Object.assign(md, { dosya, belge, no: 0, degisiklik: new Map(), yeniSayac: 0 });
    $('#md-ad').textContent = ad(dosya);
    $('#md').hidden = false;
    await mdCiz();
  } catch (h) {
    const anahtar = `is-${++isSayac}`;
    isEkle(anahtar, ad(dosya), '');
    isGuncelle(anahtar, 'hata', d('Açılamadı: {h}', { h: h?.message || h }), []);
  }
}

// pdf.js aynı tuvale aynı anda iki çizimi kabul etmez. Sayfa değişirken ya da pencere boyutlanırken
// çizim sürüyorsa yeni isteği beklet; bitince bir kez daha çiz.
let cizimSuruyor = false;
let tekrarCiz = false;
async function mdCiz() {
  if (cizimSuruyor) { tekrarCiz = true; return; }
  cizimSuruyor = true;
  try {
    do {
      tekrarCiz = false;
      if (md.belge) await mdCizBir();
    } while (tekrarCiz);
  } finally {
    cizimSuruyor = false;
  }
}

async function mdCizBir() {
  const sayfa = await md.belge.getPage(md.no + 1);
  const temel = sayfa.getViewport({ scale: 1 });
  const alan = $('#md-alan');
  md.olcek = Math.min((alan.clientWidth - 48) / temel.width, 2.5);
  const vp = sayfa.getViewport({ scale: md.olcek });
  const dpr = window.devicePixelRatio || 1;
  const tuval = $('#md-tuval');
  tuval.width = Math.round(vp.width * dpr);
  tuval.height = Math.round(vp.height * dpr);
  tuval.style.width = `${vp.width}px`;
  tuval.style.height = `${vp.height}px`;
  $('#md-sayfa').style.width = `${vp.width}px`;
  $('#md-sayfa').style.height = `${vp.height}px`;
  await sayfa.render({ canvasContext: tuval.getContext('2d', { willReadFrequently: true }), viewport: sayfa.getViewport({ scale: md.olcek * dpr }) }).promise;
  const icerik = await sayfa.getTextContent();
  Object.assign(md, { sayfa, vp, ogeler: satirlaraBirlestir(icerik.items) });
  $('#md-no').textContent = `${md.no + 1} / ${md.belge.numPages}`;
  $('#md-onceki').disabled = md.no === 0;
  $('#md-sonraki').disabled = md.no >= md.belge.numPages - 1;
  mdKatmanCiz();
}

/// pdf.js bir satırı çoğu zaman birkaç parça verir ("Monthly Report — " + "Neighbourhood Library").
/// Kullanıcı bir yere tıklayınca bütün satırı değiştirmek ister; aynı taban çizgisinde, aynı boyutta
/// ve aralarında küçük boşluk olan parçalar tek öğe olarak birleştirilir.
function satirlaraBirlestir(ogeler) {
  const sonuc = [];
  for (const o of ogeler) {
    const t = o.transform;
    const once = sonuc[sonuc.length - 1];
    if (once && o.str !== undefined) {
      const p = once.transform;
      const boyut = p[3];
      const bosluk = t[4] - (p[4] + once.width);
      const ayniSatir = Math.abs(t[5] - p[5]) < boyut * 0.3 && Math.abs(t[3] - p[3]) < 0.5 && Math.abs(t[1]) < 1e-3 && Math.abs(p[1]) < 1e-3;
      if (ayniSatir && bosluk > -boyut * 0.3 && bosluk < boyut * 0.8) {
        const araBosluk = bosluk > boyut * 0.15 && !once.str.endsWith(' ') && !o.str.startsWith(' ') ? ' ' : '';
        once.str += araBosluk + o.str;
        once.width = t[4] + o.width - p[4];
        continue;
      }
    }
    if (o.str !== undefined) sonuc.push({ str: o.str, transform: [...t], width: o.width, fontName: o.fontName });
  }
  // Sondaki boşluklar kutuyu gereksiz genişletmesin.
  for (const o of sonuc) o.str = o.str.replace(/\s+$/, '');
  return sonuc;
}

/// Metin öğesinin PDF ve ekran kutusu. Döndürülmüş metinler (t[1], t[2] ≠ 0) düzenlenemez.
function ogeKutusu(o) {
  const t = o.transform;
  if (Math.abs(t[1]) > 1e-3 || Math.abs(t[2]) > 1e-3 || t[3] <= 0) return null;
  const boyut = t[3], x = t[4], y = t[5], gen = o.width;
  const [sol, ust] = md.vp.convertToViewportPoint(x, y + boyut * 0.92);
  const [sag, alt] = md.vp.convertToViewportPoint(x + gen, y - boyut * 0.24);
  return { boyut, x, y, gen, ekran: [Math.min(sol, sag), Math.min(ust, alt), Math.abs(sag - sol), Math.abs(alt - ust)] };
}

function mdKatmanCiz() {
  const katman = $('#md-katman');
  katman.innerHTML = '';
  md.ogeler.forEach((o, i) => {
    if (!o.str || !o.str.trim() || md.degisiklik.has(`${md.no}:${i}`)) return;
    const k = ogeKutusu(o);
    if (!k) return;
    const el = document.createElement('div');
    el.className = 'kutu';
    el.dataset.i = i;
    Object.assign(el.style, { left: `${k.ekran[0]}px`, top: `${k.ekran[1]}px`, width: `${k.ekran[2]}px`, height: `${k.ekran[3]}px` });
    katman.appendChild(el);
  });
  for (const [anahtar, g] of md.degisiklik) if (g.sayfa === md.no) katman.appendChild(degisenEl(anahtar, g));
  const n = md.degisiklik.size;
  $('#md-sayac').textContent = n ? d('{n} değişiklik', { n }) : '';
  $('#md-kaydet').disabled = n === 0;
}

const css = (c) => `rgb(${c.map((v) => Math.round(v * 255)).join(',')})`;

function degisenEl(anahtar, g) {
  const el = document.createElement('div');
  el.className = 'degisti';
  el.dataset.anahtar = anahtar;
  const [sol, ust, gen, yuk] = g.kutu;
  Object.assign(el.style, {
    left: `${sol}px`, top: `${ust}px`, minWidth: `${gen}px`, height: `${yuk}px`,
    background: g.ort ? css(g.zemin) : 'transparent', color: css(g.renk),
    fontSize: `${g.boyut * md.olcek}px`, fontWeight: g.kalin ? '700' : '400',
  });
  el.textContent = g.metin;
  return el;
}

/// Kutudaki piksellerden zemin rengi (kenarlarda en sık renk) ve yazı rengi (zeminden en uzak renk).
function renkleriOlc([sol, ust, gen, yuk]) {
  const dpr = window.devicePixelRatio || 1;
  const ctx = $('#md-tuval').getContext('2d', { willReadFrequently: true });
  const x = Math.max(0, Math.floor(sol * dpr) - 2), y = Math.max(0, Math.floor(ust * dpr) - 2);
  const w = Math.max(1, Math.ceil(gen * dpr) + 4), h = Math.max(1, Math.ceil(yuk * dpr) + 4);
  const v = ctx.getImageData(x, y, w, h).data;
  // Kenar piksellerini kaba renk kümelerine ayır; en kalabalık kümenin gerçek ortalaması zemindir.
  const say = new Map();
  const ekle = (i) => {
    const k = `${v[i] >> 3},${v[i + 1] >> 3},${v[i + 2] >> 3}`;
    const t = say.get(k) || [0, 0, 0, 0];
    t[0] += v[i]; t[1] += v[i + 1]; t[2] += v[i + 2]; t[3]++;
    say.set(k, t);
  };
  for (let i = 0; i < w; i++) { ekle(i * 4); ekle(((h - 1) * w + i) * 4); }
  for (let j = 0; j < h; j++) { ekle(j * w * 4); ekle((j * w + w - 1) * 4); }
  const [r, g, b, n] = [...say.values()].sort((a, b) => b[3] - a[3])[0];
  const zeminK = [r, g, b].map((c) => c / n / 255);
  let enUzak = 0, yazi = [0, 0, 0];
  for (let i = 0; i < v.length; i += 4) {
    const c = [v[i] / 255, v[i + 1] / 255, v[i + 2] / 255];
    const uz = (c[0] - zeminK[0]) ** 2 + (c[1] - zeminK[1]) ** 2 + (c[2] - zeminK[2]) ** 2;
    if (uz > enUzak) { enUzak = uz; yazi = c; }
  }
  return { zemin: zeminK, renk: enUzak > 0.05 ? yazi : [0, 0, 0] };
}

function kalinMi(o) {
  try {
    const f = md.sayfa.commonObjs.get(o.fontName);
    return /bold|black|heavy|semibold|demi/i.test(f?.name || '');
  } catch (_) { return false; }
}

/// Yazma kutusunu açar; onaylanınca `bitir(metin)` çağrılır.
function yazmaAc({ kutu, boyut, kalin, zemin, renk, metin, ort }, bitir) {
  const katman = $('#md-katman');
  const el = document.createElement('div');
  el.className = 'yaziyor';
  el.contentEditable = 'plaintext-only';
  el.spellcheck = false;
  const [sol, ust, gen, yuk] = kutu;
  Object.assign(el.style, {
    left: `${sol}px`, top: `${ust}px`, minWidth: `${Math.max(gen, 12)}px`, height: `${yuk}px`,
    background: ort ? css(zemin) : 'rgba(255,255,255,.6)', color: css(renk),
    fontSize: `${boyut * md.olcek}px`, fontWeight: kalin ? '700' : '400',
  });
  el.textContent = metin;
  katman.appendChild(el);
  el.focus();
  const sec = document.createRange();
  sec.selectNodeContents(el);
  getSelection().removeAllRanges();
  getSelection().addRange(sec);
  let bitti = false;
  const kapat = (onayla) => {
    if (bitti) return;
    bitti = true;
    const yeni = el.textContent.replace(/\n/g, ' ');
    el.remove();
    if (onayla) bitir(yeni);
    mdKatmanCiz();
  };
  el.addEventListener('keydown', (e) => {
    e.stopPropagation();
    if (e.key === 'Enter') { e.preventDefault(); kapat(true); }
    if (e.key === 'Escape') { e.preventDefault(); kapat(false); }
  });
  el.addEventListener('blur', () => kapat(true));
  el.addEventListener('pointerdown', (e) => e.stopPropagation());
}

$('#md-katman').addEventListener('pointerdown', (e) => {
  if (e.button !== 0) return;
  const katman = $('#md-katman');
  const kutuEl = e.target.closest('.kutu');
  const degisenEl_ = e.target.closest('.degisti');
  e.preventDefault();

  if (kutuEl) {
    // Var olan yazıyı değiştir.
    const i = Number(kutuEl.dataset.i);
    const o = md.ogeler[i];
    const k = ogeKutusu(o);
    const { zemin, renk } = renkleriOlc(k.ekran);
    const taban = {
      sayfa: md.no, ort: [k.x - 0.5, k.y - k.boyut * 0.24, k.gen + 1, k.boyut * 1.18], zemin,
      x: k.x, y: k.y, boyut: k.boyut, renk, kalin: kalinMi(o), kutu: k.ekran,
    };
    yazmaAc({ ...taban, metin: o.str }, (metin) => {
      if (metin === o.str) return;
      md.degisiklik.set(`${md.no}:${i}`, { ...taban, metin });
    });
  } else if (degisenEl_) {
    // Daha önce değiştirilmiş yazıyı yeniden düzenle; boş bırakılırsa yalnızca örtü kalır.
    const anahtar = degisenEl_.dataset.anahtar;
    const g = md.degisiklik.get(anahtar);
    md.degisiklik.delete(anahtar);
    mdKatmanCiz();
    yazmaAc(g, (metin) => md.degisiklik.set(anahtar, { ...g, metin }));
    // Vazgeçilirse eski değişiklik geri gelsin.
    const el = $('#md-katman .yaziyor');
    el?.addEventListener('keydown', (ev) => { if (ev.key === 'Escape') { md.degisiklik.set(anahtar, g); } }, { capture: true });
  } else if (e.target === katman) {
    // Boş yere yeni yazı.
    const r = katman.getBoundingClientRect();
    const sx = e.clientX - r.left, sy = e.clientY - r.top;
    const boyut = 12;
    const [px, py] = md.vp.convertToPdfPoint(sx, sy + boyut * md.olcek * 0.8);
    const anahtar = `${md.no}:y${++md.yeniSayac}`;
    const g = { sayfa: md.no, ort: null, zemin: [1, 1, 1], x: px, y: py, boyut, renk: [0, 0, 0], kalin: false,
      kutu: [sx, sy, 40, boyut * md.olcek * 1.18] };
    // Tıklama olayı bitince odaklan, yoksa odak hemen kaybolur.
    setTimeout(() => yazmaAc({ ...g, metin: '' }, (metin) => { if (metin.trim()) md.degisiklik.set(anahtar, { ...g, metin }); }), 0);
  }
});

$('#md-onceki').addEventListener('click', () => { if (md.no > 0) { md.no--; mdCiz(); } });
$('#md-sonraki').addEventListener('click', () => { if (md.no < md.belge.numPages - 1) { md.no++; mdCiz(); } });
const mdKapat = () => { $('#md').hidden = true; md.belge?.destroy(); md.belge = null; };
$('#md-kapat').addEventListener('click', () => {
  if (md.degisiklik.size && !confirm(d('Kaydedilmemiş değişiklikler silinsin mi?'))) return;
  mdKapat();
});
$('#md-kaydet').addEventListener('click', () => {
  const degisiklikler = [...md.degisiklik.values()].map(({ kutu, ...g }) => g);
  const dosya = md.dosya;
  calistir(ad(dosya), d('{n} değişiklik yazılıyor…', { n: degisiklikler.length }), () => invoke('metin_kaydet', { dosya, degisiklikler }));
  mdKapat();
});
document.addEventListener('keydown', (e) => {
  if ($('#md').hidden || document.activeElement?.classList.contains('yaziyor')) return;
  if (e.key === 'ArrowRight' || e.key === 'PageDown') $('#md-sonraki').click();
  if (e.key === 'ArrowLeft' || e.key === 'PageUp') $('#md-onceki').click();
});
window.addEventListener('resize', () => { if (md.belge) mdCiz(); });

// Dil: Rust'a bildir (dosya ekleri ve iletiler için) ve TR/EN düğmesi.
invoke('dil_ayarla', { en: DIL === 'en' }).catch(() => {});
for (const b of document.querySelectorAll('#dil-sec button')) {
  b.classList.toggle('secili', b.dataset.dil === DIL);
  b.addEventListener('click', () => {
    if (b.dataset.dil === DIL) return;
    try { localStorage.setItem('dil', b.dataset.dil); } catch (_) {}
    location.reload();
  });
}

let ilkArac = ARACLAR[0];
try { ilkArac = ARACLAR.find((a) => a.id === localStorage.getItem('arac')) || ilkArac; } catch (_) {}
aracSec(ilkArac);
