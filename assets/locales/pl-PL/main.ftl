# Menu Bar Dropdowns
file = Plik
view = Widok
help = Pomoc

# Menu Bar Actions
scan-directory = 📁 Skanuj katalog
save-snapshot = 💾 Zapisz migawkę
load-snapshot = 📖 Wczytaj migawkę

# Menu Bar Status
idle = Bezczynny

# View Menu Options
monospace-paths = Ścieżki o stałej szerokości
highlight-duplicates = ✨ Wyróżnij duplikaty
deletion-confirmation = 🗑 Potwierdzenie usuwania
trash-confirmation = ♻ Potwierdzenie przenoszenia do kosza
time-format = 🕒 Format czasu
language = 💬 Język
layout-mode = Tryb układu:
classic-layout = Klasyczny układ
windirstat-layout = Układ WinDirStat
vis-mode-treemap = 📊 Treemap
vis-mode-plots = 📈 Wykresy
select-plot-label = Wybierz wykres:
vis-mode-deduplicator = 👥 Wyszukiwanie duplikatów
search-filter-label = 🔍 Filtruj:

# Panel Toggles
toggle-left-panel = { $collapsed ->
    [true] ▶ Pokaż lewy panel (F9)
   *[false] ◀ Ukryj lewy panel (F9)
}

toggle-right-panel = { $collapsed ->
    [true] { $is_classic ->
        [true] ◀ Pokaż prawy panel (F11)
       *[false] ▶ Pokaż panel rozszerzeń (F11)
    }
   *[false] { $is_classic ->
        [true] ▶ Ukryj prawy panel (F11)
       *[false] ◀ Ukryj panel rozszerzeń (F11)
    }
}

collapse-all = ⏏ Zwiń wszystko
about = ℹ O programie eDirStat

# Status Indicators
scanning-disk = Skanowanie dysku...
scan-complete = Skanowanie ukończone
path-label = Ścieżka: { $path }
worker-threads = ⚡ { $count } Wątki robocze
worker-threads-hover = Liczba równoległych rdzeni procesora (work-stealing) przypisanych do przeszukiwania katalogów.

# Stats Panel (Bottom)
directories-count = 📁 Katalogi: { $count }
files-count = 📄 Pliki: { $count }
total-size = 💾 Całkowity rozmiar: { $size }
elapsed-time = ⏱ Czas: { $time }
scan-speed = ⚡ Prędkość: { $speed }/s

# Selection Info
selection-path = Wybór: { $path }
selection-items = Wybór: { $count ->
    [one] 1 element
    [few] { $count } elementy
   *[other] { $count } elementów
}

# Plot Types
plot-size-distribution = 📊 Rozkład rozmiarów plików
plot-age-size = 🌌 Wiek plików vs. Rozmiar plików
plot-dir-composition = 🍰 Skład katalogów
plot-extension-boxplot = 📦 Rozmiary plików według rozszerzeń
plot-temporal-timeline = ⏱ Powiązane linie czasu
plot-deduplicator-waste = 👥 Przestrzeń marnowana przez duplikaty według rozszerzeń

