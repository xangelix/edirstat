# Menu Bar Dropdowns
file = Файл
view = Вигляд
help = Довідка

# Menu Bar Actions
new-scan = 📁 Нове сканування
save-snapshot = 💾 Зберегти знімок
load-snapshot = 📖 Завантажити знімок

# Menu Bar Status
idle = Очікування

# View Menu Options
monospace-paths = 🅰 Моноширинні шляхи
highlight-duplicates = ✨ Підсвічувати дублікати
treemap-borders = 🔳 Межі деревоподібної карти
treemap-style =  Стиль деревоподібної карти
treemap-style-vertical = Вертикальний градієнт
treemap-style-offset-vertical = Зміщений вертикальний градієнт
treemap-style-diagonal = Діагональний градієнт
treemap-style-cushion = Подушкове затінення
deletion-confirmation = 🗑 Підтвердження видалення
trash-confirmation = ♻ Підтвердження переміщення в кошик
time-format = 🕒 Формат часу
language = 💬 Мова
layout-mode = Режим компонування:
classic-layout = Класичне компонування
windirstat-layout = Компонування WinDirStat
vis-mode-treemap = 📊 Деревоподібна карта
vis-mode-plots = 📈 Графіки
select-plot-label = Оберіть графік:
vis-mode-deduplicator = 👥 Пошук дублікатів
search-filter-label = 🔍 Фільтр:

# Panel Toggles
toggle-left-panel = { $collapsed ->
    [true] ▶ Показати ліву панель (F9)
   *[false] ◀ Сховати ліву панель (F9)
}

toggle-right-panel = { $collapsed ->
    [true] { $is_classic ->
        [true] ◀ Показати праву панель (F11)
       *[false] ▶ Показати панель розширень (F11)
    }
   *[false] { $is_classic ->
        [true] ▶ Сховати праву панель (F11)
       *[false] ◀ Сховати панель розширень (F11)
    }
}

collapse-all = ⏏ Згорнути все
about = ℹ Про програму
web-not-available = Функція недоступна у вебверсії

# Status Indicators
scanning-disk = Сканування диска...
scan-complete = Сканування завершено
scan-cancelled = Сканування скасовано
path-label = Шлях: { $path }
worker-threads = ⚡ Робочих потоків: { $count }
worker-threads-hover = Кількість паралельних ядер ЦП із перехопленням роботи (work-stealing), виділених для обходу каталогів.

# Stats Panel (Bottom)
directories-count = 📁 Каталогів: { $count }
files-count = 📄 Файлів: { $count }
total-size = 💾 Загальний розмір: { $size }
elapsed-time = ⏱ Час: { $time }
scan-speed = ⚡ Швидкість: { $speed }/с

# Selection Info
selection-path = Виділено: { $path }
selection-items = Виділено: { $count ->
    [one] { $count } елемент
    [few] { $count } елементи
    [many] { $count } елементів
   *[other] { $count } елемента
}

# Plot Types
plot-size-distribution = 📊 Розподіл розмірів файлів
plot-age-size = 🌌 Вік файлу та розмір файлу
plot-dir-composition = 🍰 Склад каталогу
plot-extension-boxplot = 📦 Розміри файлів за розширенням
plot-temporal-timeline = ⏱ Пов'язані часові шкали
plot-deduplicator-waste = 👥 Втрати на дублікати за розширенням

