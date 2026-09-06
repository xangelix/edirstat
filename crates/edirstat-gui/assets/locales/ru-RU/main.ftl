# Menu Bar Dropdowns
file = Файл
view = Вид
help = Справка

# Menu Bar Actions
new-scan = 📁 Новое сканирование
save-snapshot = 💾 Сохранить снимок
load-snapshot = 📖 Загрузить снимок

# Menu Bar Status
idle = Ожидание

# View Menu Options
monospace-paths = 🅰 Моноширинные пути
highlight-duplicates = ✨ Подсветка дубликатов
treemap-borders = 🔳 Границы древовидной карты
treemap-style =  Стиль древовидной карты
treemap-style-vertical = Вертикальный градиент
treemap-style-offset-vertical = Смещенный вертикальный градиент
treemap-style-diagonal = Диагональный градиент
treemap-style-cushion = Рельефное затенение
deletion-confirmation = 🗑 Подтверждение удаления
trash-confirmation = ♻ Подтверждение перемещения в корзину
time-format = 🕒 Формат времени
language = 💬 Язык
layout-mode = Режим компоновки:
classic-layout = Классическая компоновка
windirstat-layout = Компоновка WinDirStat
vis-mode-treemap = 📊 Древовидная карта
vis-mode-plots = 📈 Графики
select-plot-label = Выберите график:
vis-mode-deduplicator = 👥 Поиск дубликатов файлов
search-filter-label = 🔍 Фильтр:

# Panel Toggles
toggle-left-panel = { $collapsed ->
    [true] ▶ Показать левую панель (F9)
   *[false] ◀ Скрыть левую панель (F9)
}

toggle-right-panel = { $collapsed ->
    [true] { $is_classic ->
        [true] ◀ Показать правую панель (F11)
       *[false] ▶ Показать панель расширений (F11)
    }
   *[false] { $is_classic ->
        [true] ▶ Скрыть правую панель (F11)
       *[false] ◀ Скрыть панель расширений (F11)
    }
}

collapse-all = ⏏ Свернуть все
about = ℹ О программе
web-not-available = Функция недоступна в веб-версии

# Status Indicators
scanning-disk = Сканирование диска...
scan-complete = Сканирование завершено
scan-cancelled = Сканирование отменено
path-label = Путь: { $path }
worker-threads = ⚡ Рабочие потоки: { $count }
worker-threads-hover = Количество параллельных ядер ЦП с перехватом задач, выделенных для обхода каталогов.

# Stats Panel (Bottom)
directories-count = 📁 Папок: { $count }
files-count = 📄 Файлов: { $count }
total-size = 💾 Общий размер: { $size }
elapsed-time = ⏱ Время: { $time }
scan-speed = ⚡ Скорость: { $speed }/с

# Selection Info
selection-path = Выбрано: { $path }
selection-items = Выбрано: { $count ->
    [one] { $count } элемент
    [few] { $count } элемента
    [many] { $count } элементов
   *[other] { $count } элемента
}

# Plot Types
plot-size-distribution = 📊 Распределение размеров файлов
plot-age-size = 🌌 Возраст и размер файлов
plot-dir-composition = 🍰 Состав папок
plot-extension-boxplot = 📦 Размеры файлов по расширениям
plot-temporal-timeline = ⏱ Связанные временные шкалы
plot-deduplicator-waste = 👥 Потери места от дубликатов по расширениям

