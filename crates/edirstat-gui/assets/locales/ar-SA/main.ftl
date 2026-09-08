# Menu Bar Dropdowns
file = ملف
view = عرض
help = تعليمات

# Menu Bar Actions
new-scan = 📁 فحص جديد
save-snapshot = 💾 حفظ لقطة
load-snapshot = 📖 تحميل لقطة

# Menu Bar Status
idle = خامل

# View Menu Options
monospace-paths = 🅰 مسارات بخط أحادي المسافة
highlight-duplicates = ✨ تمييز الملفات المكررة
treemap-borders = 🔳 حدود الخريطة الشجرية
treemap-style =  نمط الخريطة الشجرية
treemap-style-vertical = تدرج عمودي
treemap-style-offset-vertical = تدرج عمودي مُزاح
treemap-style-diagonal = تدرج قطري
treemap-style-cushion = تظليل وسائدي
deletion-confirmation = 🗑 تأكيد الحذف
trash-confirmation = ♻ تأكيد النقل إلى سلة المهملات
time-format = 🕒 تنسيق الوقت
language = 💬 اللغة
layout-mode = وضع التخطيط:
classic-layout = التخطيط الكلاسيكي
windirstat-layout = تخطيط WinDirStat
vis-mode-treemap = 📊 الخريطة الشجرية
vis-mode-plots = 📈 الرسوم البيانية
select-plot-label = اختر الرسم البياني:
vis-mode-deduplicator = 👥 كاشف الملفات المكررة
search-filter-label = 🔍 تصفية:

# Panel Toggles
toggle-left-panel = { $collapsed ->
    [true] ▶ إظهار اللوحة اليسرى (F9)
   *[false] ◀ إخفاء اللوحة اليسرى (F9)
}

toggle-right-panel = { $collapsed ->
    [true] { $is_classic ->
        [true] ◀ إظهار اللوحة اليمنى (F11)
       *[false] ▶ إظهار لوحة الامتدادات (F11)
    }
   *[false] { $is_classic ->
        [true] ▶ إخفاء اللوحة اليمنى (F11)
       *[false] ◀ إخفاء لوحة الامتدادات (F11)
    }
}

collapse-all = ⏏ طي الكل
about = ℹ حول
web-not-available = الميزة غير متوفرة في إصدار الويب

# Status Indicators
scanning-disk = جارٍ فحص القرص...
scan-complete = اكتمل الفحص
scan-cancelled = تم إلغاء الفحص
path-label = المسار: { $path }
worker-threads = ⚡ { $count } مؤشرات ترابط العمل
worker-threads-hover = عدد أنوية المعالج المتوازية العاملة بمبدأ «سرقة المهام» والمخصصة لاجتياز المجلدات.

# Stats Panel (Bottom)
directories-count = 📁 المجلدات: { $count }
files-count = 📄 الملفات: { $count }
total-size = 💾 الحجم الإجمالي: { $size }
elapsed-time = ⏱ الوقت: { $time }
scan-speed = ⚡ السرعة: { $speed }/s

# Selection Info
selection-path = التحديد: { $path }
selection-items = التحديد: { $count ->
    [zero] لا عناصر
    [one] عنصر واحد
    [two] عنصران
    [few] { $count } عناصر
    [many] { $count } عنصرًا
   *[other] { $count } عنصر
}

# Plot Types
plot-size-distribution = 📊 توزيع أحجام الملفات
plot-age-size = 🌌 عمر الملف مقابل حجم الملف
plot-dir-composition = 🍰 تكوين المجلد
plot-extension-boxplot = 📦 أحجام الملفات حسب الامتداد
plot-temporal-timeline = ⏱ خطوط زمنية مترابطة
plot-deduplicator-waste = 👥 الهدر من التكرار حسب الامتداد