# --- Deduplicator Strings ---
dedup-desc = Знаходьте та безпечно видаляйте побайтово ідентичні файли за допомогою криптографічно стійких хешів BLAKE3.
dedup-how-it-works = ℹ Як це працює
dedup-min-size = Мін. розмір файлу:
dedup-ignore-system = Ігнорувати системні файли
dedup-ignore-hidden = Ігнорувати приховані файли
dedup-start-scan = ⚡ Почати пошук дублікатів
dedup-scan-first = Спершу проскануйте каталог.
dedup-cancelled-msg = Сканування скасовано. Почніть нове сканування, щоб знайти дублікати.
dedup-analyzing = Аналіз файлів...
dedup-no-duplicates = Груп дублікатів не знайдено. Спробуйте зменшити мінімальний розмір файлу або проскануйте іншу теку.
no-permission = Немає дозволу
hardlink-badge = Жорстке посилання
dedup-select-items = 🎯 Виділити елементи...
dedup-select-all-but-oldest = 🎯 Усі, крім найстарішого
dedup-select-all-but-newest = 🎯 Усі, крім найновішого
dedup-select-all-but-shortest = 🎯 Усі, крім найкоротшого шляху
dedup-select-all-but-rootmost = 🎯 Усі, крім найближчого до кореня
dedup-select-all-but-longest = 🎯 Усі, крім найдовшого шляху
dedup-pref-dir-pattern = Шаблон бажаного каталогу:
dedup-select-all-but-pref = 🎯 Усі, крім бажаного каталогу
dedup-clear-selection = ❌ Зняти виділення
dedup-link-menu = 🔗 Пов'язати... (файлів: { $count })
dedup-link-menu-disabled = 🔗 Пов'язати... (файлів: 0)
dedup-link-hardlinks = 🔗 Замінити виділені жорсткими посиланнями
dedup-link-softlinks = 🔗 Замінити виділені символьними посиланнями
dedup-remove-menu = 🗑 Видалити... (файлів: { $count }, { $size })
dedup-remove-menu-disabled = 🗑 Видалити... (файлів: 0)
dedup-remove-trash = ♻ Перемістити виділені в кошик
dedup-remove-delete = 🗑 Остаточно видалити виділені
dedup-warning-title = ⚠ ПОПЕРЕДЖЕННЯ ПРО ВТРАТУ ДАНИХ
dedup-warning-desc = { $count ->
    [one] Видалення всіх версій { $count } файлу
    [few] Видалення всіх версій { $count } файлів
    [many] Видалення всіх версій { $count } файлів
   *[other] Видалення всіх версій { $count } файлу
}
dedup-warning-no-original = Жодної оригінальної копії не залишиться:
dedup-warning-details = Ви позначили і оригінал, і всі копії-дублікати файлів, наведених нижче. Їх видалення, найімовірніше, призведе до остаточної втрати даних:
dedup-cancel-hover = Натисніть, щоб скасувати сканування
scan-cancel-hover = Натисніть, щоб скасувати сканування
dedup-current-label = Поточний
dedup-phase1-size = Етап 1/7: Групування всіх просканованих файлів за розміром...
dedup-phase1-filter = Етап 1/7: Застосування винятків до кандидатів у дублікати...
dedup-phase2-prefix = Етап 2/7: Хешування початків файлів (перші 4 КБ)...
dedup-phase3-midpoint = Етап 3/7: Хешування середин файлів...
dedup-phase4-suffix = Етап 4/7: Хешування кінців файлів...
dedup-phase5-multirange = Етап 5/7: Багатодіапазонне хешування великих файлів...
dedup-phase6-full = Етап 6/7: Повне хешування BLAKE3 решти кандидатів...
dedup-phase7-validation = Етап 7/7: Остаточна перевірка часових позначок...
dedup-phase-finished = Завершено за { $duration }! Знайдено груп дублікатів: { $count }. Можна звільнити: { $space }
dedup-scan-cancelled-with-error = Сканування скасовано: { $error }

# Deduplicator Table Headers
dedup-hdr-checkbox = [     ]
dedup-hdr-filename = Ім'я файлу
dedup-hdr-directory = Батьківський каталог
dedup-hdr-size = Розмір
dedup-hdr-reclaimable = Можна звільнити
dedup-hdr-created = Створено
dedup-hdr-modified = Змінено
dedup-copies-selected = ({ $count ->
    [one] виділено { $count } копію
    [few] виділено { $count } копії
    [many] виділено { $count } копій
   *[other] виділено { $count } копії
})

# --- Explorer Details Panel ---
explorer-details-header = ℹ Подробиці
explorer-deselect-hover = Зняти виділення з елементів
explorer-deselect-single-hover = Зняти виділення з елемента
explorer-selected-items-count = { $count ->
    [one] Виділено { $count } елемент
    [few] Виділено { $count } елементи
    [many] Виділено { $count } елементів
   *[other] Виділено { $count } елемента
}
explorer-total-size = Загальний розмір: { $size }
explorer-files = Файлів: { $count }
explorer-directories = Каталогів: { $count }
explorer-actions-title = Дії
explorer-actions-operations = Операції:
explorer-action-refresh-hover = Оновити всі виділені піддерева каталогів
explorer-grid-type = Тип:
explorer-grid-size = Розмір:
explorer-grid-bytes = Байтів:
explorer-grid-items = Елементів:
explorer-grid-files = Файлів:
explorer-grid-subdirs = Підкаталогів:
explorer-grid-user = Користувач:
explorer-grid-group = Група:
explorer-grid-permissions = Дозволи:
explorer-grid-path = Повний шлях:

# Explorer Type Names
type-symlink = Символьне посилання
type-directory = Каталог
type-file = Файл

# Explorer Actions
explorer-action-copy-path = 📋 Копіювати шлях
explorer-action-open-file = 📄 Відкрити файл
explorer-action-open-manager = 🗁 Відкрити в менеджері
explorer-action-refresh-subtree = 🔄 Оновити піддерево
explorer-action-move-trash = ♻ Перемістити в кошик
explorer-action-delete-permanently = 🗑 Видалити остаточно
explorer-action-refresh-directory = 🔄 Оновити каталог

# Explorer Empty State
explorer-empty-state = Натисніть «Нове сканування», щоб дослідити використання диска.
choose-an-option = Оберіть варіант
web-viewer = Вебпереглядач
load-demo = 👁 Завантажити демонстраційний знімок
placeholder-treemap = Тут проскановану файлову систему буде показано у вигляді деревоподібної карти.
placeholder-plots = Тут буде побудовано графіки просканованої файлової системи.

# Treemap Zoom & Navigation
zoom-up = ⏶ Вгору
zoom-reset = ❌ Скинути
zoom-to-dir = 🔍 Показати на карті
zoom-up-level = ⏶ На рівень вище
zoom-empty-dir = Каталог порожній

# --- Extensions Panel ---
extensions-header = 📂 Розширення
extensions-empty = Статистику ще не зібрано.
extensions-hover-files = Файлів: { $count }

# --- Operations (Context Actions) ---
op-up-one-level = На рівень вище
op-zoom-treemap = Показати на карті
op-refresh-entire-scan = Оновити все сканування
op-refresh-directory = Оновити каталог
op-open-file = Відкрити файл
op-open-file-manager = Відкрити у файловому менеджері
op-open-terminal = Відкрити термінал тут
op-copy-path = Копіювати шлях
op-copy-name = Копіювати ім'я
op-move-trash = Перемістити в кошик
op-permanently-delete = Видалити остаточно

# Toast Notifications
toast-already-root = Ви вже на кореневому рівні
toast-navigated-up = Перехід на рівень вище
toast-zoomed-treemap = Карту сфокусовано на каталозі
toast-refreshing-scan = Оновлення всього сканування...
toast-refreshing-dir = Оновлення виділених каталогів...
toast-opened-file = Відкрито: { $path }
toast-failed-open-file = Не вдалося відкрити файл: { $error }
toast-opened-manager = Відкрито у файловому менеджері: { $path }
toast-failed-open-manager = Не вдалося відкрити у файловому менеджері: { $error }
toast-opened-terminal = Термінал відкрито в: { $path }
toast-failed-open-terminal = Не вдалося відкрити термінал: { $error }
toast-copied-paths = { $count ->
    [one] Скопійовано { $count } шлях до буфера обміну
    [few] Скопійовано { $count } шляхи до буфера обміну
    [many] Скопійовано { $count } шляхів до буфера обміну
   *[other] Скопійовано { $count } шляху до буфера обміну
}
toast-copied-names = { $count ->
    [one] Скопійовано { $count } ім'я до буфера обміну
    [few] Скопійовано { $count } імені до буфера обміну
    [many] Скопійовано { $count } імен до буфера обміну
   *[other] Скопійовано { $count } імені до буфера обміну
}