# --- Deduplicator Strings ---
dedup-desc = Поиск и безопасное удаление побайтово идентичных файлов с помощью криптографически стойких хешей BLAKE3.
dedup-how-it-works = ℹ Как это работает
dedup-min-size = Мин. размер файла:
dedup-ignore-system = Игнорировать системные файлы
dedup-ignore-hidden = Игнорировать скрытые файлы
dedup-start-scan = ⚡ Начать поиск дубликатов
dedup-scan-first = Сначала просканируйте папку.
dedup-cancelled-msg = Сканирование было отменено. Запустите новое сканирование, чтобы найти дубликаты.
dedup-analyzing = Анализ файлов...
dedup-no-duplicates = Группы дубликатов не найдены. Попробуйте уменьшить минимальный размер файла или просканировать другую папку.
no-permission = Нет доступа
hardlink-badge = Жесткая ссылка
dedup-select-items = 🎯 Выбрать элементы...
dedup-select-all-but-oldest = 🎯 Все, кроме самых старых
dedup-select-all-but-newest = 🎯 Все, кроме самых новых
dedup-select-all-but-shortest = 🎯 Все, кроме самых коротких путей
dedup-select-all-but-rootmost = 🎯 Все, кроме ближайших к корню
dedup-select-all-but-longest = 🎯 Все, кроме самых длинных путей
dedup-pref-dir-pattern = Шаблон предпочтительной папки:
dedup-select-all-but-pref = 🎯 Все, кроме предпочтительной папки
dedup-clear-selection = ❌ Снять выделение
dedup-link-menu = 🔗 Связать... (файлов: { $count })
dedup-link-menu-disabled = 🔗 Связать... (файлов: 0)
dedup-link-hardlinks = 🔗 Заменить выбранные жесткими ссылками
dedup-link-softlinks = 🔗 Заменить выбранные символическими ссылками
dedup-remove-menu = 🗑 Удалить... (файлов: { $count }, { $size })
dedup-remove-menu-disabled = 🗑 Удалить... (файлов: 0)
dedup-remove-trash = ♻ Переместить выбранные в корзину
dedup-remove-delete = 🗑 Удалить выбранные безвозвратно
dedup-warning-title = ⚠ ПРЕДУПРЕЖДЕНИЕ О ПОТЕРЕ ДАННЫХ
dedup-warning-desc = { $count ->
    [one] Удаление всех версий { $count } файла
    [few] Удаление всех версий { $count } файлов
    [many] Удаление всех версий { $count } файлов
   *[other] Удаление всех версий { $count } файлов
}
dedup-warning-no-original = Исходная копия не сохранится:
dedup-warning-details = Для перечисленных ниже файлов вы отметили как исходные, так и все повторяющиеся копии. Их удаление, скорее всего, приведет к безвозвратной потере данных:
dedup-cancel-hover = Нажмите, чтобы отменить сканирование
scan-cancel-hover = Нажмите, чтобы отменить сканирование
dedup-current-label = Текущий
dedup-phase1-size = Этап 1/7: Группировка всех просканированных файлов по размеру...
dedup-phase1-filter = Этап 1/7: Фильтрация кандидатов в дубликаты по исключениям...
dedup-phase2-prefix = Этап 2/7: Хеширование префиксов файлов (первые 4 КБ)...
dedup-phase3-midpoint = Этап 3/7: Хеширование средних частей файлов...
dedup-phase4-suffix = Этап 4/7: Хеширование суффиксов файлов...
dedup-phase5-multirange = Этап 5/7: Многодиапазонное хеширование больших файлов...
dedup-phase6-full = Этап 6/7: Полное хеширование BLAKE3 оставшихся кандидатов...
dedup-phase7-validation = Этап 7/7: Финальная проверка меток времени...
dedup-phase-finished = Завершено за { $duration }! Найдено групп дубликатов: { $count }. Потенциально освобождаемое место: { $space }
dedup-scan-cancelled-with-error = Сканирование было отменено: { $error }

# Deduplicator Table Headers
dedup-hdr-checkbox = [     ]
dedup-hdr-filename = Имя файла
dedup-hdr-directory = Родительская папка
dedup-hdr-size = Размер
dedup-hdr-reclaimable = Освобождаемое место
dedup-hdr-created = Создан
dedup-hdr-modified = Изменен
dedup-copies-selected = ({ $count ->
    [one] выбрана { $count } копия
    [few] выбрано { $count } копии
    [many] выбрано { $count } копий
   *[other] выбрано { $count } копии
})

