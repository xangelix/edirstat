# Menü Çubuğu
file = Dosya
view = Görünüm
help = Yardım

# Menü İşlemleri
new-scan = 📁 Yeni tarama
save-snapshot = 💾 Anlık görüntüyü kaydet
load-snapshot = 📖 Anlık görüntü yükle

# Menü Durumu
idle = Boşta

# Görünüm Menüsü
monospace-paths = 🅰 Eş aralıklı yollar
highlight-duplicates = ✨ Yinelenenleri vurgula
treemap-borders = 🔳 Treemap kenarlıkları
treemap-style =  Treemap stili
treemap-style-vertical = Dikey geçiş
treemap-style-offset-vertical = Kaydırılmış dikey geçiş
treemap-style-diagonal = Çapraz geçiş
treemap-style-cushion = Yastık gölgelendirmesi
deletion-confirmation = 🗑 Silme onayı
trash-confirmation = ♻ Çöp kutusu onayı
time-format = 🕒 Saat biçimi
language = 💬 Dil
layout-mode = Yerleşim modu:
classic-layout = Klasik yerleşim
windirstat-layout = WinDirStat yerleşimi
vis-mode-treemap = 📊 Treemap
vis-mode-plots = 📈 Grafikler
select-plot-label = Grafik seç:
vis-mode-deduplicator = 👥 Yinelenen dosya bulucu
search-filter-label = 🔍 Filtrele:

# Panel Kontrolleri
toggle-left-panel = { $collapsed ->
    [true] ▶ Sol paneli göster (F9)
   *[false] ◀ Sol paneli gizle (F9)
}

toggle-right-panel = { $collapsed ->
    [true] { $is_classic ->
        [true] ◀ Sağ paneli göster (F11)
       *[false] ▶ Uzantılar panelini göster (F11)
    }
   *[false] { $is_classic ->
        [true] ▶ Sağ paneli gizle (F11)
       *[false] ◀ Uzantılar panelini gizle (F11)
    }
}

collapse-all = ⏏ Tümünü daralt
about = ℹ Hakkında
web-not-available = Bu özellik web sürümünde kullanılamaz

# Durum Göstergeleri
scanning-disk = Disk taranıyor...
scan-complete = Tarama tamamlandı
scan-cancelled = Tarama iptal edildi
path-label = Yol: { $path }
worker-threads = ⚡ { $count } çalışma iş parçacığı
worker-threads-hover = Dizin taraması için ayrılan paralel, iş çalan CPU çekirdeği sayısı.

# Alt İstatistikler
directories-count = 📁 Dizinler: { $count }
files-count = 📄 Dosyalar: { $count }
total-size = 💾 Toplam boyut: { $size }
elapsed-time = ⏱ Süre: { $time }
scan-speed = ⚡ Hız: { $speed }/sn

# Seçim Bilgisi
selection-path = Seçim: { $path }
selection-items = Seçim: { $count ->
   *[other] { $count } öğe
}

# Grafik Türleri
plot-size-distribution = 📊 Dosya boyutu dağılımı
plot-age-size = 🌌 Dosya yaşı ve boyutu
plot-dir-composition = 🍰 Dizin bileşimi
plot-extension-boxplot = 📦 Uzantıya göre dosya boyutları
plot-temporal-timeline = ⏱ Bağlantılı zaman çizelgeleri
plot-deduplicator-waste = 👥 Uzantıya göre yinelenen alan kaybı