# --- Deduplicator Strings ---
dedup-desc = Wyszukuj i bezpiecznie usuwaj identyczne pliki (bajt po bajcie) przy użyciu bezpiecznych kryptograficznie skrótów BLAKE3.
dedup-how-it-works = ℹ Jak to działa
dedup-min-size = Minimalny rozmiar pliku:
dedup-ignore-system = Ignoruj pliki systemowe
dedup-ignore-hidden = Ignoruj pliki ukryte
dedup-start-scan = ⚡ Uruchom skanowanie duplikatów
dedup-scan-first = Najpierw zeskanuj katalog.
dedup-cancelled-msg = Skanowanie zostało anulowane. Uruchom nowe skanowanie, aby znaleźć duplikaty.
dedup-analyzing = Analizowanie plików...
dedup-no-duplicates = Nie znaleziono żadnych duplikatów. Spróbuj zmniejszyć minimalny rozmiar pliku lub zeskanować inny folder.
no-permission = Brak uprawnień
hardlink-badge = Hardlink
dedup-select-items = 🎯 Wybierz elementy...
dedup-select-all-but-oldest = 🎯 Wszystkie oprócz najstarszego
dedup-select-all-but-newest = 🎯 Wszystkie oprócz najnowszego
dedup-select-all-but-shortest = 🎯 Wszystkie oprócz najkrótszej ścieżki
dedup-select-all-but-rootmost = 🎯 Wszystkie oprócz najbliższego głównego katalogu
dedup-select-all-but-longest = 🎯 Wszystkie oprócz najdłuższej ścieżki
dedup-pref-dir-pattern = Preferowany wzorzec katalogu:
dedup-select-all-but-pref = 🎯 Wszystkie oprócz preferowanego katalogu
dedup-clear-selection = ❌ Wyczyść wybór
dedup-link-menu = 🔗 Połącz... ({ $count } plików)
dedup-link-menu-disabled = 🔗 Połącz... (0 plików)
dedup-link-hardlinks = 🔗 Zastąp wybrane twardymi dowiązaniami (hardlinks)
dedup-link-softlinks = 🔗 Zastąp wybrane dowiązaniami symbolicznymi (softlinks)
dedup-remove-menu = 🗑 Usuń... ({ $count } plików, { $size })
dedup-remove-menu-disabled = 🗑 Usuń... (0 plików)
dedup-remove-trash = ♻ Przenieś wybrane do kosza
dedup-remove-delete = 🗑 Usuń wybrane trwale
dedup-warning-title = ⚠ OSTRZEŻENIE O UTRACIE DANYCH
dedup-warning-desc = { $count ->
    [one] Usuwanie wszystkich wersji 1 pliku
   *[other] Usuwanie wszystkich wersji { $count } plików
}
dedup-warning-no-original = Żadna oryginalna kopia nie pozostanie:
dedup-warning-details = Zaznaczono oryginał oraz wszystkie kopie duplikatów dla poniższych plików. Ich usunięcie doprowadzi do trwałej utraty danych:
dedup-cancel-hover = Kliknij, aby anulować skanowanie
dedup-current-label = Bieżący
dedup-phase1-size = Faza 1/7: Grupowanie wszystkich plików według rozmiaru...
dedup-phase1-filter = Faza 1/7: Filtrowanie wykluczeń z kandydatów na duplikaty...
dedup-phase2-prefix = Faza 2/7: Haszowanie początków plików (pierwsze 4KB)...
dedup-phase3-midpoint = Faza 3/7: Haszowanie środków plików...
dedup-phase4-suffix = Faza 4/7: Haszowanie końców plików...
dedup-phase5-multirange = Faza 5/7: Haszowanie wielozakresowe dużych plików...
dedup-phase6-full = Faza 6/7: Pełne haszowanie BLAKE3 pozostałych kandydatów...
dedup-phase7-validation = Faza 7/7: Ostateczna weryfikacja znaczników czasu...
dedup-phase-finished = Ukończono w czasie: { $duration }! Znaleziono { $count } grup duplikatów. Potencjalne odzyskanie miejsca: { $space }
dedup-scan-cancelled-with-error = Skanowanie zostało anulowane: { $error }

# Deduplicator Table Headers
dedup-hdr-checkbox = [     ]
dedup-hdr-filename = Nazwa pliku
dedup-hdr-directory = Katalog nadrzędny
dedup-hdr-size = Rozmiar
dedup-hdr-reclaimable = Do odzyskania
dedup-hdr-created = Utworzony
dedup-hdr-modified = Zmodyfikowany
dedup-copies-selected = ({ $count ->
    [one] Zaznaczono 1 kopię
    [few] Zaznaczono { $count } kopie
   *[other] Zaznaczono { $count } kopii
})

# --- Explorer Details Panel ---
explorer-details-header = ℹ Szczegóły
explorer-deselect-hover = Odznacz elementy
explorer-deselect-single-hover = Odznacz element
explorer-selected-items-count = { $count ->
    [one] Zaznaczono 1 element
    [few] Zaznaczono { $count } elementy
   *[other] Zaznaczono { $count } elementów
}
explorer-total-size = Całkowity rozmiar: { $size }
explorer-files = Pliki: { $count }
explorer-directories = Katalogi: { $count }
explorer-actions-title = Akcje
explorer-actions-operations = Operacje:
explorer-action-refresh-hover = Odśwież wszystkie zaznaczone poddrzewa katalogów
explorer-grid-type = Typ:
explorer-grid-size = Rozmiar:
explorer-grid-bytes = Bajty:
explorer-grid-items = Elementy:
explorer-grid-files = Pliki:
explorer-grid-subdirs = Podkatalogi:
explorer-grid-user = Użytkownik:
explorer-grid-group = Grupa:
explorer-grid-permissions = Uprawnienia:
explorer-grid-path = Pełna ścieżka:

# Explorer Type Names
type-symlink = Dowiązanie symboliczne
type-directory = Katalog
type-file = Plik