# --- Modals ---
modal-remember-confirmation = Запам'ятати підтвердження для всіх майбутніх файлів і каталогів
modal-process-multiple = Буде оброблено файлів/елементів-дублікатів: { $count }
modal-process-single = Буде оброблено такий шлях:
# Confirm Deletion/Trash/Link Modals
modal-delete-title = ⚠ ПОПЕРЕДЖЕННЯ ПРО ОСТАТОЧНЕ ВИДАЛЕННЯ
modal-delete-header = ⚠ Попередження про остаточне видалення!
modal-delete-info = Загальний розмір: { $size }
modal-delete-warning = Це рекурсивне видалення. Усі файли, теки та підкаталоги за виділеними шляхами буде видалено остаточно без можливості відновлення (в обхід кошика).
modal-delete-checkbox = Я розумію, що файли буде видалено остаточно без можливості відновлення.
modal-delete-confirm = 🗑 Так, видалити остаточно

modal-trash-title = ♻ ПЕРЕМІСТИТИ В КОШИК
modal-trash-header = ♻ Перемістити в кошик
modal-trash-info = Загальний розмір: { $size }
modal-trash-warning = Виділені шляхи та весь їхній вміст буде переміщено до системного кошика, звідки їх можна буде відновити або пізніше видалити остаточно.
modal-trash-checkbox = Я підтверджую, що хочу перемістити це в кошик.
modal-trash-confirm = ♻ Так, перемістити в кошик

modal-delete-duplicates-title = ⚠ ПОПЕРЕДЖЕННЯ ПРО ОСТАТОЧНЕ ВИДАЛЕННЯ ДУБЛІКАТІВ
modal-delete-duplicates-header = ⚠ Попередження про остаточне видалення дублікатів!
modal-delete-duplicates-info = Буде звільнено: { $size }
modal-delete-duplicates-warning = Усі виділені файли буде видалено остаточно без можливості відновлення (в обхід кошика).
modal-delete-duplicates-checkbox = Я розумію, що файли буде видалено остаточно без можливості відновлення.
modal-delete-duplicates-confirm = 🗑 Так, остаточно видалити виділені

modal-trash-duplicates-title = ♻ ПЕРЕМІСТИТИ ДУБЛІКАТИ В КОШИК
modal-trash-duplicates-header = ♻ Перемістити дублікати в кошик
modal-trash-duplicates-info = Буде звільнено: { $size }
modal-trash-duplicates-warning = Усі виділені файли буде переміщено в кошик.
modal-trash-duplicates-checkbox = Я підтверджую, що хочу перемістити ці файли в кошик.
modal-trash-duplicates-confirm = ♻ Так, перемістити виділені в кошик

modal-hardlink-duplicates-title = 🔗 ЗАМІНИТИ ДУБЛІКАТИ ЖОРСТКИМИ ПОСИЛАННЯМИ
modal-hardlink-duplicates-header = 🔗 Замінити дублікати жорсткими посиланнями
modal-hardlink-duplicates-info = Файлів до обробки: { $count }. Сукупний віртуальний розмір: { $size }
modal-hardlink-duplicates-warning = Виділені файли-дублікати буде видалено та замінено жорсткими посиланнями файлової системи на оригінальний файл, що залишається в кожній групі. Файли залишаються на своїх місцях, а фізичне місце на диску звільняється.
modal-hardlink-duplicates-checkbox = Я підтверджую, що хочу замінити виділені файли жорсткими посиланнями.
modal-hardlink-duplicates-confirm = 🔗 Так, замінити жорсткими посиланнями

modal-softlink-duplicates-title = 🔗 ЗАМІНИТИ ДУБЛІКАТИ СИМВОЛЬНИМИ ПОСИЛАННЯМИ
modal-softlink-duplicates-header = 🔗 Замінити дублікати символьними посиланнями
modal-softlink-duplicates-info = Файлів до обробки: { $count }. Сукупний віртуальний розмір: { $size }
modal-softlink-duplicates-warning = Виділені файли-дублікати буде видалено та замінено символьними посиланнями (softlinks) файлової системи на оригінальний файл, що залишається в кожній групі. Файли залишаються на своїх місцях, а фізичне місце на диску звільняється.
modal-softlink-duplicates-checkbox = Я підтверджую, що хочу замінити виділені файли символьними посиланнями.
modal-softlink-duplicates-confirm = 🔗 Так, замінити символьними посиланнями

# Path Does Not Exist Modal
modal-path-not-exist-title = ❌ Шлях не існує!
modal-path-not-exist-msg = Помилка: шлях, який ви намагаєтеся видалити, не існує на диску.
modal-close-btn = Закрити
modal-details-label = Подробиці:
modal-cancel-btn = Скасувати