# Yinelenen Bulucu
dedup-desc = Kriptografik olarak güvenli BLAKE3 özetleriyle birebir aynı dosyaları bulun ve güvenle kaldırın.
dedup-how-it-works = ℹ Nasıl çalışır
dedup-min-size = En küçük dosya boyutu:
dedup-ignore-system = Sistem dosyalarını yoksay
dedup-ignore-hidden = Gizli dosyaları yoksay
dedup-start-scan = ⚡ Yinelenen taramasını başlat
dedup-scan-first = Önce bir dizin tarayın.
dedup-cancelled-msg = Tarama iptal edildi. Yinelenenleri bulmak için yeni bir tarama başlatın.
dedup-analyzing = Dosyalar analiz ediliyor...
dedup-no-duplicates = Yinelenen grup bulunamadı. En küçük dosya boyutunu azaltmayı veya farklı bir klasör taramayı deneyin.
no-permission = İzin yok
hardlink-badge = Sabit bağlantı
dedup-select-items = 🎯 Öğeleri seç...
dedup-select-all-but-oldest = 🎯 En eski hariç tümü
dedup-select-all-but-newest = 🎯 En yeni hariç tümü
dedup-select-all-but-shortest = 🎯 En kısa yol hariç tümü
dedup-select-all-but-rootmost = 🎯 Kök dizine en yakın hariç tümü
dedup-select-all-but-longest = 🎯 En uzun yol hariç tümü
dedup-pref-dir-pattern = Tercih edilen dizin deseni:
dedup-select-all-but-pref = 🎯 Tercih edilen dizin hariç tümü
dedup-clear-selection = ❌ Seçimi temizle
dedup-link-menu = 🔗 Bağla... ({ $count } dosya)
dedup-link-menu-disabled = 🔗 Bağla... (0 dosya)
dedup-link-hardlinks = 🔗 Seçilenleri sabit bağlantılarla değiştir
dedup-link-softlinks = 🔗 Seçilenleri sembolik bağlantılarla değiştir
dedup-remove-menu = 🗑 Kaldır... ({ $count } dosya, { $size })
dedup-remove-menu-disabled = 🗑 Kaldır... (0 dosya)
dedup-remove-trash = ♻ Seçilenleri çöp kutusuna taşı
dedup-remove-delete = 🗑 Seçilenleri kalıcı olarak sil
dedup-warning-title = ⚠ VERİ KAYBI UYARISI
dedup-warning-desc = { $count ->
   *[other] { $count } dosyanın tüm kopyaları siliniyor
}
dedup-warning-no-original = Özgün kopya kalmayacak:
dedup-warning-details = Aşağıdaki dosyaların özgün ve yinelenen tüm kopyalarını seçtiniz. Silme işlemi kalıcı veri kaybına yol açabilir:
dedup-cancel-hover = Taramayı iptal etmek için tıklayın
scan-cancel-hover = Taramayı iptal etmek için tıklayın
dedup-current-label = Geçerli
dedup-phase1-size = Aşama 1/7: Taranan tüm dosyalar boyuta göre gruplanıyor...
dedup-phase1-filter = Aşama 1/7: Yinelenen adaylarından hariç tutulanlar ayıklanıyor...
dedup-phase2-prefix = Aşama 2/7: Dosya başlangıçları özetleniyor (ilk 4 KB)...
dedup-phase3-midpoint = Aşama 3/7: Dosya orta noktaları özetleniyor...
dedup-phase4-suffix = Aşama 4/7: Dosya sonları özetleniyor...
dedup-phase5-multirange = Aşama 5/7: Büyük dosyalarda çok aralıklı özetleme yapılıyor...
dedup-phase6-full = Aşama 6/7: Kalan adaylar için tam BLAKE3 özeti hesaplanıyor...
dedup-phase7-validation = Aşama 7/7: Son zaman damgası doğrulaması yapılıyor...
dedup-phase-finished = { $duration } içinde tamamlandı. { $count } yinelenen grup bulundu. Geri kazanılabilir alan: { $space }
dedup-scan-cancelled-with-error = Tarama iptal edildi: { $error }

# Yinelenen Tablosu
dedup-hdr-checkbox = [     ]
dedup-hdr-filename = Dosya adı
dedup-hdr-directory = Üst dizin
dedup-hdr-size = Boyut
dedup-hdr-reclaimable = Geri kazanılabilir
dedup-hdr-created = Oluşturulma
dedup-hdr-modified = Değiştirilme
dedup-copies-selected = ({ $count ->
   *[other] { $count } kopya seçildi
})