# --- Deduplicator Strings ---
dedup-desc = اعثر على الملفات المتطابقة بايتًا بايت وأزلها بأمان باستخدام تجزئات BLAKE3 الآمنة تشفيريًا.
dedup-how-it-works = ℹ كيف يعمل
dedup-min-size = الحد الأدنى لحجم الملف:
dedup-ignore-system = تجاهل ملفات النظام
dedup-ignore-hidden = تجاهل الملفات المخفية
dedup-start-scan = ⚡ بدء فحص إزالة التكرار
dedup-scan-first = الرجاء فحص مجلد أولًا.
dedup-cancelled-msg = تم إلغاء الفحص. ابدأ فحصًا جديدًا للعثور على التكرارات.
dedup-analyzing = جارٍ تحليل الملفات...
dedup-no-duplicates = لم يتم العثور على مجموعات مكررة. جرّب تقليل الحد الأدنى لحجم الملف أو فحص مجلد مختلف.
no-permission = لا يوجد إذن
hardlink-badge = ارتباط ثابت
dedup-select-items = 🎯 تحديد العناصر...
dedup-select-all-but-oldest = 🎯 الكل ما عدا الأقدم
dedup-select-all-but-newest = 🎯 الكل ما عدا الأحدث
dedup-select-all-but-shortest = 🎯 الكل ما عدا الأقصر مسارًا
dedup-select-all-but-rootmost = 🎯 الكل ما عدا الأقرب إلى الجذر
dedup-select-all-but-longest = 🎯 الكل ما عدا الأطول مسارًا
dedup-pref-dir-pattern = نمط المجلد المفضل:
dedup-select-all-but-pref = 🎯 الكل ما عدا المجلد المفضل
dedup-clear-selection = ❌ مسح التحديد
dedup-link-menu = 🔗 ربط... ({ $count } ملفات)
dedup-link-menu-disabled = 🔗 ربط... (0 ملفات)
dedup-link-hardlinks = 🔗 استبدال المحدد بارتباطات ثابتة
dedup-link-softlinks = 🔗 استبدال المحدد بارتباطات رمزية
dedup-remove-menu = 🗑 إزالة... ({ $count } ملفات، { $size })
dedup-remove-menu-disabled = 🗑 إزالة... (0 ملفات)
dedup-remove-trash = ♻ نقل المحدد إلى سلة المهملات
dedup-remove-delete = 🗑 حذف المحدد نهائيًا
dedup-warning-title = ⚠ تحذير من فقدان البيانات
dedup-warning-desc = { $count ->
    [one] حذف جميع نسخ ملف واحد
    [two] حذف جميع نسخ ملفين
    [few] حذف جميع نسخ { $count } ملفات
    [many] حذف جميع نسخ { $count } ملفًا
   *[other] حذف جميع نسخ { $count } ملف
}
dedup-warning-no-original = لن تتبقى أي نسخة أصلية:
dedup-warning-details = لقد حددت كلاً من النسخة الأصلية وجميع النسخ المكررة للملفات المدرجة أدناه. سيؤدي حذفها على الأرجح إلى فقدان دائم للبيانات:
dedup-cancel-hover = انقر لإلغاء الفحص
scan-cancel-hover = انقر لإلغاء الفحص
dedup-current-label = الحالي
dedup-phase1-size = المرحلة 1/7: تجميع جميع الملفات المفحوصة حسب الحجم...
dedup-phase1-filter = المرحلة 1/7: تصفية الاستثناءات على المرشحين للتكرار...
dedup-phase2-prefix = المرحلة 2/7: تجزئة بادئات الملفات (أول 4KB)...
dedup-phase3-midpoint = المرحلة 3/7: تجزئة نقاط منتصف الملفات...
dedup-phase4-suffix = المرحلة 4/7: تجزئة لواحق الملفات...
dedup-phase5-multirange = المرحلة 5/7: تجزئة متعددة النطاقات للملفات الكبيرة...
dedup-phase6-full = المرحلة 6/7: تجزئة BLAKE3 الكاملة للمرشحين المتبقين...
dedup-phase7-validation = المرحلة 7/7: التحقق النهائي من الطوابع الزمنية...
dedup-phase-finished = اكتمل في { $duration }! تم العثور على { $count } مجموعات مكررة. المساحة القابلة للاسترداد المحتملة: { $space }
dedup-scan-cancelled-with-error = تم إلغاء الفحص: { $error }