# Explorer Actions
explorer-action-copy-path = 📋 Kopiuj ścieżkę
explorer-action-open-manager = 🗁 Otwórz menedżer plików
explorer-action-refresh-subtree = 🔄 Odśwież poddrzewo
explorer-action-move-trash = ♻ Przenieś do kosza
explorer-action-delete-permanently = 🗑 Usuń trwale
explorer-action-refresh-directory = 🔄 Odśwież katalog

# Explorer Empty State
explorer-empty-state = Kliknij 'Skanuj katalog', aby zbadać zużycie dysku.
placeholder-treemap = Zeskanowany system plików zostanie tutaj zwizualizowany w postaci mapy drzewa (treemap).
placeholder-plots = Zeskanowany system plików zostanie tutaj przedstawiony na wykresie.

# --- Extensions Panel ---
extensions-header = 📂 Rozszerzenia
extensions-empty = Nie zebrano jeszcze statystyk.
extensions-hover-files = Pliki: { $count }

# --- Operations (Context Actions) ---
op-up-one-level = Przejdź poziom wyżej
op-refresh-entire-scan = Odśwież całe skanowanie
op-refresh-directory = Odśwież katalog
op-open-file-manager = Otwórz w menedżerze plików
op-open-terminal = Otwórz terminal tutaj
op-copy-path = Kopiuj ścieżkę
op-copy-name = Kopiuj nazwę
op-move-trash = Przenieś do kosza
op-permanently-delete = Usuń trwale

# Toast Notifications
toast-already-root = Jesteś już na najwyższym poziomie
toast-navigated-up = Przejście o poziom wyżej powiodło się
toast-refreshing-scan = Odświeżanie całego skanowania...
toast-refreshing-dir = Odświeżanie zaznaczonych katalogów...
toast-opened-manager = Otwarto w menedżerze plików: { $path }
toast-failed-open-manager = Nie udało się otworzyć menedżera plików: { $error }
toast-opened-terminal = Otwarto terminal w: { $path }
toast-failed-open-terminal = Nie udało się otworzyć terminala: { $error }
toast-copied-paths = { $count ->
    [one] Skopiowano 1 ścieżkę do schowka
    [few] Skopiowano { $count } ścieżki do schowka
   *[other] Skopiowano { $count } ścieżek do schowka
}
toast-copied-names = { $count ->
    [one] Skopiowano 1 nazwę do schowka
    [few] Skopiowano { $count } nazwy do schowka
   *[other] Skopiowano { $count } nazw do schowka
}

# --- Modals ---
modal-remember-confirmation = Zapamiętaj potwierdzenie dla wszystkich przyszłych plików i katalogów
modal-process-multiple = Zamierzasz przetworzyć { $count } zduplikowanych plików/elementów:
modal-process-single = Zamierzasz przetworzyć następującą ścieżkę:
# Confirm Deletion/Trash/Link Modals
modal-delete-title = ⚠ OSTRZEŻENIE O TRWAŁYM USUWANIU
modal-delete-header = ⚠ Ostrzeżenie o trwałym usuwaniu!
modal-delete-info = Całkowity rozmiar: { $size }
modal-delete-warning = Jest to usuwanie rekurencyjne. Wszystkie pliki, foldery i podkatalogi pod wybranymi ścieżkami zostaną trwale usunięte i nie będzie można ich odzyskać (z pominięciem kosza).
modal-delete-checkbox = Rozumiem, że pliki zostaną trwale usunięte i nie będzie można ich odzyskać.
modal-delete-confirm = 🗑 Tak, usuń trwale

modal-trash-title = ♻ PRZENIEŚ DO KOSZA
modal-trash-header = ♻ Przenieś do kosza
modal-trash-info = Całkowity rozmiar: { $size }
modal-trash-warning = Spowoduje to przeniesienie wybranych ścieżek oraz ich zawartości do systemowego kosza, skąd mogą być później przywrócone lub trwale usunięte.
modal-trash-checkbox = Potwierdzam chęć przeniesienia tego elementu do kosza.
modal-trash-confirm = ♻ Tak, przenieś do kosza

modal-delete-duplicates-title = ⚠ OSTRZEŻENIE O TRWAŁYM USUWANIU DUPLIKATÓW
modal-delete-duplicates-header = ⚠ Ostrzeżenie o trwałym usuwaniu duplikatów!
modal-delete-duplicates-info = Całkowita przestrzeń do odzyskania: { $size }
modal-delete-duplicates-warning = Wszystkie wybrane pliki zostaną trwale usunięte i nie będzie można ich odzyskać (z pominięciem kosza).
modal-delete-duplicates-checkbox = Rozumiem, że pliki zostaną trwale usunięte i nie będzie można ich odzyskać.
modal-delete-duplicates-confirm = 🗑 Tak, usuń wybrane trwale