# Elevation Recommended Modal
modal-elevation-title = ⚠ Рекомендовано підвищені права
modal-elevation-desc = За замовчуванням eDirStat працює з правами звичайного користувача. Однак Windows суворо обмежує прямий доступ до фізичного диска обліковими записами адміністраторів.
modal-elevation-mft-disabled = Драйвер NTFS MFT для Windows вимкнено
modal-elevation-mft-desc = Без прав адміністратора сканер MFT із прямим доступом до диска не може запуститися. Для аналізу файлів буде використано резервний стандартний драйвер обходу, що може сповільнити сканування до 20 разів.
modal-elevation-relaunch-prompt = Перезапустити програму з правами адміністратора зараз?
modal-elevation-continue-std = Продовжити як звичайний користувач
modal-elevation-relaunch-btn = 🛡 Перезапустити як адміністратор

# About Modal
modal-about-title = ℹ Про eDirStat
modal-about-author = Автор: Cody Wyatt Neiman (xangelix) <neiman@cody.to>
modal-about-license-btn = 📜 Ліцензія (MIT)
modal-about-desc1 = Високопродуктивний інструмент для аналізу дискового простору та пошуку дублікатів, написаний на Rust.
modal-about-desc2 = Паралельний обхід каталогів із перехопленням роботи, стиснені знімки з десеріалізацією без розбору (zero-parsing) та чутливі інтерактивні деревоподібні карти.
modal-about-desc3 = Вбудований пошук дублікатів використовує багатоетапний конвеєр криптографічного хешування, щоб безпечно виокремлювати групи дублікатів, обчислювати місце, яке можна звільнити, і враховувати жорсткі посилання системи.
modal-about-licenses-btn = Переглянути ліцензії відкритого коду
modal-about-version = v{ $version }

# How Deduplication Works Modal
modal-how-dedup-title = ℹ Як працює пошук дублікатів
modal-how-dedup-desc1 = Замість того щоб напряму порівнювати байти кожного файлу (що вимагає повільних попарних перевірок O(N²)), система використовує оптимізований 7-етапний конвеєр для безпечного й ефективного виявлення ідентичного вмісту.
modal-how-dedup-pipeline-title = 7-етапний конвеєр:
modal-how-dedup-why-title = Чому цього достатньо?
modal-how-dedup-why-desc1 = Цей багатоетапний фільтр гарантує, що повністю читаються лише файли з однаковим розміром, початком, серединою, кінцем і вибірками розподілених блоків. Насамкінець порівняння 256-бітного криптографічного хешу BLAKE3 забезпечує рівень надійності, порівнянний із промисловими протоколами безпечної передачі даних, без потреби в повільних попарних побайтових порівняннях.

# How Deduplication Works Steps
modal-how-dedup-step1-title = 1. Розбиття за розміром
modal-how-dedup-step1-desc = Файли групуються за точним розміром у байтах. Будь-який файл з унікальним розміром одразу відкидається, без жодного звернення до диска.
modal-how-dedup-step2-title = 2. Хешування початку
modal-how-dedup-step2-desc = Хешуються перші 4 КБ решти кандидатів. Це швидко відсіює файли з різними заголовками чи форматами метаданих.
modal-how-dedup-step3-title = 3. Хешування середини
modal-how-dedup-step3-desc = Хешується блок 4 КБ із середини решти файлів, що виявляє внутрішні структурні відмінності.
modal-how-dedup-step4-title = 4. Хешування кінця
modal-how-dedup-step4-desc = Хешуються останні 4 КБ даних. Це дуже ефективно виявляє відмінності в кінцевому вмісті чи метаданих.
modal-how-dedup-step5-title = 5. Багатодіапазонне хешування
modal-how-dedup-step5-desc = Для великих файлів (понад 100 МБ) по всій довжині періодично вибираються блоки, щоб перевірити узгодженість вмісту без читання всього файлу.
modal-how-dedup-step6-title = 6. Повне хешування BLAKE3
modal-how-dedup-step6-desc = Для решти кандидатів обчислюється повний криптографічний хеш BLAKE3. Завдяки високій стійкості до колізій 256-бітного простору збіг хешів означає астрономічно малу ймовірність того, що файли відрізняються, — це дуже надійний доказ ідентичності без попарних порівнянь.
modal-how-dedup-step7-title = 7. Перевірка часових позначок
modal-how-dedup-step7-desc = Безпосередньо перед показом або виконанням будь-якої дії з дублікатами програма перевіряє часові позначки файлів на диску, щоб захиститися від змін, що відбулися після створення знімка.