# Deduplicator Table Headers
dedup-hdr-checkbox = [     ]
dedup-hdr-filename = اسم الملف
dedup-hdr-directory = المجلد الأصل
dedup-hdr-size = الحجم
dedup-hdr-reclaimable = قابل للاسترداد
dedup-hdr-created = تاريخ الإنشاء
dedup-hdr-modified = تاريخ التعديل
dedup-copies-selected = ({ $count ->
    [zero] لا نُسخ محددة
    [one] نسخة واحدة محددة
    [two] نسختان محددتان
    [few] { $count } نسخ محددة
    [many] { $count } نسخةً محددةً
   *[other] { $count } نسخة محددة
})

# --- Explorer Details Panel ---
explorer-details-header = ℹ التفاصيل
explorer-deselect-hover = إلغاء تحديد العناصر
explorer-deselect-single-hover = إلغاء تحديد العنصر
explorer-selected-items-count = { $count ->
    [zero] لا عناصر محددة
    [one] عنصر محدد واحد
    [two] عنصران محددان
    [few] { $count } عناصر محددة
    [many] { $count } عنصرًا محددًا
   *[other] { $count } عنصر محدد
}
explorer-total-size = الحجم الإجمالي: { $size }
explorer-files = الملفات: { $count }
explorer-directories = المجلدات: { $count }
explorer-actions-title = الإجراءات
explorer-actions-operations = العمليات:
explorer-action-refresh-hover = تحديث جميع الأشجار الفرعية للمجلدات المحددة
explorer-grid-type = النوع:
explorer-grid-size = الحجم:
explorer-grid-bytes = بايت:
explorer-grid-items = العناصر:
explorer-grid-files = الملفات:
explorer-grid-subdirs = المجلدات الفرعية:
explorer-grid-user = المستخدم:
explorer-grid-group = المجموعة:
explorer-grid-permissions = الأذونات:
explorer-grid-path = المسار الكامل:

# Explorer Type Names
type-symlink = ارتباط رمزي
type-directory = مجلد
type-file = ملف

# Explorer Actions
explorer-action-copy-path = 📋 نسخ المسار
explorer-action-open-file = 📄 فتح الملف
explorer-action-open-manager = 🗁 فتح مدير الملفات
explorer-action-refresh-subtree = 🔄 تحديث الشجرة الفرعية
explorer-action-move-trash = ♻ نقل إلى سلة المهملات
explorer-action-delete-permanently = 🗑 حذف نهائيًا
explorer-action-refresh-directory = 🔄 تحديث المجلد

# Explorer Empty State
explorer-empty-state = انقر على «فحص جديد» لاستكشاف استخدام القرص.
choose-an-option = اختر خيارًا
web-viewer = عارض الويب
load-demo = 👁 تحميل لقطة تجريبية نموذجية
placeholder-treemap = سيتم عرض نظام الملفات المفحوص هنا كخريطة شجرية.
placeholder-plots = سيتم رسم نظام الملفات المفحوص هنا.

# Treemap Zoom & Navigation
zoom-up = ⏶ لأعلى
zoom-reset = ❌ إعادة تعيين
zoom-to-dir = 🔍 التركيز في الخريطة الشجرية
zoom-up-level = ⏶ الانتقال إلى مستوى أعلى
zoom-empty-dir = المجلد فارغ

# --- Extensions Panel ---
extensions-header = 📂 الامتدادات
extensions-empty = لم يتم جمع إحصاءات بعد.
extensions-hover-files = الملفات: { $count }

# --- Operations (Context Actions) ---
op-up-one-level = الانتقال إلى مستوى أعلى
op-zoom-treemap = التركيز في الخريطة الشجرية
op-refresh-entire-scan = تحديث الفحص بالكامل
op-refresh-directory = تحديث المجلد
op-open-file = فتح الملف
op-open-file-manager = فتح في مدير الملفات
op-open-terminal = فتح الطرفية هنا
op-copy-path = نسخ المسار
op-copy-name = نسخ الاسم
op-move-trash = نقل إلى سلة المهملات
op-permanently-delete = حذف نهائي