modal-trash-duplicates-title = ♻ PRZENIEŚ DUPLIKATY DO KOSZA
modal-trash-duplicates-header = ♻ Przenieś duplikaty do kosza
modal-trash-duplicates-info = Całkowita przestrzeń do odzyskania: { $size }
modal-trash-duplicates-warning = Wszystkie wybrane pliki zostaną przeniesione do systemowego kosza.
modal-trash-duplicates-checkbox = Potwierdzam chęć przeniesienia tych plików do kosza.
modal-trash-duplicates-confirm = ♻ Tak, przenieś wybrane do kosza

modal-hardlink-duplicates-title = 🔗 ZASTĄP DUPLIKATY TWARDYMI DOWIĄZANIAMI
modal-hardlink-duplicates-header = 🔗 Zastąp duplikaty twardymi dowiązaniami
modal-hardlink-duplicates-info = Liczba plików do przetworzenia: { $count }. Skumulowany rozmiar wirtualny: { $size }
modal-hardlink-duplicates-warning = Spowoduje to usunięcie zaznaczonych zduplikowanych plików i zastąpienie ich twardymi dowiązaniami na poziomie systemu plików, wskazującymi na pozostały oryginalny plik z każdej grupy. Pozwala to na wizualne zachowanie plików przy jednoczesnym zwolnieniu rzeczywistego fizycznego miejsca na dysku.
modal-hardlink-duplicates-checkbox = Potwierdzam chęć zastąpienia wybranych plików twardymi dowiązaniami.
modal-hardlink-duplicates-confirm = 🔗 Tak, zastąp twardymi dowiązaniami

modal-softlink-duplicates-title = 🔗 ZASTĄP DUPLIKATY DOWIĄZANIAMI SYMBOLICZNYMI
modal-softlink-duplicates-header = 🔗 Zastąp duplikaty dowiązaniami symbolicznymi
modal-softlink-duplicates-info = Liczba plików do przetworzenia: { $count }. Skumulowany rozmiar wirtualny: { $size }
modal-softlink-duplicates-warning = Spowoduje to usunięcie zaznaczonych zduplikowanych plików i zastąpienie ich dowiązaniami symbolicznymi (softlinks) na poziomie systemu plików, wskazującymi na pozostały oryginalny plik z każdej grupy. Pozwala to na wizualne zachowanie plików przy jednoczesnym zwolnieniu rzeczywistego fizycznego miejsca na dysku.
modal-softlink-duplicates-checkbox = Potwierdzam chęć zastąpienia wybranych plików dowiązaniami symbolicznymi.
modal-softlink-duplicates-confirm = 🔗 Tak, zastąp dowiązaniami symbolicznymi

# Path Does Not Exist Modal
modal-path-not-exist-title = ❌ Ścieżka nie istnieje!
modal-path-not-exist-msg = Błąd: Ścieżka, którą próbujesz usunąć, nie istnieje na dysku.
modal-close-btn = Zamknij
modal-details-label = Szczegóły: 
modal-cancel-btn = Anuluj

# Elevation Recommended Modal
modal-elevation-title = ⚠ Zalecane podniesienie uprawnień
modal-elevation-desc = Program eDirStat jest domyślnie uruchamiany ze standardowymi uprawnieniami użytkownika. Jednak system Windows ściśle ogranicza bezpośredni dostęp do fizycznego uchwytu dysku dla kont administratora.
modal-elevation-mft-disabled = Sterownik NTFS MFT systemu Windows jest wyłączony
modal-elevation-mft-desc = Bez uprawnień administratora nie można zainicjować bezpośredniego skanera MFT. Analiza plików skorzysta z alternatywnego, standardowego przeszukiwania katalogów, co obniża wydajność skanowania nawet 20-krotnie.
modal-elevation-relaunch-prompt = Czy chcesz teraz uruchomić aplikację z uprawnieniami administratora?
modal-elevation-continue-std = Kontynuuj jako standardowy użytkownik
modal-elevation-relaunch-btn = 🛡 Uruchom jako administrator

# About Modal
modal-about-title = ℹ O programie eDirStat
modal-about-author = Autor: Cody Wyatt Neiman (xangelix) <neiman@cody.to>
modal-about-desc1 = Wydajne narzędzie do analizy przestrzeni dyskowej i deduplikacji napisane w języku Rust.
modal-about-desc2 = Posiada równoległe przeszukiwanie katalogów metodą work-stealing, kompresowane migawki z deserializacją układu bez konieczności parsowania oraz interaktywne i płynne mapy drzewa.
modal-about-desc3 = Zintegrowany moduł deduplikacji uruchamia wieloetapowy proces kryptograficznego haszowania w celu bezpiecznego wyodrębnienia grup duplikatów, obliczenia przestrzeni do odzyskania oraz uwzględnienia systemowych twardych dowiązań.
modal-about-licenses-btn = Wyświetl licencje Open Source
modal-about-version = v{ $version }