# --- Explorer Details Panel ---
explorer-details-header = ℹ Сведения
explorer-deselect-hover = Снять выделение с элементов
explorer-deselect-single-hover = Снять выделение с элемента
explorer-selected-items-count = { $count ->
    [one] Выбран { $count } элемент
    [few] Выбрано { $count } элемента
    [many] Выбрано { $count } элементов
   *[other] Выбрано { $count } элемента
}
explorer-total-size = Общий размер: { $size }
explorer-files = Файлов: { $count }
explorer-directories = Папок: { $count }
explorer-actions-title = Действия
explorer-actions-operations = Операции:
explorer-action-refresh-hover = Обновить все поддеревья выбранных папок
explorer-grid-type = Тип:
explorer-grid-size = Размер:
explorer-grid-bytes = Байт:
explorer-grid-items = Элементов:
explorer-grid-files = Файлов:
explorer-grid-subdirs = Подпапок:
explorer-grid-user = Пользователь:
explorer-grid-group = Группа:
explorer-grid-permissions = Права доступа:
explorer-grid-path = Полный путь:

# Explorer Type Names
type-symlink = Символическая ссылка
type-directory = Папка
type-file = Файл

# Explorer Actions
explorer-action-copy-path = 📋 Копировать путь
explorer-action-open-file = 📄 Открыть файл
explorer-action-open-manager = 🗁 Открыть в файловом менеджере
explorer-action-refresh-subtree = 🔄 Обновить поддерево
explorer-action-move-trash = ♻ Переместить в корзину
explorer-action-delete-permanently = 🗑 Удалить безвозвратно
explorer-action-refresh-directory = 🔄 Обновить папку

# Explorer Empty State
explorer-empty-state = Нажмите «Новое сканирование», чтобы изучить использование диска.
choose-an-option = Выберите вариант
web-viewer = Веб-просмотрщик
load-demo = 👁 Загрузить демонстрационный снимок
placeholder-treemap = Здесь просканированная файловая система будет визуализирована в виде древовидной карты.
placeholder-plots = Здесь будут построены графики по просканированной файловой системе.

# Treemap Zoom & Navigation
zoom-up = ⏶ Вверх
zoom-reset = ❌ Сброс
zoom-to-dir = 🔍 Фокус в Treemap
zoom-up-level = ⏶ На один уровень вверх
zoom-empty-dir = Каталог пуст

# --- Extensions Panel ---
extensions-header = 📂 Расширения
extensions-empty = Статистика еще не собрана.
extensions-hover-files = Файлов: { $count }

# --- Operations (Context Actions) ---
op-up-one-level = На уровень вверх
op-zoom-treemap = Фокус в Treemap
op-refresh-entire-scan = Обновить всё сканирование
op-refresh-directory = Обновить папку
op-open-file = Открыть файл
op-open-file-manager = Открыть в файловом менеджере
op-open-terminal = Открыть терминал здесь
op-copy-path = Копировать путь
op-copy-name = Копировать имя
op-move-trash = Переместить в корзину
op-permanently-delete = Удалить безвозвратно

# Toast Notifications
toast-already-root = Вы уже на корневом уровне
toast-navigated-up = Выполнен переход на уровень вверх
toast-zoomed-treemap = Treemap сфокусирован на папке
toast-refreshing-scan = Обновление всего сканирования...
toast-refreshing-dir = Обновление выбранных папок...
toast-opened-file = Открыто: { $path }
toast-failed-open-file = Не удалось открыть файл: { $error }
toast-opened-manager = Открыто в файловом менеджере: { $path }
toast-failed-open-manager = Не удалось открыть в файловом менеджере: { $error }
toast-opened-terminal = Открыт терминал в: { $path }
toast-failed-open-terminal = Не удалось открыть терминал: { $error }
toast-copied-paths = { $count ->
    [one] Скопирован { $count } путь в буфер обмена
    [few] Скопировано { $count } пути в буфер обмена
    [many] Скопировано { $count } путей в буфер обмена
   *[other] Скопировано { $count } пути в буфер обмена
}
toast-copied-names = { $count ->
    [one] Скопировано { $count } имя в буфер обмена
    [few] Скопировано { $count } имени в буфер обмена
    [many] Скопировано { $count } имен в буфер обмена
   *[other] Скопировано { $count } имени в буфер обмена
}