# Toast Notifications
toast-already-root = أنت بالفعل في المستوى الجذر
toast-navigated-up = تم الانتقال إلى مستوى أعلى
toast-zoomed-treemap = تم تركيز الخريطة الشجرية على المجلد
toast-refreshing-scan = جارٍ تحديث الفحص بالكامل...
toast-refreshing-dir = جارٍ تحديث المجلد/المجلدات المحددة...
toast-opened-file = تم فتح: { $path }
toast-failed-open-file = فشل فتح الملف: { $error }
toast-opened-manager = تم الفتح في مدير الملفات: { $path }
toast-failed-open-manager = فشل الفتح في مدير الملفات: { $error }
toast-opened-terminal = تم فتح الطرفية في: { $path }
toast-failed-open-terminal = فشل فتح الطرفية: { $error }
toast-copied-paths = تم نسخ { $count ->
    [one] مسار واحد إلى الحافظة
    [two] مسارين إلى الحافظة
    [few] { $count } مسارات إلى الحافظة
    [many] { $count } مسارًا إلى الحافظة
   *[other] { $count } مسار إلى الحافظة
}
toast-copied-names = تم نسخ { $count ->
    [one] اسم واحد إلى الحافظة
    [two] اسمين إلى الحافظة
    [few] { $count } أسماء إلى الحافظة
    [many] { $count } اسمًا إلى الحافظة
   *[other] { $count } اسم إلى الحافظة
}

# --- Modals ---
modal-remember-confirmation = تذكّر التأكيد لجميع الملفات والمجلدات مستقبلًا
modal-process-multiple = أنت على وشك معالجة { $count } من الملفات/العناصر المكررة:
modal-process-single = أنت على وشك معالجة المسار التالي:
# Confirm Deletion/Trash/Link Modals
modal-delete-title = ⚠ تحذير الحذف النهائي
modal-delete-header = ⚠ تحذير الحذف النهائي!
modal-delete-info = الحجم الإجمالي: { $size }
modal-delete-warning = هذا حذف تكراري. سيتم حذف جميع الملفات والمجلدات والمجلدات الفرعية ضمن المسارات المحددة نهائيًا ولا يمكن استردادها (دون المرور بسلة المهملات).
modal-delete-checkbox = أفهم أن الملفات سيتم حذفها نهائيًا ولا يمكن استردادها.
modal-delete-confirm = 🗑 نعم، احذف نهائيًا

modal-trash-title = ♻ نقل إلى سلة المهملات
modal-trash-header = ♻ نقل إلى سلة المهملات
modal-trash-info = الحجم الإجمالي: { $size }
modal-trash-warning = سيؤدي هذا إلى نقل المسارات المحددة وجميع محتوياتها إلى سلة المهملات في نظامك، حيث يمكن استردادها أو حذفها نهائيًا لاحقًا.
modal-trash-checkbox = أؤكد أنني أريد نقل هذا إلى سلة المهملات.
modal-trash-confirm = ♻ نعم، انقل إلى سلة المهملات

modal-delete-duplicates-title = ⚠ تحذير الحذف النهائي للتكرارات
modal-delete-duplicates-header = ⚠ تحذير الحذف النهائي للملفات المكررة!
modal-delete-duplicates-info = إجمالي المساحة التي سيتم استردادها: { $size }
modal-delete-duplicates-warning = سيتم حذف جميع الملفات المحددة نهائيًا ولا يمكن استردادها (دون المرور بسلة المهملات).
modal-delete-duplicates-checkbox = أفهم أن الملفات سيتم حذفها نهائيًا ولا يمكن استردادها.
modal-delete-duplicates-confirm = 🗑 نعم، احذف المحدد نهائيًا

modal-trash-duplicates-title = ♻ نقل التكرارات إلى سلة المهملات
modal-trash-duplicates-header = ♻ نقل التكرارات إلى سلة المهملات
modal-trash-duplicates-info = إجمالي المساحة التي سيتم استردادها: { $size }
modal-trash-duplicates-warning = سيتم نقل جميع الملفات المحددة إلى سلة المهملات.
modal-trash-duplicates-checkbox = أؤكد أنني أريد نقل هذه الملفات إلى سلة المهملات.
modal-trash-duplicates-confirm = ♻ نعم، انقل المحدد إلى سلة المهملات