# Gezgin Ayrıntıları
explorer-details-header = ℹ Ayrıntılar
explorer-deselect-hover = Öğelerin seçimini kaldır
explorer-deselect-single-hover = Öğenin seçimini kaldır
explorer-selected-items-count = { $count ->
   *[other] { $count } öğe seçildi
}
explorer-total-size = Toplam boyut: { $size }
explorer-files = Dosyalar: { $count }
explorer-directories = Dizinler: { $count }
explorer-actions-title = İşlemler
explorer-actions-operations = İşlemler:
explorer-action-refresh-hover = Seçili tüm dizin alt ağaçlarını yenile
explorer-grid-type = Tür:
explorer-grid-size = Boyut:
explorer-grid-bytes = Bayt:
explorer-grid-items = Öğeler:
explorer-grid-files = Dosyalar:
explorer-grid-subdirs = Alt dizinler:
explorer-grid-user = Kullanıcı:
explorer-grid-group = Grup:
explorer-grid-permissions = İzinler:
explorer-grid-path = Tam yol:

# Gezgin Tür Adları
type-symlink = Sembolik bağlantı
type-directory = Dizin
type-file = Dosya

# Gezgin İşlemleri
explorer-action-copy-path = 📋 Yolu kopyala
explorer-action-open-file = 📄 Dosyayı aç
explorer-action-open-manager = 🗁 Dosya yöneticisinde aç
explorer-action-refresh-subtree = 🔄 Alt ağacı yenile
explorer-action-move-trash = ♻ Çöp kutusuna taşı
explorer-action-delete-permanently = 🗑 Kalıcı olarak sil
explorer-action-refresh-directory = 🔄 Dizini yenile

# Gezgin Boş Durumu
explorer-empty-state = Disk kullanımını incelemek için “Yeni tarama”yı seçin.
choose-an-option = Bir seçenek belirleyin
web-viewer = Web görüntüleyici
load-demo = 👁 Örnek anlık görüntüyü yükle
placeholder-treemap = Taranan dosya sistemi burada treemap olarak görüntülenecek.
placeholder-plots = Taranan dosya sistemi burada grafik olarak görüntülenecek.

# Uzantılar Paneli
extensions-header = 📂 Uzantılar
extensions-empty = Henüz istatistik toplanmadı.
extensions-hover-files = Dosyalar: { $count }

# Bağlam İşlemleri
op-up-one-level = Bir üst düzeye çık
op-refresh-entire-scan = Tüm taramayı yenile
op-refresh-directory = Dizini yenile
op-open-file = Dosyayı aç
op-open-file-manager = Dosya yöneticisinde aç
op-open-terminal = Burada terminal aç
op-copy-path = Yolu kopyala
op-copy-name = Adı kopyala
op-move-trash = Çöp kutusuna taşı
op-permanently-delete = Kalıcı olarak sil

# Bildirimler
toast-already-root = Zaten kök düzeydesiniz
toast-navigated-up = Bir üst düzeye çıkıldı
toast-refreshing-scan = Tüm tarama yenileniyor...
toast-refreshing-dir = Seçili dizin veya dizinler yenileniyor...
toast-opened-file = Açıldı: { $path }
toast-failed-open-file = Dosya açılamadı: { $error }
toast-opened-manager = Dosya yöneticisinde açıldı: { $path }
toast-failed-open-manager = Dosya yöneticisi açılamadı: { $error }
toast-opened-terminal = Terminal açıldı: { $path }
toast-failed-open-terminal = Terminal açılamadı: { $error }
toast-copied-paths = { $count ->
   *[other] { $count } yol panoya kopyalandı
}
toast-copied-names = { $count ->
   *[other] { $count } ad panoya kopyalandı
}

# İletişim Pencereleri
modal-remember-confirmation = Tüm sonraki dosya ve dizinler için onayı hatırla
modal-process-multiple = { $count } yinelenen dosya/öğe üzerinde işlem yapmak üzeresiniz:
modal-process-single = Aşağıdaki yol üzerinde işlem yapmak üzeresiniz:

# Silme / Çöp Kutusu / Bağlama Onayları
modal-delete-title = ⚠ KALICI SİLME UYARISI
modal-delete-header = ⚠ Kalıcı silme uyarısı!
modal-delete-info = Toplam boyut: { $size }
modal-delete-warning = Bu işlem özyinelemeli silmedir. Seçili yol veya yolların altındaki tüm dosyalar, klasörler ve alt dizinler geri dönüşüm kutusu atlanarak kalıcı biçimde silinecek ve geri getirilemeyecektir.
modal-delete-checkbox = Dosyaların kalıcı olarak silineceğini ve geri getirilemeyeceğini anlıyorum.
modal-delete-confirm = 🗑 Evet, kalıcı olarak sil

modal-trash-title = ♻ ÇÖP KUTUSUNA TAŞI
modal-trash-header = ♻ Çöp kutusuna taşı
modal-trash-info = Toplam boyut: { $size }
modal-trash-warning = Seçili yol veya yollar ve tüm içerikleri, daha sonra geri yüklenebilecek ya da kalıcı olarak silinebilecek sistem geri dönüşüm kutusuna taşınacak.
modal-trash-checkbox = Bunu çöp kutusuna taşımak istediğimi onaylıyorum.
modal-trash-confirm = ♻ Evet, çöp kutusuna taşı

modal-delete-duplicates-title = ⚠ KALICI YİNELENEN SİLME UYARISI
modal-delete-duplicates-header = ⚠ Kalıcı yinelenen silme uyarısı!
modal-delete-duplicates-info = Geri kazanılacak toplam alan: { $size }
modal-delete-duplicates-warning = Seçili tüm dosyalar geri dönüşüm kutusu atlanarak kalıcı biçimde silinecek ve geri getirilemeyecektir.
modal-delete-duplicates-checkbox = Dosyaların kalıcı olarak silineceğini ve geri getirilemeyeceğini anlıyorum.
modal-delete-duplicates-confirm = 🗑 Evet, seçilenleri kalıcı olarak sil

modal-trash-duplicates-title = ♻ YİNELENENLERİ ÇÖP KUTUSUNA TAŞI
modal-trash-duplicates-header = ♻ Yinelenenleri çöp kutusuna taşı
modal-trash-duplicates-info = Geri kazanılacak toplam alan: { $size }
modal-trash-duplicates-warning = Seçili tüm dosyalar geri dönüşüm kutusuna taşınacak.
modal-trash-duplicates-checkbox = Bunları çöp kutusuna taşımak istediğimi onaylıyorum.
modal-trash-duplicates-confirm = ♻ Evet, seçilenleri çöp kutusuna taşı

modal-hardlink-duplicates-title = 🔗 YİNELENENLERİ SABİT BAĞLANTILARLA DEĞİŞTİR
modal-hardlink-duplicates-header = 🔗 Yinelenenleri sabit bağlantılarla değiştir
modal-hardlink-duplicates-info = İşlem yapılacak toplam dosya: { $count }. Toplam sanal boyut: { $size }
modal-hardlink-duplicates-warning = Bu işlem seçili yinelenen dosyaları siler ve her grupta kalan özgün dosyayı işaret eden dosya sistemi düzeyindeki sabit bağlantılarla değiştirir. Dosyalar görünür kalırken kullanılan fiziksel alan azalır.
modal-hardlink-duplicates-checkbox = Seçili dosyaları sabit bağlantılarla değiştirmek istediğimi onaylıyorum.
modal-hardlink-duplicates-confirm = 🔗 Evet, sabit bağlantılarla değiştir

modal-softlink-duplicates-title = 🔗 YİNELENENLERİ SEMBOLİK BAĞLANTILARLA DEĞİŞTİR
modal-softlink-duplicates-header = 🔗 Yinelenenleri sembolik bağlantılarla değiştir
modal-softlink-duplicates-info = İşlem yapılacak toplam dosya: { $count }. Toplam sanal boyut: { $size }
modal-softlink-duplicates-warning = Bu işlem seçili yinelenen dosyaları siler ve her grupta kalan özgün dosyayı işaret eden dosya sistemi düzeyindeki sembolik bağlantılarla değiştirir. Dosyalar görünür kalırken kullanılan fiziksel alan azalır.
modal-softlink-duplicates-checkbox = Seçili dosyaları sembolik bağlantılarla değiştirmek istediğimi onaylıyorum.
modal-softlink-duplicates-confirm = 🔗 Evet, sembolik bağlantılarla değiştir