# --- Modals ---
modal-remember-confirmation = Запомнить подтверждение для всех последующих файлов и папок
modal-process-multiple = Вы собираетесь обработать дубликаты файлов/элементов ({ $count }):
modal-process-single = Вы собираетесь обработать следующий путь:
# Confirm Deletion/Trash/Link Modals
modal-delete-title = ⚠ ПРЕДУПРЕЖДЕНИЕ О БЕЗВОЗВРАТНОМ УДАЛЕНИИ
modal-delete-header = ⚠ Предупреждение о безвозвратном удалении!
modal-delete-info = Общий размер: { $size }
modal-delete-warning = Это рекурсивное удаление. Все файлы, папки и подпапки по выбранным путям будут удалены безвозвратно, без возможности восстановления (в обход корзины).
modal-delete-checkbox = Я понимаю, что файлы будут удалены безвозвратно и их невозможно будет восстановить.
modal-delete-confirm = 🗑 Да, удалить безвозвратно

modal-trash-title = ♻ ПЕРЕМЕЩЕНИЕ В КОРЗИНУ
modal-trash-header = ♻ Переместить в корзину
modal-trash-info = Общий размер: { $size }
modal-trash-warning = Выбранные пути и все их содержимое будут перемещены в системную корзину, откуда их позже можно будет восстановить или удалить безвозвратно.
modal-trash-checkbox = Я подтверждаю, что хочу переместить это в корзину.
modal-trash-confirm = ♻ Да, переместить в корзину

modal-delete-duplicates-title = ⚠ ПРЕДУПРЕЖДЕНИЕ О БЕЗВОЗВРАТНОМ УДАЛЕНИИ ДУБЛИКАТОВ
modal-delete-duplicates-header = ⚠ Предупреждение о безвозвратном удалении дубликатов!
modal-delete-duplicates-info = Общий объем освобождаемого пространства: { $size }
modal-delete-duplicates-warning = Все выбранные файлы будут удалены безвозвратно, без возможности восстановления (в обход корзины).
modal-delete-duplicates-checkbox = Я понимаю, что файлы будут удалены безвозвратно и их невозможно будет восстановить.
modal-delete-duplicates-confirm = 🗑 Да, удалить выбранные безвозвратно

modal-trash-duplicates-title = ♻ ПЕРЕМЕЩЕНИЕ ДУБЛИКАТОВ В КОРЗИНУ
modal-trash-duplicates-header = ♻ Переместить дубликаты в корзину
modal-trash-duplicates-info = Общий объем освобождаемого пространства: { $size }
modal-trash-duplicates-warning = Все выбранные файлы будут перемещены в корзину.
modal-trash-duplicates-checkbox = Я подтверждаю, что хочу переместить эти файлы в корзину.
modal-trash-duplicates-confirm = ♻ Да, переместить выбранные в корзину

modal-hardlink-duplicates-title = 🔗 ЗАМЕНА ДУБЛИКАТОВ ЖЕСТКИМИ ССЫЛКАМИ
modal-hardlink-duplicates-header = 🔗 Заменить дубликаты жесткими ссылками
modal-hardlink-duplicates-info = Всего файлов для обработки: { $count }. Суммарный виртуальный размер: { $size }
modal-hardlink-duplicates-warning = Это удалит выбранные дубликаты файлов и заменит их жесткими ссылками на уровне файловой системы, указывающими на оставшийся исходный файл в каждой группе. Файлы останутся видимыми, при этом физическое место на диске будет фактически освобождено.
modal-hardlink-duplicates-checkbox = Я подтверждаю, что хочу заменить выбранные файлы жесткими ссылками.
modal-hardlink-duplicates-confirm = 🔗 Да, заменить жесткими ссылками

modal-softlink-duplicates-title = 🔗 ЗАМЕНА ДУБЛИКАТОВ СИМВОЛИЧЕСКИМИ ССЫЛКАМИ
modal-softlink-duplicates-header = 🔗 Заменить дубликаты символическими ссылками
modal-softlink-duplicates-info = Всего файлов для обработки: { $count }. Суммарный виртуальный размер: { $size }
modal-softlink-duplicates-warning = Это удалит выбранные дубликаты файлов и заменит их символическими ссылками на уровне файловой системы, указывающими на оставшийся исходный файл в каждой группе. Файлы останутся видимыми, при этом физическое место на диске будет фактически освобождено.
modal-softlink-duplicates-checkbox = Я подтверждаю, что хочу заменить выбранные файлы символическими ссылками.
modal-softlink-duplicates-confirm = 🔗 Да, заменить символическими ссылками