modal-hardlink-duplicates-title = 🔗 استبدال التكرارات بارتباطات ثابتة
modal-hardlink-duplicates-header = 🔗 استبدال التكرارات بارتباطات ثابتة
modal-hardlink-duplicates-info = إجمالي الملفات المطلوب معالجتها: { $count }. الحجم الافتراضي التراكمي: { $size }
modal-hardlink-duplicates-warning = سيؤدي هذا إلى حذف الملفات المكررة المحددة واستبدالها بارتباطات ثابتة على مستوى نظام الملفات تشير إلى الملف الأصلي المتبقي في كل مجموعة. يحافظ هذا على بقاء الملفات ظاهرة مع تحرير مساحة التخزين الفعلية.
modal-hardlink-duplicates-checkbox = أؤكد أنني أريد استبدال الملفات المحددة بارتباطات ثابتة.
modal-hardlink-duplicates-confirm = 🔗 نعم، استبدل بارتباطات ثابتة

modal-softlink-duplicates-title = 🔗 استبدال التكرارات بارتباطات رمزية
modal-softlink-duplicates-header = 🔗 استبدال التكرارات بارتباطات رمزية
modal-softlink-duplicates-info = إجمالي الملفات المطلوب معالجتها: { $count }. الحجم الافتراضي التراكمي: { $size }
modal-softlink-duplicates-warning = سيؤدي هذا إلى حذف الملفات المكررة المحددة واستبدالها بارتباطات مرنة (ارتباطات رمزية) على مستوى نظام الملفات تشير إلى الملف الأصلي المتبقي في كل مجموعة. يحافظ هذا على بقاء الملفات ظاهرة مع تحرير مساحة التخزين الفعلية.
modal-softlink-duplicates-checkbox = أؤكد أنني أريد استبدال الملفات المحددة بارتباطات رمزية.
modal-softlink-duplicates-confirm = 🔗 نعم، استبدل بارتباطات رمزية

# Path Does Not Exist Modal
modal-path-not-exist-title = ❌ المسار غير موجود!
modal-path-not-exist-msg = خطأ: المسار الذي تحاول حذفه غير موجود على القرص.
modal-close-btn = إغلاق
modal-details-label = التفاصيل: 
modal-cancel-btn = إلغاء

# Elevation Recommended Modal
modal-elevation-title = ⚠ يُوصى برفع الامتيازات
modal-elevation-desc = يعمل eDirStat افتراضيًا بامتيازات المستخدم القياسية. ومع ذلك، يقيّد Windows بشدة الوصول المباشر إلى مؤشرات الأقراص الفعلية الخام، فيحصره في حسابات المسؤولين.
modal-elevation-mft-disabled = تم تعطيل برنامج تشغيل MFT الخاص بنظام NTFS في Windows
modal-elevation-mft-desc = بدون امتيازات المسؤول، لا يمكن لماسح MFT المباشر إلى القرص أن يبدأ. سيستخدم تحليل الملفات برنامج تشغيل الاجتياز القياسي الاحتياطي بدلًا منه، مما يقلل أداء الفحص بنسبة قد تصل إلى 20 ضعفًا.
modal-elevation-relaunch-prompt = هل تريد إعادة تشغيل التطبيق بامتيازات المسؤول الآن؟
modal-elevation-continue-std = المتابعة كمستخدم قياسي
modal-elevation-relaunch-btn = 🛡 إعادة التشغيل كمسؤول

# About Modal
modal-about-title = ℹ حول eDirStat
modal-about-author = تأليف: Cody Wyatt Neiman (xangelix) <neiman@cody.to>
modal-about-license-btn = 📜 الترخيص (MIT)
modal-about-desc1 = أداة عالية الأداء لتحليل مساحة القرص وإزالة التكرارات، مبنية بلغة Rust.
modal-about-desc2 = تتميز باجتياز متوازٍ للمجلدات بمبدأ «سرقة المهام»، ولقطات مضغوطة مع إلغاء تسلسل التخطيط دون تحليل، وخرائط شجرية تفاعلية سريعة الاستجابة.
modal-about-desc3 = يشغّل مُزيل التكرارات المدمج خط أنابيب تجزئة تشفيرية متعدد المراحل لعزل مجموعات التكرارات بأمان، وحساب المساحة القابلة للاسترداد، واحترام الارتباطات الثابتة على مستوى النظام.
modal-about-licenses-btn = عرض تراخيص المصدر المفتوح
modal-about-version = v{ $version }