# How Deduplication Works Modal
modal-how-dedup-title = ℹ Jak działa deduplikacja
modal-how-dedup-desc1 = Zamiast bezpośredniego porównywania bajtów każdego pliku (co wymagałoby powolnego, parzystego skanowania O(N²)), system ten wykorzystuje zoptymalizowany, 7-etapowy proces do bezpiecznej i wydajnej identyfikacji identycznej zawartości.
modal-how-dedup-pipeline-title = 7-etapowy proces:
modal-how-dedup-why-title = Dlaczego to wystarcza?
modal-how-dedup-why-desc1 = Ten wieloetapowy filtr gwarantuje, że w całości zostaną odczytane tylko te pliki, które posiadają identyczny rozmiar, początek, środek, koniec i próbki bloków. Porównanie 256-bitowego skrótu kryptograficznego BLAKE3 zapewnia poziom bezpieczeństwa zgodny z branżowymi protokołami bezpiecznego transferu danych, eliminując potrzebę powolnego porównywania bajt po bajcie.

# How Deduplication Works Steps
modal-how-dedup-step1-title = 1. Podział według rozmiaru
modal-how-dedup-step1-desc = Pliki są grupowane według ich dokładnego rozmiaru w bajtach. Każdy plik o unikalnym rozmiarze jest natychmiast odrzucany, co całkowicie omija operacje wejścia/wyjścia na dysku.
modal-how-dedup-step2-title = 2. Haszowanie początku (prefix)
modal-how-dedup-step2-desc = Pierwsze 4KB pozostałych kandydatów jest haszowane. Pozwala to na szybkie odfiltrowanie plików o różnych nagłówkach lub formatach metadanych.
modal-how-dedup-step3-title = 3. Haszowanie środka (midpoint)
modal-how-dedup-step3-desc = Środkowy blok 4KB pozostałych plików jest haszowany, co ujawnia wewnętrzne różnice strukturalne.
modal-how-dedup-step4-title = 4. Haszowanie końca (suffix)
modal-how-dedup-step4-desc = Ostatnie 4KB danych jest haszowane. Jest to bardzo skuteczne przy identyfikowaniu różnic w końcowej zawartości pliku lub metadanych.
modal-how-dedup-step5-title = 5. Haszowanie wielozakresowe
modal-how-dedup-step5-desc = Duże pliki (powyżej 100MB) podlegają okresowemu próbkowaniu bloków na całej ich długości w celu sprawdzenia spójności zawartości bez odczytywania całego pliku.
modal-how-dedup-step6-title = 6. Pełny hasz BLAKE3
modal-how-dedup-step6-desc = Dla pozostałych kandydatów obliczany jest pełny hasz kryptograficzny BLAKE3. Ze względu na wysoką odporność na kolizje przestrzeni 256-bitowej, pasujące skróty wskazują na astronomiczne prawdopodobieństwo, że pliki są identyczne, stanowiąc wysoce wiarygodny dowód tożsamości bez konieczności porównań parzystych.
modal-how-dedup-step7-title = 7. Weryfikacja znaczników czasu
modal-how-dedup-step7-desc = Tuż przed wyświetleniem lub wykonaniem jakiejkolwiek akcji deduplikacji program weryfikuje znaczniki czasu plików na dysku, aby zabezpieczyć się przed zmianami, które zaszły od momentu wygenerowania migawki.

# Open Source Licenses Modal
modal-licenses-title = 📜 Licencje Open Source
modal-licenses-desc = W aplikacji używane są następujące biblioteki i pakiety (crates) stron trzecich:


# Explorer Column Headers
explorer-hdr-name = Nazwa
explorer-hdr-percentage = Procent
explorer-hdr-size = Rozmiar
explorer-hdr-items = Elementy
explorer-hdr-files = Pliki
explorer-hdr-subdirs = Podkatal.
explorer-hdr-created = Utworzony
explorer-hdr-modified = Zmodyfikowany

# Update Checker
update-checking = Sprawdzanie aktualizacji...
update-available = Nowa wersja { $version } jest dostępna!
update-up-to-date = Masz najnowszą wersję
update-failed = Błąd sprawdzania aktualizacji: { $error }