# Path Does Not Exist Modal
modal-path-not-exist-title = ❌ Путь не существует!
modal-path-not-exist-msg = Ошибка: путь, который вы пытаетесь удалить, не существует на диске.
modal-close-btn = Закрыть
modal-details-label = Подробности: 
modal-cancel-btn = Отмена

# Elevation Recommended Modal
modal-elevation-title = ⚠ Рекомендуется повышение прав
modal-elevation-desc = По умолчанию eDirStat работает с правами обычного пользователя. Однако Windows строго ограничивает доступ к дескрипторам физических дисков, разрешая его только учетным записям администраторов.
modal-elevation-mft-disabled = Драйвер Windows NTFS MFT отключен
modal-elevation-mft-desc = Без прав администратора сканер MFT с прямым доступом к диску не может быть запущен. Анализ файлов будет использовать резервный стандартный драйвер обхода, что снижает производительность сканирования вплоть до 20 раз.
modal-elevation-relaunch-prompt = Хотите перезапустить приложение с правами администратора сейчас?
modal-elevation-continue-std = Продолжить как обычный пользователь
modal-elevation-relaunch-btn = 🛡 Перезапустить от имени администратора

# About Modal
modal-about-title = ℹ О программе eDirStat
modal-about-author = Автор: Cody Wyatt Neiman (xangelix) <neiman@cody.to>
modal-about-desc1 = Высокопроизводительный анализатор дискового пространства и инструментарий дедупликации, написанный на Rust.
modal-about-desc2 = Возможности: параллельный обход каталогов с перехватом задач, сжатые снимки с десериализацией структуры без парсинга, а также отзывчивые интерактивные древовидные карты.
modal-about-desc3 = Встроенный дедупликатор использует многоэтапный конвейер криптографического хеширования для безопасного выявления групп дубликатов, расчета освобождаемого места и корректного учета жестких ссылок на уровне системы.
modal-about-licenses-btn = Просмотреть лицензии открытого ПО
modal-about-version = v{ $version }

# How Deduplication Works Modal
modal-how-dedup-title = ℹ Как работает дедупликация
modal-how-dedup-desc1 = Вместо прямого сравнения байтов каждого файла (что требует медленных попарных проверок со сложностью O(N²)) эта система использует высокооптимизированный 7-этапный конвейер, чтобы безопасно и эффективно выявлять идентичное содержимое.
modal-how-dedup-pipeline-title = 7-этапный конвейер:
modal-how-dedup-why-title = Почему этого достаточно?
modal-how-dedup-why-desc1 = Такой многоэтапный фильтр гарантирует, что полностью считываются только файлы с одинаковыми размером, префиксом, средней частью, суффиксом и распределенными выборками блоков. Наконец, сравнение 256-битного криптографического хеша BLAKE3 обеспечивает уровень надежности, сопоставимый с промышленными протоколами защищенной передачи данных, и устраняет необходимость медленного попарного побайтового сравнения.

# How Deduplication Works Steps
modal-how-dedup-step1-title = 1. Разбиение по размеру
modal-how-dedup-step1-desc = Файлы группируются по точному размеру в байтах. Любой файл с уникальным размером сразу отбрасывается, полностью исключая дисковые операции ввода-вывода.
modal-how-dedup-step2-title = 2. Хеширование префиксов
modal-how-dedup-step2-desc = Хешируются первые 4 КБ оставшихся кандидатов. Это быстро отсеивает файлы с разными заголовками или форматами метаданных.
modal-how-dedup-step3-title = 3. Хеширование средней части
modal-how-dedup-step3-desc = Хешируется блок 4 КБ из середины оставшихся файлов, что выявляет внутренние структурные различия.
modal-how-dedup-step4-title = 4. Хеширование суффиксов
modal-how-dedup-step4-desc = Хешируются последние 4 КБ данных. Это очень эффективно для выявления различий в конечном содержимом или метаданных.
modal-how-dedup-step5-title = 5. Многодиапазонное хеширование
modal-how-dedup-step5-desc = Для больших файлов (более 100 МБ) выполняется периодическая выборка блоков по всей длине, что позволяет проверить целостность содержимого без чтения всего файла.
modal-how-dedup-step6-title = 6. Полное хеширование BLAKE3
modal-how-dedup-step6-desc = Для оставшихся кандидатов вычисляется полный криптографический хеш BLAKE3. Благодаря высокой устойчивости к коллизиям 256-битного пространства совпадение хешей означает астрономически малую вероятность различий между файлами, что дает высоконадежное доказательство идентичности без попарных сравнений.
modal-how-dedup-step7-title = 7. Проверка меток времени
modal-how-dedup-step7-desc = Непосредственно перед отображением или выполнением любого действия дедупликации приложение проверяет метки времени файлов на диске, чтобы защититься от изменений, произошедших с момента создания снимка.