# Open Source Licenses Modal
modal-licenses-title = 📜 Ліцензії відкритого коду
modal-licenses-tab-app = eDirStat (MIT)
modal-licenses-tab-deps = Сторонні бібліотеки
modal-licenses-app-desc = eDirStat — програмне забезпечення з відкритим кодом, що розповсюджується за ліцензією MIT:
modal-licenses-desc = У цій програмі використовуються такі сторонні бібліотеки та crates:
modal-licenses-copy-btn = 📋 Копіювати ліцензію
modal-licenses-copy-all-btn = 📋 Копіювати ліцензії

# Processing Modal
modal-processing-title = ⏳ Обробка...
modal-processing-deletion = Видалення файлів і каталогів...
modal-processing-trash = Переміщення файлів і каталогів у кошик...
modal-processing-hardlink = Заміна дублікатів жорсткими посиланнями...
modal-processing-softlink = Заміна дублікатів символьними посиланнями...

# Explorer Column Headers
explorer-hdr-name = Ім'я
explorer-hdr-percentage = Відсоток
explorer-hdr-size = Розмір
explorer-hdr-items = Елементи
explorer-hdr-files = Файли
explorer-hdr-subdirs = Підкат.
explorer-hdr-created = Створено
explorer-hdr-modified = Змінено

# Update Checker
update-checking = Перевірка оновлень...
update-available = Доступна нова версія { $version }!
update-up-to-date = У вас найновіша версія
update-failed = Не вдалося перевірити оновлення: { $error }

# Themes
theme = 🎨 Тема
theme-dark = Темна
theme-high-contrast = Висока контрастність
theme-light = Світла
theme-system = Системна

# New Scan Options Modal
modal-scan-options-title = Параметри нового сканування
modal-scan-options-header = Почати нове сканування
modal-scan-options-path-label = Шлях до каталогу для сканування:
modal-scan-options-paste-tooltip = Вставити з буфера обміну
modal-scan-options-browse-tooltip = Огляд теки...
modal-scan-options-scan-btn = Сканувати
modal-scan-options-cancel-btn = Скасувати
modal-scan-options-same-filesystem = Обмежити сканування однією файловою системою/томом
modal-scan-options-drives-header = 💽 Накопичувачі та томи
modal-scan-options-refresh-tooltip = Оновити список накопичувачів
modal-scan-options-root-system = Коренева система
modal-scan-options-selected-badge = ✅ Вибрано
modal-scan-options-free-of = Вільно { $free } з { $total }
modal-scan-options-subtitle = Оберіть том, швидке розташування або власний каталог для аналізу.
modal-scan-options-quick-access = 📍 Швидкий доступ
modal-scan-options-path-hint = /шлях/для/сканування
modal-scan-options-hint = ℹ Оберіть накопичувач вище або введіть шлях до каталогу.
modal-scan-options-sandbox-auth = 🔒 Потрібен доступ до пісочниці — натисніть «Сканувати», щоб надати доступ
modal-scan-options-valid-dir = ✅ Коректний каталог — готово до сканування
modal-scan-options-points-to-file = ⚠ Шлях вказує на файл — оберіть теку.
modal-scan-options-dir-not-exist = ⚠ Каталог не існує у файловій системі.
quick-loc-home = 🏠 Домашня тека
quick-loc-documents = 📄 Документи
quick-loc-downloads = 📥 Завантаження
quick-loc-desktop = 🖥 Робочий стіл
quick-loc-pictures = 🖼 Зображення
search-use-regex = Використовувати регулярний вираз (Regex)
search-match-case = Враховувати регістр
dedup-pref-dir-hint = напр. /home/user/Archive

file-menu-close = Закрити сканування
file-menu-quit = Вийти