# Yol Bulunamadı Penceresi
modal-path-not-exist-title = ❌ Yol mevcut değil!
modal-path-not-exist-msg = Hata: Silmeye çalıştığınız yol diskte bulunmuyor.
modal-close-btn = Kapat
modal-details-label = Ayrıntılar:
modal-cancel-btn = İptal

# Yönetici Yetkisi Önerisi
modal-elevation-title = ⚠ Yönetici yetkisi önerilir
modal-elevation-desc = eDirStat varsayılan olarak standart kullanıcı izinleriyle çalışır. Ancak Windows, fiziksel diske ham erişimi yönetici hesaplarıyla sınırlar.
modal-elevation-mft-disabled = Windows NTFS MFT sürücüsü devre dışı
modal-elevation-mft-desc = Yönetici yetkisi olmadan, diske doğrudan erişen MFT tarayıcısı başlatılamaz. Dosya analizi standart tarama sürücüsünü kullanır; bu da tarama performansını 20 kata kadar azaltabilir.
modal-elevation-relaunch-prompt = Uygulamayı şimdi yönetici yetkileriyle yeniden başlatmak ister misiniz?
modal-elevation-continue-std = Standart kullanıcı olarak devam et
modal-elevation-relaunch-btn = 🛡 Yönetici olarak yeniden başlat

# Hakkında Penceresi
modal-about-title = ℹ eDirStat hakkında
modal-about-author = Geliştiren: Cody Wyatt Neiman (xangelix) <neiman@cody.to>
modal-about-desc1 = Rust ile geliştirilmiş yüksek performanslı disk alanı analiz ve yinelenen bulma aracı.
modal-about-desc2 = Paralel iş çalan dizin taraması, sıkıştırılmış anlık görüntüler, sıfır ayrıştırmalı yerleşim geri yükleme ve duyarlı etkileşimli treemap özelliklerini sunar.
modal-about-desc3 = Yerleşik yinelenen bulucu, özdeş dosya gruplarını güvenle ayırmak, geri kazanılabilir alanı hesaplamak ve sistem düzeyi sabit bağlantıları korumak için çok aşamalı kriptografik özetleme hattı kullanır.
modal-about-licenses-btn = Açık kaynak lisanslarını görüntüle
modal-about-version = v{ $version }

# Yinelenen Bulucu Açıklaması
modal-how-dedup-title = ℹ Yinelenen bulucu nasıl çalışır
modal-how-dedup-desc1 = Bu sistem, her dosyanın baytlarını yavaş ve ikili O(N²) karşılaştırmalarla okumak yerine, aynı içeriği güvenli ve verimli biçimde tanımlamak için yüksek düzeyde optimize edilmiş 7 aşamalı bir hat kullanır.
modal-how-dedup-pipeline-title = 7 aşamalı süreç:
modal-how-dedup-why-title = Bu neden yeterli?
modal-how-dedup-why-desc1 = Çok aşamalı filtre; yalnızca boyutu, başlangıcı, orta noktası, sonu ve dağıtılmış blok örnekleri aynı olan dosyaların tamamının okunmasını sağlar. Son aşamada 256 bit BLAKE3 kriptografik özeti karşılaştırılır; bu da yavaş ikili bayt karşılaştırmalarına gerek kalmadan çok güçlü bir özdeşlik kanıtı sunar.