# Open Source Licenses Modal
modal-licenses-title = 📜 Лицензии открытого ПО
modal-licenses-desc = В этом приложении используются следующие сторонние библиотеки и крейты:

# Processing Modal
modal-processing-title = ⏳ Обработка...
modal-processing-deletion = Удаление файлов и папок...
modal-processing-trash = Перемещение файлов и папок в корзину...
modal-processing-hardlink = Замена дубликатов жесткими ссылками...
modal-processing-softlink = Замена дубликатов символическими ссылками...

# Explorer Column Headers
explorer-hdr-name = Имя
explorer-hdr-percentage = Процент
explorer-hdr-size = Размер
explorer-hdr-items = Элементов
explorer-hdr-files = Файлов
explorer-hdr-subdirs = Подпапок
explorer-hdr-created = Создан
explorer-hdr-modified = Изменен

# Update Checker
update-checking = Проверка обновлений...
update-available = Доступна новая версия { $version }!
update-up-to-date = У вас установлена последняя версия
update-failed = Не удалось проверить наличие обновлений: { $error }

# Themes
theme = 🎨 Тема
theme-dark = Темная
theme-high-contrast = Высококонтрастная
theme-light = Светлая
theme-system = Системная

# New Scan Options Modal
modal-scan-options-title = Параметры нового сканирования
modal-scan-options-header = Начать новое сканирование
modal-scan-options-path-label = Путь к папке для сканирования:
modal-scan-options-paste-tooltip = Вставить из буфера обмена
modal-scan-options-browse-tooltip = Выбрать папку...
modal-scan-options-scan-btn = Сканировать
modal-scan-options-cancel-btn = Отмена
modal-scan-options-same-filesystem = Ограничить сканирование одной файловой системой/томом
modal-scan-options-drives-header = 💽 Диски и тома
modal-scan-options-refresh-tooltip = Обновить список дисков
modal-scan-options-root-system = Корневая файловая система
modal-scan-options-selected-badge = ✅ Выбрано
modal-scan-options-free-of = свободно { $free } из { $total }
modal-scan-options-subtitle = Выберите том накопителя, быстрый путь или пользовательскую папку для анализа.
modal-scan-options-quick-access = 📍 Быстрый доступ
modal-scan-options-path-hint = /путь/к/папке
modal-scan-options-hint = ℹ Выберите диск выше или введите путь к каталогу.
modal-scan-options-sandbox-auth = 🔒 Требуется доступ в песочнице — нажмите «Сканировать» для предоставления доступа
modal-scan-options-valid-dir = ✅ Корректный каталог — готов к сканированию
modal-scan-options-points-to-file = ⚠ Путь указывает на файл — выберите папку.
modal-scan-options-dir-not-exist = ⚠ Каталог не существует в файловой системе.
quick-loc-home = 🏠 Домашняя папка
quick-loc-documents = 📄 Документы
quick-loc-downloads = 📥 Загрузки
quick-loc-desktop = 🖥 Рабочий стол
quick-loc-pictures = 🖼 Изображения
search-use-regex = Использовать регулярные выражения (Regex)
search-match-case = С учётом регистра
dedup-pref-dir-hint = напр. /home/user/Archive

file-menu-close = Закрыть сканирование
file-menu-quit = Выйти

badge-dataless-cloud = Облачный файл / без локальных данных
badge-symlink = Символическая ссылка
badge-special-file = Специальный файл (Канал / Сокет / Устройство)
badge-permission-denied = В доступе отказано