badge-dataless-cloud = Хмарний файл / без локальних даних
badge-symlink = Символьне посилання
badge-special-file = Спеціальний файл (канал / сокет / пристрій)
badge-permission-denied = Доступ заборонено

# Docker Disk Usage
vis-mode-docker = 📦 Docker
docker-desc = Використання диска локальною інсталяцією Docker, зчитане безпосередньо з кореневого каталогу даних на диску — без підключення до демона.
docker-no-install = На цьому комп'ютері не виявлено інсталяції Docker.
docker-env-roots-title = Кореневі каталоги даних
docker-env-vmdisks-title = Диски віртуальних машин
docker-env-socket = Сокет демона:
docker-env-roots-header = Кореневі каталоги даних ({ $count })
docker-env-vmdisks-header = Диски віртуальних машин ({ $count })
docker-kind-overlay2 = overlay2
docker-kind-containerd = containerd (сховище образів)
docker-kind-unknown = невідома структура
docker-kind-overlay = overlay
docker-kind-aufs = aufs
docker-kind-btrfs = btrfs
docker-kind-zfs = zfs
docker-kind-devicemapper = devicemapper
docker-kind-vfs = vfs
docker-kind-windowsfilter = windowsfilter
docker-scope-system = системний
docker-scope-rootless = rootless
docker-readable = ✅ доступний для читання
docker-not-readable = ❌ недоступний для читання
docker-vm-size = { $apparent } видимий розмір, { $allocated } виділено на диску
docker-vm-note = 💽 Docker Desktop зберігає всі свої дані всередині диска ВМ, наведеного вище; з хоста неможливо визначити, що саме займає місце.
docker-needs-elevated = ⚠ Знайдено кореневий каталог даних Docker, але він недоступний для читання. Запустіть eDirStat із підвищеними правами (sudo або адміністратор), щоб проаналізувати його.
docker-unsupported-layout = Цей кореневий каталог даних Docker використовує структуру сховища, яку eDirStat поки що не вміє аналізувати (наразі підтримується лише overlay2).
docker-analyze = ⚡ Аналізувати
docker-analyzing = Аналіз кореневого каталогу даних Docker...
docker-error = Не вдалося провести інвентаризацію Docker: { $error }
docker-lbl-images = Дані образів
docker-lbl-shared = Спільні шари
docker-lbl-unique = Унікальні дані образів
docker-lbl-containers = RW контейнерів
docker-lbl-volumes = Томи
docker-lbl-build-cache = Кеш збірки
docker-lbl-logs = Журнали контейнерів
docker-lbl-reclaimable = Можна звільнити
docker-section-images = Образи
docker-section-containers = Контейнери
docker-section-volumes = Томи
docker-hdr-id = ID
docker-hdr-shared = Спільний
docker-hdr-exclusive = Ексклюзивний
docker-hdr-layers = Шари
docker-hdr-rw = Розмір RW
docker-hdr-log = Розмір журналу
docker-no-entries = Немає записів.
docker-warnings-title = ⚠ Попередження
docker-warn-large-log = Контейнер '{ $container }' має великий файл журналу ({ $size }).
docker-warn-unref-layer = На шар { $cache_id } не посилається жоден образ (можна звільнити { $size }).
docker-warn-root-unreadable = Кореневий каталог даних { $path } недоступний для читання (потрібні підвищені права).
docker-warn-metadata = Проблема з метаданими: { $detail }
docker-warn-containerd-partial = Використовується сховище образів containerd: образи та шари неможливо визначити з диска (підтримку Docker API заплановано), тому ця інвентаризація охоплює лише контейнери, журнали, томи та кеш збірки.
docker-source-disk = сканування диска
docker-source-partial = сканування диска (часткове)
docker-source-api = Docker API
docker-collected-at = Зібрано { $time }
docker-refresh = 🔄 Оновити
docker-no-snapshot-data = Цей знімок не містить даних Docker.
docker-sandbox-unavailable = Жива інтеграція з Docker недоступна у версії eDirStat з App Store, але знімки, зібрані на інших комп'ютерах, усе одно можна переглядати тут.
docker-delete = Видалити
docker-delete-title = Видалити образ Docker
docker-delete-confirm = Видалити образ Docker { $name } ({ $size }) через демон Docker?
docker-delete-warning = Це знімає теги та видаляє образ через демон Docker. Операцію неможливо скасувати; шари, на які ще посилаються інші образи, буде збережено.
docker-delete-force = Примусове видалення
docker-delete-force-hover = Видалити образ, навіть якщо на нього посилаються контейнери (прапорець force демона).
docker-deleting = Видалення образу...
docker-delete-done = Образ { $name } видалено: знято тегів — { $untagged }, видалено шарів — { $deleted }.
docker-delete-failed = Не вдалося видалити образ: { $error }
docker-delete-container = Видалити контейнер
docker-delete-volume = Видалити том
docker-delete-container-confirm = Видалити контейнер { $name } (записуваний шар { $size }, журнал { $extra }) через демон Docker?
docker-delete-volume-confirm = Видалити том { $name } ({ $size }) через демон Docker?
docker-delete-container-warning = Це остаточно видаляє контейнер через демон Docker. Операцію неможливо скасувати.
docker-delete-volume-warning = Це остаточно видаляє том і його дані через демон Docker. Операцію неможливо скасувати.
docker-delete-container-force-hover = Видалити контейнер, навіть якщо він ще працює (прапорець force демона).
docker-delete-volume-force-hover = Видалити том, навіть якщо його використовують контейнери (прапорець force демона).
docker-deleting-container = Видалення контейнера...
docker-deleting-volume = Видалення тому...
docker-delete-container-done = Контейнер { $name } видалено.
docker-delete-volume-done = Том { $name } видалено.
docker-delete-container-failed = Не вдалося видалити контейнер: { $error }
docker-delete-volume-failed = Не вдалося видалити том: { $error }
docker-hdr-created = Створено
docker-hdr-refs = Посилання
docker-delete-title-multi = Видалити образи Docker
docker-delete-container-multi = Видалити контейнери
docker-delete-volume-multi = Видалити томи
docker-delete-confirm-multi = Видалити образи ({ $count }, загалом { $size }) через демон Docker?
docker-delete-container-confirm-multi = Видалити контейнери ({ $count }; записуваний шар { $size }, журнал { $extra } загалом) через демон Docker?
docker-delete-volume-confirm-multi = Видалити томи ({ $count }, загалом { $size }) через демон Docker?
docker-delete-more = ... і ще { $count }
docker-delete-done-multi = Видалено образів: { $count }. Знято тегів — { $untagged }, видалено шарів — { $deleted }.
docker-delete-container-done-multi = Видалено контейнерів: { $count }.
docker-delete-volume-done-multi = Видалено томів: { $count }.
docker-delete-partial-multi = Видалено { $succeeded } з { $count }, не вдалося — { $failed }. Перша помилка: { $error }
badge-docker = Кореневий каталог даних Docker
badge-docker-vm = Віртуальний диск Docker
badge-docker-area = Сховище Docker: { $area }