# How Deduplication Works Modal
modal-how-dedup-title = ℹ كيف تعمل إزالة التكرار
modal-how-dedup-desc1 = بدلًا من مقارنة بايتات كل ملف مباشرةً (وهو ما يتطلب فحوصات زوجية بطيئة من رتبة O(N²))، يستخدم هذا النظام خط أنابيب محسّنًا للغاية من 7 مراحل لتحديد المحتوى المتطابق بأمان وكفاءة.
modal-how-dedup-pipeline-title = خط الأنابيب ذو المراحل السبع:
modal-how-dedup-why-title = لماذا يكفي هذا؟
modal-how-dedup-why-desc1 = يضمن هذا المرشح متعدد المراحل ألا تُقرأ بالكامل إلا الملفات المتطابقة في الحجم والبادئة ونقطة المنتصف واللاحقة وعينات الكتل الموزعة. وأخيرًا، توفر مقارنة تجزئة BLAKE3 التشفيرية ذات 256-bit مستوى أمان يضاهي بروتوكولات النقل الآمنة المعتمدة في الصناعة، مما يلغي الحاجة إلى مقارنات زوجية بطيئة بايتًا بايت.

# How Deduplication Works Steps
modal-how-dedup-step1-title = 1. التقسيم حسب الحجم
modal-how-dedup-step1-desc = تُجمَّع الملفات حسب حجمها الدقيق بالبايت. يُستبعد فورًا أي ملف ذي حجم فريد، متجاوزًا عمليات إدخال/إخراج القرص بالكامل.
modal-how-dedup-step2-title = 2. تجزئة البادئة
modal-how-dedup-step2-desc = تُجزَّأ أول 4KB من المرشحين المتبقين. يرشّح هذا بسرعة الملفات ذات الترويسات أو تنسيقات البيانات الوصفية المختلفة.
modal-how-dedup-step3-title = 3. تجزئة نقطة المنتصف
modal-how-dedup-step3-desc = تُجزَّأ كتلة بحجم 4KB من منتصف الملفات المتبقية، مما يلتقط الاختلافات البنيوية الداخلية.
modal-how-dedup-step4-title = 4. تجزئة اللاحقة
modal-how-dedup-step4-desc = تُجزَّأ آخر 4KB من البيانات. وهذا فعال للغاية في تحديد الاختلافات في المحتويات الختامية أو البيانات الوصفية.
modal-how-dedup-step5-title = 5. التجزئة متعددة النطاقات
modal-how-dedup-step5-desc = تخضع الملفات الكبيرة (التي يتجاوز حجمها 100MB) لأخذ عينات كتل دورية عبر كامل طولها للتحقق من اتساق المحتوى دون قراءة الملف بأكمله.
modal-how-dedup-step6-title = 6. تجزئة BLAKE3 الكاملة
modal-how-dedup-step6-desc = للمرشحين المتبقين، تُحسب تجزئة BLAKE3 التشفيرية كاملةً. ونظرًا للمقاومة العالية للتصادم في فضاء 256-bit، فإن تطابق التجزئات يعني أن احتمال اختلاف الملفات شبه معدوم فلكيًا، مما يوفر دليل تطابق موثوقًا للغاية دون الحاجة إلى مقارنات زوجية.
modal-how-dedup-step7-title = 7. التحقق من الطوابع الزمنية
modal-how-dedup-step7-desc = مباشرةً قبل عرض أو تنفيذ أي إجراء من إجراءات إزالة التكرار، يتحقق التطبيق من الطوابع الزمنية للملفات على القرص للحماية من التغييرات التي طرأت منذ إنشاء اللقطة.