# Yinelenen Bulucu Aşamaları
modal-how-dedup-step1-title = 1. Boyuta göre ayırma
modal-how-dedup-step1-desc = Dosyalar bayt cinsinden kesin boyutlarına göre gruplanır. Eşsiz boyuta sahip dosyalar, disk G/Ç işlemi yapılmadan hemen elenir.
modal-how-dedup-step2-title = 2. Başlangıç özeti
modal-how-dedup-step2-desc = Kalan adayların ilk 4 KB bölümü özetlenir. Bu işlem, başlığı veya meta veri biçimi farklı dosyaları hızla eler.
modal-how-dedup-step3-title = 3. Orta nokta özeti
modal-how-dedup-step3-desc = Kalan dosyaların merkezinden alınan 4 KB blok özetlenir; böylece iç yapısal farklılıklar yakalanır.
modal-how-dedup-step4-title = 4. Son bölüm özeti
modal-how-dedup-step4-desc = Verinin son 4 KB bölümü özetlenir. Bu yöntem, sondaki içerik veya meta veri farklarını belirlemede etkilidir.
modal-how-dedup-step5-title = 5. Çok aralıklı özetleme
modal-how-dedup-step5-desc = Büyük dosyalarda (100 MB üzeri), tüm dosyayı okumadan içerik tutarlılığını doğrulamak için dosya boyunca düzenli blok örnekleri alınır.
modal-how-dedup-step6-title = 6. Tam BLAKE3 özeti
modal-how-dedup-step6-desc = Kalan adaylar için tam BLAKE3 kriptografik özeti hesaplanır. 256 bitlik alanın yüksek çakışma direnci sayesinde eşleşen özetler, dosyaların farklı olma olasılığını ihmal edilebilir düzeye indirir.
modal-how-dedup-step7-title = 7. Zaman damgası doğrulaması
modal-how-dedup-step7-desc = Herhangi bir yinelenen işlemi gösterilmeden veya yürütülmeden hemen önce, taramadan sonra oluşmuş değişikliklere karşı dosyaların disk üzerindeki zaman damgaları doğrulanır.

# Açık Kaynak Lisansları
modal-licenses-title = 📜 Açık kaynak lisansları
modal-licenses-desc = Bu uygulamada aşağıdaki üçüncü taraf kütüphaneler ve Rust paketleri kullanılmaktadır:

# İşlem Penceresi
modal-processing-title = ⏳ İşleniyor...
modal-processing-deletion = Dosyalar ve dizinler siliniyor...
modal-processing-trash = Dosyalar ve dizinler çöp kutusuna taşınıyor...
modal-processing-hardlink = Yinelenenler sabit bağlantılarla değiştiriliyor...
modal-processing-softlink = Yinelenenler sembolik bağlantılarla değiştiriliyor...

# Gezgin Sütun Başlıkları
explorer-hdr-name = Ad
explorer-hdr-percentage = Yüzde
explorer-hdr-size = Boyut
explorer-hdr-items = Öğeler
explorer-hdr-files = Dosyalar
explorer-hdr-subdirs = Alt dizinler
explorer-hdr-created = Oluşturulma
explorer-hdr-modified = Değiştirilme

# Güncelleme Denetleyicisi
update-checking = Güncellemeler denetleniyor...
update-available = Yeni sürüm { $version } kullanılabilir!
update-up-to-date = Güncelsiniz
update-failed = Güncelleme denetimi başarısız oldu: { $error }

# Temalar
theme = 🎨 Tema
theme-dark = Koyu
theme-high-contrast = Yüksek kontrast
theme-light = Açık
theme-system = Sistem

# Yeni Tarama Seçenekleri
modal-scan-options-title = Yeni tarama seçenekleri
modal-scan-options-header = Yeni tarama başlat
modal-scan-options-path-label = Taranacak dizin yolu:
modal-scan-options-paste-tooltip = Panodan yapıştır
modal-scan-options-browse-tooltip = Klasöre göz at...
modal-scan-options-scan-btn = Tara
modal-scan-options-cancel-btn = İptal
modal-scan-options-same-filesystem = Taramayı aynı dosya sistemi/birimle sınırla
modal-scan-options-drives-header = 💽 Depolama sürücüleri ve birimler
modal-scan-options-refresh-tooltip = Depolama sürücülerini yenile
modal-scan-options-root-system = Kök sistem
modal-scan-options-selected-badge = ✅ Seçili
modal-scan-options-free-of = { $total } alanın { $free } kadarı boş