# Cleanup
vis-mode-cleanup = 🗑 Очищення
cleanup-desc = Відомі каталоги, які можна очистити, знайдені в цьому скануванні: артефакти збірки та кеші, які можна перемістити в кошик, а також системні кеші, які краще очищати їхніми власними інструментами.
cleanup-summary = Місць: { $count }, можна звільнити { $size }
cleanup-regenerable-total = { $size } можна звільнити напряму
cleanup-tool-total = { $size } краще очистити системними інструментами
cleanup-tier-regenerable = Відновлюване
cleanup-tier-tool = Системний інструмент
cleanup-empty = У цьому скануванні немає відомих цілей для очищення.
cleanup-analyzing = Пошук цілей для очищення в скануванні...
cleanup-run-command = Виконайте в терміналі:
cleanup-copy = Копіювати
cleanup-copied = Команду скопійовано до буфера обміну.
cleanup-age-days = Днів: { $days }
cleanup-age-fresh = Нещодавнє
cleanup-toast = 🗑 Знайдено { $size } місця, яке можна звільнити, — відкрийте вкладку «Очищення», щоб переглянути.
cleanup-hdr-tier = Рівень
cleanup-hdr-title = Що
cleanup-hdr-path = Розташування
cleanup-hdr-age = Вік
cleanup-hdr-action = Дія