# Open Source Licenses Modal
modal-licenses-title = 📜 تراخيص المصدر المفتوح
modal-licenses-tab-app = eDirStat (MIT)
modal-licenses-tab-deps = مكتبات الطرف الثالث
modal-licenses-app-desc = eDirStat هو برنامج مفتوح المصدر يتم توزيعه بموجب ترخيص MIT:
modal-licenses-desc = تُستخدم المكتبات والحزم الخارجية التالية في هذا التطبيق:
modal-licenses-copy-btn = 📋 نسخ الترخيص
modal-licenses-copy-all-btn = 📋 نسخ التراخيص

# Processing Modal
modal-processing-title = ⏳ جارٍ المعالجة...
modal-processing-deletion = جارٍ حذف الملفات والمجلدات...
modal-processing-trash = جارٍ نقل الملفات والمجلدات إلى سلة المهملات...
modal-processing-hardlink = جارٍ استبدال التكرارات بارتباطات ثابتة...
modal-processing-softlink = جارٍ استبدال التكرارات بارتباطات رمزية...

# Explorer Column Headers
explorer-hdr-name = الاسم
explorer-hdr-percentage = النسبة المئوية
explorer-hdr-size = الحجم
explorer-hdr-items = العناصر
explorer-hdr-files = الملفات
explorer-hdr-subdirs = المجلدات الفرعية
explorer-hdr-created = تاريخ الإنشاء
explorer-hdr-modified = تاريخ التعديل

# Update Checker
update-checking = جارٍ التحقق من التحديثات...
update-available = يتوفر إصدار جديد { $version }!
update-up-to-date = لديك أحدث إصدار
update-failed = فشل التحقق من التحديث: { $error }

# Themes
theme = 🎨 السمة
theme-dark = داكنة
theme-high-contrast = تباين عالٍ
theme-light = فاتحة
theme-system = النظام

# New Scan Options Modal
modal-scan-options-title = خيارات الفحص الجديد
modal-scan-options-header = بدء فحص جديد
modal-scan-options-path-label = مسار المجلد المراد فحصه:
modal-scan-options-paste-tooltip = لصق من الحافظة
modal-scan-options-browse-tooltip = استعراض مجلد...
modal-scan-options-scan-btn = فحص
modal-scan-options-cancel-btn = إلغاء
modal-scan-options-same-filesystem = تقييد الفحص بنفس نظام الملفات/وحدة التخزين
modal-scan-options-drives-header = 💽 محركات الأقراص ووحدات التخزين
modal-scan-options-refresh-tooltip = تحديث محركات أقراص التخزين
modal-scan-options-root-system = نظام الملفات الجذر
modal-scan-options-selected-badge = ✅ محدد
modal-scan-options-free-of = { $free } متاح من { $total }
modal-scan-options-subtitle = اختر وحدة تخزين أو موقعًا سريعًا أو مجلدًا مخصصًا لتحليله.
modal-scan-options-quick-access = 📍 اختصارات الوصول السريع
modal-scan-options-path-hint = /path/to/scan
modal-scan-options-hint = ℹ اختر محرك أقراص أعلاه أو أدخل مسار مجلد.
modal-scan-options-sandbox-auth = 🔒 مطلوب إذن الوصول إلى وضع الحماية — انقر «فحص» لمنح الوصول
modal-scan-options-valid-dir = ✅ مجلد صالح — جاهز للفحص
modal-scan-options-points-to-file = ⚠ يشير المسار إلى ملف — الرجاء تحديد مجلد.
modal-scan-options-dir-not-exist = ⚠ المجلد غير موجود على نظام الملفات.
quick-loc-home = 🏠 الرئيسية
quick-loc-documents = 📄 المستندات
quick-loc-downloads = 📥 التنزيلات
quick-loc-desktop = 🖥 سطح المكتب
quick-loc-pictures = 🖼 الصور
search-use-regex = استخدام تعبير عادي (Regex)
search-match-case = مطابقة حالة الأحرف (حساس لحالة الأحرف)
dedup-pref-dir-hint = مثال: /home/user/Archive

file-menu-close = إغلاق الفحص
file-menu-quit = إنهاء

badge-dataless-cloud = ملف سحابي / بلا بيانات
badge-symlink = ارتباط رمزي
badge-special-file = ملف خاص (أنبوب / مقبس / جهاز)
badge-permission-denied = تم رفض الوصول
