# Menu Bar Dropdowns
file = ফাইল
view = ভিউ
help = সহায়তা

# Menu Bar Actions
new-scan = 📁 নতুন স্ক্যান
save-snapshot = 💾 স্ন্যাপশট সংরক্ষণ করুন
load-snapshot = 📖 স্ন্যাপশট লোড করুন

# Menu Bar Status
idle = নিষ্ক্রিয়

# View Menu Options
monospace-paths = 🅰 মোনোস্পেস পাথ
highlight-duplicates = ✨ ডুপ্লিকেট হাইলাইট করুন
treemap-borders = 🔳 ট্রিম্যাপ বর্ডার
treemap-style =  ট্রিম্যাপ স্টাইল
treemap-style-vertical = উল্লম্ব গ্রেডিয়েন্ট
treemap-style-offset-vertical = অফসেট উল্লম্ব গ্রেডিয়েন্ট
treemap-style-diagonal = তির্যক গ্রেডিয়েন্ট
treemap-style-cushion = কুশন শেডিং
deletion-confirmation = 🗑 মুছে ফেলার নিশ্চিতকরণ
trash-confirmation = ♻ ট্র্যাশ নিশ্চিতকরণ
time-format = 🕒 সময়ের ফর্ম্যাট
language = 💬 ভাষা
layout-mode = লেআউট মোড:
classic-layout = ক্লাসিক লেআউট
windirstat-layout = WinDirStat লেআউট
vis-mode-treemap = 📊 ট্রিম্যাপ
vis-mode-plots = 📈 প্লট
select-plot-label = প্লট নির্বাচন করুন:
vis-mode-deduplicator = 👥 ডুপ্লিকেট ফাইল অনুসন্ধানকারী
search-filter-label = 🔍 ফিল্টার:

# Panel Toggles
toggle-left-panel = { $collapsed ->
    [true] ▶ বাম প্যানেল দেখান (F9)
   *[false] ◀ বাম প্যানেল লুকান (F9)
}

toggle-right-panel = { $collapsed ->
    [true] { $is_classic ->
        [true] ◀ ডান প্যানেল দেখান (F11)
       *[false] ▶ এক্সটেনশন প্যানেল দেখান (F11)
    }
   *[false] { $is_classic ->
        [true] ▶ ডান প্যানেল লুকান (F11)
       *[false] ◀ এক্সটেনশন প্যানেল লুকান (F11)
    }
}

collapse-all = ⏏ সব ভাঁজ করুন
about = ℹ সম্পর্কে
web-not-available = ওয়েব সংস্করণে এই ফিচারটি উপলব্ধ নয়

# Status Indicators
scanning-disk = ডিস্ক স্ক্যান করা হচ্ছে...
scan-complete = স্ক্যান সম্পন্ন
scan-cancelled = স্ক্যান বাতিল করা হয়েছে
path-label = পাথ: { $path }
worker-threads = ⚡ { $count } ওয়ার্কার থ্রেড
worker-threads-hover = ডিরেক্টরি ট্রাভার্সালের জন্য বরাদ্দ করা সমান্তরাল, ওয়ার্ক-স্টিলিং CPU কোরের সংখ্যা।

# Stats Panel (Bottom)
directories-count = 📁 ডিরেক্টরি: { $count }
files-count = 📄 ফাইল: { $count }
total-size = 💾 মোট আকার: { $size }
elapsed-time = ⏱ সময়: { $time }
scan-speed = ⚡ গতি: { $speed }/s

# Selection Info
selection-path = নির্বাচন: { $path }
selection-items = নির্বাচন: { $count ->
    [one] 1টি আইটেম
   *[other] { $count }টি আইটেম
}

# Plot Types
plot-size-distribution = 📊 ফাইলের আকারের বণ্টন
plot-age-size = 🌌 ফাইলের বয়স বনাম ফাইলের আকার
plot-dir-composition = 🍰 ডিরেক্টরির গঠন
plot-extension-boxplot = 📦 এক্সটেনশন অনুযায়ী ফাইলের আকার
plot-temporal-timeline = ⏱ সংযুক্ত টেম্পোরাল টাইমলাইন
plot-deduplicator-waste = 👥 এক্সটেনশন অনুযায়ী ডুপ্লিকেট অপচয়

# --- Deduplicator Strings ---
dedup-desc = ক্রিপ্টোগ্রাফিকভাবে নিরাপদ BLAKE3 হ্যাশ ব্যবহার করে বাইট-টু-বাইট অভিন্ন ফাইল খুঁজে নিরাপদে মুছে ফেলুন।
dedup-how-it-works = ℹ এটি কীভাবে কাজ করে
dedup-min-size = সর্বনিম্ন ফাইলের আকার:
dedup-ignore-system = সিস্টেম ফাইল উপেক্ষা করুন
dedup-ignore-hidden = লুকানো ফাইল উপেক্ষা করুন
dedup-start-scan = ⚡ ডুপ্লিকেট স্ক্যান শুরু করুন
dedup-scan-first = অনুগ্রহ করে প্রথমে একটি ডিরেক্টরি স্ক্যান করুন।
dedup-cancelled-msg = স্ক্যান বাতিল করা হয়েছে। ডুপ্লিকেট খুঁজতে একটি নতুন স্ক্যান শুরু করুন।
dedup-analyzing = ফাইল বিশ্লেষণ করা হচ্ছে...
dedup-no-duplicates = কোনো ডুপ্লিকেট গ্রুপ পাওয়া যায়নি। সর্বনিম্ন ফাইলের আকার কমিয়ে দেখুন বা অন্য ফোল্ডার স্ক্যান করুন।
no-permission = অনুমতি নেই
hardlink-badge = হার্ডলিংক
dedup-select-items = 🎯 আইটেম নির্বাচন করুন...
dedup-select-all-but-oldest = 🎯 পুরোনোতম বাদে সব
dedup-select-all-but-newest = 🎯 নতুনতম বাদে সব
dedup-select-all-but-shortest = 🎯 সবচেয়ে ছোট পাথ বাদে সব
dedup-select-all-but-rootmost = 🎯 রুটের সবচেয়ে কাছেরটি বাদে সব
dedup-select-all-but-longest = 🎯 সবচেয়ে লম্বা পাথ বাদে সব
dedup-pref-dir-pattern = পছন্দের ডিরেক্টরি প্যাটার্ন:
dedup-select-all-but-pref = 🎯 পছন্দের ডিরেক্টরি বাদে সব
dedup-clear-selection = ❌ নির্বাচন মুছুন
dedup-link-menu = 🔗 লিংক... ({ $count }টি ফাইল)
dedup-link-menu-disabled = 🔗 লিংক... (0টি ফাইল)
dedup-link-hardlinks = 🔗 নির্বাচিতগুলো হার্ডলিংক দিয়ে প্রতিস্থাপন করুন
dedup-link-softlinks = 🔗 নির্বাচিতগুলো সফটলিংক দিয়ে প্রতিস্থাপন করুন
dedup-remove-menu = 🗑 সরান... ({ $count }টি ফাইল, { $size })
dedup-remove-menu-disabled = 🗑 সরান... (0টি ফাইল)
dedup-remove-trash = ♻ নির্বাচিতগুলো ট্র্যাশে সরান
dedup-remove-delete = 🗑 নির্বাচিতগুলো স্থায়ীভাবে মুছে ফেলুন
dedup-warning-title = ⚠ ডেটা হারানোর সতর্কতা
dedup-warning-desc = { $count ->
    [one] 1টি ফাইলের সব সংস্করণ মুছে ফেলা হচ্ছে
   *[other] { $count }টি ফাইলের সব সংস্করণ মুছে ফেলা হচ্ছে
}
dedup-warning-no-original = কোনো মূল কপি থাকবে না:
dedup-warning-details = নিচে তালিকাভুক্ত ফাইলগুলোর মূল এবং সব ডুপ্লিকেট কপি আপনি চেক করেছেন। এগুলো মুছে ফেললে সম্ভবত স্থায়ী ডেটা ক্ষতি হবে:
dedup-cancel-hover = স্ক্যান বাতিল করতে ক্লিক করুন
scan-cancel-hover = স্ক্যান বাতিল করতে ক্লিক করুন
dedup-current-label = বর্তমান
dedup-phase1-size = ধাপ 1/7: সব স্ক্যান করা ফাইল আকার অনুযায়ী গ্রুপ করা হচ্ছে...
dedup-phase1-filter = ধাপ 1/7: ডুপ্লিকেট প্রার্থীদের ওপর বাদদের ফিল্টার প্রয়োগ করা হচ্ছে...
dedup-phase2-prefix = ধাপ 2/7: ফাইলের প্রিফিক্স হ্যাশ করা হচ্ছে (প্রথম 4KB)...
dedup-phase3-midpoint = ধাপ 3/7: ফাইলের মধ্যবিন্দু হ্যাশ করা হচ্ছে...
dedup-phase4-suffix = ধাপ 4/7: ফাইলের সাফিক্স হ্যাশ করা হচ্ছে...
dedup-phase5-multirange = ধাপ 5/7: বড় ফাইলগুলোতে মাল্টি-রেঞ্জ হ্যাশিং চলছে...
dedup-phase6-full = ধাপ 6/7: বাকি প্রার্থীদের সম্পূর্ণ BLAKE3 হ্যাশিং চলছে...
dedup-phase7-validation = ধাপ 7/7: চূড়ান্ত টাইমস্ট্যাম্প যাচাই করা হচ্ছে...
dedup-phase-finished = { $duration } সময়ে সম্পন্ন! { $count }টি ডুপ্লিকেট গ্রুপ পাওয়া গেছে। সম্ভাব্য পুনরুদ্ধারযোগ্য স্থান: { $space }
dedup-scan-cancelled-with-error = স্ক্যান বাতিল করা হয়েছে: { $error }

# Deduplicator Table Headers
dedup-hdr-checkbox = [     ]
dedup-hdr-filename = ফাইলের নাম
dedup-hdr-directory = প্যারেন্ট ডিরেক্টরি
dedup-hdr-size = আকার
dedup-hdr-reclaimable = পুনরুদ্ধারযোগ্য
dedup-hdr-created = তৈরির সময়
dedup-hdr-modified = পরিবর্তনের সময়
dedup-copies-selected = ({ $count ->
    [one] 1টি কপি নির্বাচিত
   *[other] { $count }টি কপি নির্বাচিত
})

# --- Explorer Details Panel ---
explorer-details-header = ℹ বিবরণ
explorer-deselect-hover = আইটেমগুলোর নির্বাচন বাতিল করুন
explorer-deselect-single-hover = আইটেমের নির্বাচন বাতিল করুন
explorer-selected-items-count = { $count ->
    [one] 1টি আইটেম নির্বাচিত
   *[other] { $count }টি আইটেম নির্বাচিত
}
explorer-total-size = মোট আকার: { $size }
explorer-files = ফাইল: { $count }
explorer-directories = ডিরেক্টরি: { $count }
explorer-actions-title = অ্যাকশন
explorer-actions-operations = অপারেশন:
explorer-action-refresh-hover = নির্বাচিত সব ডিরেক্টরি সাবট্রি রিফ্রেশ করুন
explorer-grid-type = ধরন:
explorer-grid-size = আকার:
explorer-grid-bytes = বাইট:
explorer-grid-items = আইটেম:
explorer-grid-files = ফাইল:
explorer-grid-subdirs = সাবডিরেক্টরি:
explorer-grid-user = ব্যবহারকারী:
explorer-grid-group = গ্রুপ:
explorer-grid-permissions = অনুমতি:
explorer-grid-path = সম্পূর্ণ পাথ:

# Explorer Type Names
type-symlink = সিম্বলিক লিংক
type-directory = ডিরেক্টরি
type-file = ফাইল

# Explorer Actions
explorer-action-copy-path = 📋 পাথ কপি করুন
explorer-action-open-file = 📄 ফাইল খুলুন
explorer-action-open-manager = 🗁 ম্যানেজারে খুলুন
explorer-action-refresh-subtree = 🔄 সাবট্রি রিফ্রেশ করুন
explorer-action-move-trash = ♻ ট্র্যাশে সরান
explorer-action-delete-permanently = 🗑 স্থায়ীভাবে মুছে ফেলুন
explorer-action-refresh-directory = 🔄 ডিরেক্টরি রিফ্রেশ করুন

# Explorer Empty State
explorer-empty-state = ডিস্কের ব্যবহার দেখতে 'নতুন স্ক্যান'-এ ক্লিক করুন।
choose-an-option = একটি অপশন বেছে নিন
web-viewer = ওয়েব ভিউয়ার
load-demo = 👁 নমুনা ডেমো স্ন্যাপশট লোড করুন
placeholder-treemap = স্ক্যান করা ফাইলসিস্টেম এখানে ট্রিম্যাপ হিসেবে দেখানো হবে।
placeholder-plots = স্ক্যান করা ফাইলসিস্টেম এখানে প্লট করা হবে।

# Treemap Zoom & Navigation
zoom-up = ⏶ উপরে
zoom-reset = ❌ রিসেট
zoom-to-dir = 🔍 ট্রিম্যাপে ফোকাস করুন
zoom-up-level = ⏶ এক স্তর উপরে যান
zoom-empty-dir = ডিরেক্টরি খালি

# --- Extensions Panel ---
extensions-header = 📂 এক্সটেনশন
extensions-empty = এখনও কোনো পরিসংখ্যান সংগৃহীত হয়নি।
extensions-hover-files = ফাইল: { $count }

# --- Operations (Context Actions) ---
op-up-one-level = এক স্তর উপরে
op-zoom-treemap = ট্রিম্যাপে ফোকাস করুন
op-refresh-entire-scan = পুরো স্ক্যান রিফ্রেশ করুন
op-refresh-directory = ডিরেক্টরি রিফ্রেশ করুন
op-open-file = ফাইল খুলুন
op-open-file-manager = ফাইল ম্যানেজারে খুলুন
op-open-terminal = এখানে টার্মিনাল খুলুন
op-copy-path = পাথ কপি করুন
op-copy-name = নাম কপি করুন
op-move-trash = ট্র্যাশে সরান
op-permanently-delete = স্থায়ীভাবে মুছে ফেলুন

# Toast Notifications
toast-already-root = ইতিমধ্যে রুট স্তরে আছেন
toast-navigated-up = এক স্তর উপরে যাওয়া হয়েছে
toast-zoomed-treemap = ট্রিম্যাপ ডিরেক্টরিতে ফোকাস করা হয়েছে
toast-refreshing-scan = পুরো স্ক্যান রিফ্রেশ করা হচ্ছে...
toast-refreshing-dir = নির্বাচিত ডিরেক্টরি রিফ্রেশ করা হচ্ছে...
toast-opened-file = খোলা হয়েছে: { $path }
toast-failed-open-file = ফাইল খুলতে ব্যর্থ: { $error }
toast-opened-manager = ফাইল ম্যানেজারে খোলা হয়েছে: { $path }
toast-failed-open-manager = ফাইল ম্যানেজারে খুলতে ব্যর্থ: { $error }
toast-opened-terminal = টার্মিনাল খোলা হয়েছে: { $path }
toast-failed-open-terminal = টার্মিনাল খুলতে ব্যর্থ: { $error }
toast-copied-paths = ক্লিপবোর্ডে { $count ->
    [one] 1টি পাথ
   *[other] { $count }টি পাথ
} কপি করা হয়েছে
toast-copied-names = ক্লিপবোর্ডে { $count ->
    [one] 1টি নাম
   *[other] { $count }টি নাম
} কপি করা হয়েছে

# --- Modals ---
modal-remember-confirmation = ভবিষ্যতের সব ফাইল ও ডিরেক্টরির জন্য নিশ্চিতকরণ মনে রাখুন
modal-process-multiple = আপনি { $count }টি ডুপ্লিকেট ফাইল/আইটেম প্রসেস করতে চলেছেন:
modal-process-single = আপনি নিচের পাথটি প্রসেস করতে চলেছেন:
# Confirm Deletion/Trash/Link Modals
modal-delete-title = ⚠ স্থায়ী মুছে ফেলার সতর্কতা
modal-delete-header = ⚠ স্থায়ী মুছে ফেলার সতর্কতা!
modal-delete-info = মোট আকার: { $size }
modal-delete-warning = এটি একটি রিকার্সিভ মুছে ফেলা। নির্বাচিত পাথের অধীনে সব ফাইল, ফোল্ডার ও সাবডিরেক্টরি স্থায়ীভাবে মুছে যাবে এবং পুনরুদ্ধার করা যাবে না (রিসাইকেল বিন/ট্র্যাশ এড়িয়ে যাবে)।
modal-delete-checkbox = আমি বুঝতে পেরেছি যে ফাইলগুলো স্থায়ীভাবে মুছে যাবে এবং পুনরুদ্ধার করা যাবে না।
modal-delete-confirm = 🗑 হ্যাঁ, স্থায়ীভাবে মুছে ফেলুন

modal-trash-title = ♻ ট্র্যাশে সরান
modal-trash-header = ♻ ট্র্যাশে সরান
modal-trash-info = মোট আকার: { $size }
modal-trash-warning = এটি নির্বাচিত পাথ এবং তাদের সব কন্টেন্ট আপনার সিস্টেমের রিসাইকেল বিন/ট্র্যাশে সরিয়ে দেবে, যেখান থেকে পরে সেগুলো পুনরুদ্ধার বা স্থায়ীভাবে মুছে ফেলা যাবে।
modal-trash-checkbox = আমি নিশ্চিত করছি যে আমি এটি ট্র্যাশে সরাতে চাই।
modal-trash-confirm = ♻ হ্যাঁ, ট্র্যাশে সরান

modal-delete-duplicates-title = ⚠ স্থায়ী ডুপ্লিকেট অপসারণের সতর্কতা
modal-delete-duplicates-header = ⚠ স্থায়ী ডুপ্লিকেট মুছে ফেলার সতর্কতা!
modal-delete-duplicates-info = পুনরুদ্ধারযোগ্য মোট স্থান: { $size }
modal-delete-duplicates-warning = নির্বাচিত সব ফাইল স্থায়ীভাবে মুছে যাবে এবং পুনরুদ্ধার করা যাবে না (রিসাইকেল বিন/ট্র্যাশ এড়িয়ে যাবে)।
modal-delete-duplicates-checkbox = আমি বুঝতে পেরেছি যে ফাইলগুলো স্থায়ীভাবে মুছে যাবে এবং পুনরুদ্ধার করা যাবে না।
modal-delete-duplicates-confirm = 🗑 হ্যাঁ, নির্বাচিতগুলো স্থায়ীভাবে মুছে ফেলুন

modal-trash-duplicates-title = ♻ ডুপ্লিকেটগুলো ট্র্যাশে সরান
modal-trash-duplicates-header = ♻ ডুপ্লিকেটগুলো ট্র্যাশে সরান
modal-trash-duplicates-info = পুনরুদ্ধারযোগ্য মোট স্থান: { $size }
modal-trash-duplicates-warning = নির্বাচিত সব ফাইল রিসাইকেল বিন/ট্র্যাশে সরিয়ে দেওয়া হবে।
modal-trash-duplicates-checkbox = আমি নিশ্চিত করছি যে আমি এই ফাইলগুলো ট্র্যাশে সরাতে চাই।
modal-trash-duplicates-confirm = ♻ হ্যাঁ, নির্বাচিতগুলো ট্র্যাশে সরান

modal-hardlink-duplicates-title = 🔗 ডুপ্লিকেটগুলো হার্ডলিংক দিয়ে প্রতিস্থাপন করুন
modal-hardlink-duplicates-header = 🔗 ডুপ্লিকেটগুলো হার্ডলিংক দিয়ে প্রতিস্থাপন করুন
modal-hardlink-duplicates-info = প্রসেস করার মোট ফাইল: { $count }। ক্রমযোজিত ভার্চুয়াল আকার: { $size }
modal-hardlink-duplicates-warning = এটি নির্বাচিত ডুপ্লিকেট ফাইলগুলো মুছে ফেলবে এবং প্রতিটি গ্রুপে অবশিষ্ট মূল ফাইলের দিকে নির্দেশকারী ফাইলসিস্টেম-স্তরের হার্ডলিংক দিয়ে প্রতিস্থাপন করবে। এটি ফাইলগুলোকে দৃশ্যত ধরে রাখে, অথচ প্রকৃত ফিজিক্যাল স্টোরেজ খালি করে।
modal-hardlink-duplicates-checkbox = আমি নিশ্চিত করছি যে আমি নির্বাচিত ফাইলগুলো হার্ডলিংক দিয়ে প্রতিস্থাপন করতে চাই।
modal-hardlink-duplicates-confirm = 🔗 হ্যাঁ, হার্ডলিংক দিয়ে প্রতিস্থাপন করুন

modal-softlink-duplicates-title = 🔗 ডুপ্লিকেটগুলো সফটলিংক দিয়ে প্রতিস্থাপন করুন
modal-softlink-duplicates-header = 🔗 ডুপ্লিকেটগুলো সফটলিংক দিয়ে প্রতিস্থাপন করুন
modal-softlink-duplicates-info = প্রসেস করার মোট ফাইল: { $count }। ক্রমযোজিত ভার্চুয়াল আকার: { $size }
modal-softlink-duplicates-warning = এটি নির্বাচিত ডুপ্লিকেট ফাইলগুলো মুছে ফেলবে এবং প্রতিটি গ্রুপে অবশিষ্ট মূল ফাইলের দিকে নির্দেশকারী ফাইলসিস্টেম-স্তরের সফটলিংক (সিম্বলিক লিংক) দিয়ে প্রতিস্থাপন করবে। এটি ফাইলগুলোকে দৃশ্যত ধরে রাখে, অথচ প্রকৃত ফিজিক্যাল স্টোরেজ খালি করে।
modal-softlink-duplicates-checkbox = আমি নিশ্চিত করছি যে আমি নির্বাচিত ফাইলগুলো সফটলিংক দিয়ে প্রতিস্থাপন করতে চাই।
modal-softlink-duplicates-confirm = 🔗 হ্যাঁ, সফটলিংক দিয়ে প্রতিস্থাপন করুন

# Path Does Not Exist Modal
modal-path-not-exist-title = ❌ পাথটি বিদ্যমান নেই!
modal-path-not-exist-msg = ত্রুটি: আপনি যে পাথটি মুছতে চাইছেন তা ডিস্কে নেই।
modal-close-btn = বন্ধ করুন
modal-details-label = বিবরণ: 
modal-cancel-btn = বাতিল

# Elevation Recommended Modal
modal-elevation-title = ⚠ এলিভেশন প্রস্তাবিত
modal-elevation-desc = eDirStat ডিফল্টভাবে সাধারণ ব্যবহারকারীর অনুমতিতে চলে। তবে Windows প্রশাসক অ্যাকাউন্ট ছাড়া সরাসরি ফিজিক্যাল ডিস্ক হ্যান্ডল অ্যাক্সেস কঠোরভাবে সীমিত করে।
modal-elevation-mft-disabled = Windows NTFS MFT ড্রাইভার নিষ্ক্রিয়
modal-elevation-mft-desc = প্রশাসনিক অনুমতি ছাড়া ডাইরেক্ট-টু-ডিস্ক MFT স্ক্যানার চালু করা যায় না। ফাইল বিশ্লেষণ ফলব্যাক স্ট্যান্ডার্ড ট্রাভার্সাল ড্রাইভার ব্যবহার করবে, ফলে স্ক্যানের পারফরম্যান্স 20x পর্যন্ত কমে যাবে।
modal-elevation-relaunch-prompt = আপনি কি এখন প্রশাসক অনুমতিসহ অ্যাপ্লিকেশনটি পুনরায় চালু করতে চান?
modal-elevation-continue-std = সাধারণ ব্যবহারকারী হিসেবে চালিয়ে যান
modal-elevation-relaunch-btn = 🛡 প্রশাসক হিসেবে পুনরায় চালু করুন

# About Modal
modal-about-title = ℹ eDirStat সম্পর্কে
modal-about-author = প্রণেতা: Cody Wyatt Neiman (xangelix) <neiman@cody.to>
modal-about-license-btn = 📜 লাইসেন্স (MIT)
modal-about-desc1 = Rust-এ নির্মিত একটি হাই-পারফরম্যান্স ডিস্ক স্পেস বিশ্লেষক ও ডুপ্লিকেট অপসারণ টুলকিট।
modal-about-desc2 = সমান্তরাল, ওয়ার্ক-স্টিলিং ডিরেক্টরি ট্রাভার্সাল, জিরো-পার্সিং লেআউট ডিসিরিয়ালাইজেশনসহ কম্প্রেসড স্ন্যাপশট এবং রেসপন্সিভ, ইন্টারঅ্যাকটিভ ট্রিম্যাপ ফিচার রয়েছে।
modal-about-desc3 = অন্তর্নির্মিত ডুপ্লিকেটর একটি মাল্টি-স্টেজ ক্রিপ্টোগ্রাফিক হ্যাশিং পাইপলাইন চালায়, যা ডুপ্লিকেট গ্রুপগুলো নিরাপদে শনাক্ত করে, পুনরুদ্ধারযোগ্য স্থান হিসাব করে এবং সিস্টেম-স্তরের হার্ডলিংকগুলোকে সম্মান করে।
modal-about-licenses-btn = ওপেন সোর্স লাইসেন্স দেখুন
modal-about-version = v{ $version }

# How Deduplication Works Modal
modal-how-dedup-title = ℹ ডুপ্লিকেট অপসারণ কীভাবে কাজ করে
modal-how-dedup-desc1 = প্রতিটি ফাইলের বাইট সরাসরি তুলনা না করে (যার জন্য ধীর, জোড়াভিত্তিক O(N²) স্ক্যান প্রয়োজন), এই সিস্টেমটি অভিন্ন কন্টেন্ট নিরাপদে ও দক্ষতার সাথে শনাক্ত করতে অত্যন্ত অপ্টিমাইজড 7-স্তরের পাইপলাইন ব্যবহার করে।
modal-how-dedup-pipeline-title = 7-স্তরের পাইপলাইন:
modal-how-dedup-why-title = কেন এটি যথেষ্ট?
modal-how-dedup-why-desc1 = এই মাল্টি-স্টেজ ফিল্টার নিশ্চিত করে যে শুধুমাত্র অভিন্ন আকার, প্রিফিক্স, মধ্যবিন্দু, সাফিক্স এবং বণ্টিত ব্লক নমুনা বিশিষ্ট ফাইলগুলোই সম্পূর্ণ পড়া হয়। অবশেষে, একটি 256-bit BLAKE3 ক্রিপ্টোগ্রাফিক হ্যাশ তুলনা করা শিল্প-মানের নিরাপদ ট্রান্সফার প্রোটোকলের সমতুল্য নিরাপত্তা দেয়, ফলে ধীর, জোড়াভিত্তিক বাইট-টু-বাইট তুলনার প্রয়োজন দূর হয়।

# How Deduplication Works Steps
modal-how-dedup-step1-title = 1. আকার অনুযায়ী ভাগ করা
modal-how-dedup-step1-desc = ফাইলগুলো বাইটে তাদের সঠিক আকার অনুযায়ী গ্রুপ করা হয়। অনন্য আকারের যেকোনো ফাইল সাথে সাথে বাদ দেওয়া হয়, ফলে ডিস্ক I/O সম্পূর্ণ এড়িয়ে যায়।
modal-how-dedup-step2-title = 2. প্রিফিক্স হ্যাশিং
modal-how-dedup-step2-desc = অবশিষ্ট প্রার্থীদের প্রথম 4KB হ্যাশ করা হয়। এটি ভিন্ন হেডার বা মেটাডেটা ফর্ম্যাটের ফাইলগুলো দ্রুত বাদ দেয়।
modal-how-dedup-step3-title = 3. মধ্যবিন্দু হ্যাশিং
modal-how-dedup-step3-desc = অবশিষ্ট ফাইলগুলোর মাঝখান থেকে একটি 4KB ব্লক হ্যাশ করা হয়, যা অভ্যন্তরীণ কাঠামোগত পার্থক্য ধরে ফেলে।
modal-how-dedup-step4-title = 4. সাফিক্স হ্যাশিং
modal-how-dedup-step4-desc = ডেটার শেষ 4KB হ্যাশ করা হয়। এটি শেষাংশের কন্টেন্ট বা মেটাডেটার পার্থক্য শনাক্ত করতে অত্যন্ত কার্যকর।
modal-how-dedup-step5-title = 5. মাল্টি-রেঞ্জ হ্যাশিং
modal-how-dedup-step5-desc = বড় ফাইলগুলোতে (100MB-এর বেশি) পুরো দৈর্ঘ্য জুড়ে পর্যায়ক্রমিক ব্লক নমুনা নেওয়া হয়, যাতে পুরো ফাইল না পড়েই কন্টেন্টের সামঞ্জস্য যাচাই করা যায়।
modal-how-dedup-step6-title = 6. সম্পূর্ণ BLAKE3 হ্যাশিং
modal-how-dedup-step6-desc = অবশিষ্ট প্রার্থীদের জন্য একটি সম্পূর্ণ BLAKE3 ক্রিপ্টোগ্রাফিক হ্যাশ গণনা করা হয়। 256-bit স্পেসের উচ্চ কলিশন প্রতিরোধের কারণে মিলে যাওয়া হ্যাশগুলো নির্দেশ করে যে ফাইলগুলো ভিন্ন হওয়ার সম্ভাবনা জ্যোতির্বিজ্ঞানসংখ্যিকভাবে নগণ্য — জোড়াভিত্তিক তুলনা ছাড়াই এটি পরিচয়ের অত্যন্ত নির্ভরযোগ্য প্রমাণ দেয়।
modal-how-dedup-step7-title = 7. টাইমস্ট্যাম্প যাচাইকরণ
modal-how-dedup-step7-desc = যেকোনো ডুপ্লিকেট অ্যাকশন দেখানো বা চালানোর ঠিক আগে, অ্যাপ্লিকেশনটি ডিস্কে ফাইলগুলোর টাইমস্ট্যাম্প যাচাই করে, যাতে স্ন্যাপশট তৈরির পরে ঘটে যাওয়া পরিবর্তন থেকে সুরক্ষা পাওয়া যায়।

# Open Source Licenses Modal
modal-licenses-title = 📜 ওপেন সোর্স লাইসেন্স
modal-licenses-tab-app = eDirStat (MIT)
modal-licenses-tab-deps = তৃতীয়-পক্ষের লাইব্রেরি
modal-licenses-app-desc = eDirStat হলো এমআইটি (MIT) লাইসেন্সের অধীনে বিতরণকৃত ওপেন সোর্স সফটওয়্যার:
modal-licenses-desc = এই অ্যাপ্লিকেশনে নিচের তৃতীয়-পক্ষের লাইব্রেরি ও ক্রেটগুলো ব্যবহার করা হয়েছে:
modal-licenses-copy-btn = 📋 লাইসেন্স কপি করুন
modal-licenses-copy-all-btn = 📋 লাইসেন্সগুলো কপি করুন

# Processing Modal
modal-processing-title = ⏳ প্রসেস করা হচ্ছে...
modal-processing-deletion = ফাইল ও ডিরেক্টরি মুছে ফেলা হচ্ছে...
modal-processing-trash = ফাইল ও ডিরেক্টরি ট্র্যাশে সরানো হচ্ছে...
modal-processing-hardlink = ডুপ্লিকেটগুলো হার্ডলিংক দিয়ে প্রতিস্থাপন করা হচ্ছে...
modal-processing-softlink = ডুপ্লিকেটগুলো সফটলিংক দিয়ে প্রতিস্থাপন করা হচ্ছে...

# Explorer Column Headers
explorer-hdr-name = নাম
explorer-hdr-percentage = শতাংশ
explorer-hdr-size = আকার
explorer-hdr-items = আইটেম
explorer-hdr-files = ফাইল
explorer-hdr-subdirs = সাবডিরেক্টরি
explorer-hdr-created = তৈরির সময়
explorer-hdr-modified = পরিবর্তনের সময়

# Update Checker
update-checking = আপডেট চেক করা হচ্ছে...
update-available = নতুন সংস্করণ { $version } উপলব্ধ!
update-up-to-date = আপনি আপ টু ডেট আছেন
update-failed = আপডেট চেক ব্যর্থ হয়েছে: { $error }

# Themes
theme = 🎨 থিম
theme-dark = ডার্ক
theme-high-contrast = হাই কনট্রাস্ট
theme-light = লাইট
theme-system = সিস্টেম

# New Scan Options Modal
modal-scan-options-title = নতুন স্ক্যানের অপশন
modal-scan-options-header = একটি নতুন স্ক্যান শুরু করুন
modal-scan-options-path-label = স্ক্যান করার ডিরেক্টরির পাথ:
modal-scan-options-paste-tooltip = ক্লিপবোর্ড থেকে পেস্ট করুন
modal-scan-options-browse-tooltip = ফোল্ডার ব্রাউজ করুন...
modal-scan-options-scan-btn = স্ক্যান
modal-scan-options-cancel-btn = বাতিল
modal-scan-options-same-filesystem = স্ক্যান একই ফাইলসিস্টেম/ভলিউমে সীমিত রাখুন
modal-scan-options-drives-header = 💽 স্টোরেজ ড্রাইভ ও ভলিউম
modal-scan-options-refresh-tooltip = স্টোরেজ ড্রাইভ রিফ্রেশ করুন
modal-scan-options-root-system = রুট সিস্টেম
modal-scan-options-selected-badge = ✅ নির্বাচিত
modal-scan-options-free-of = { $total }-এর মধ্যে { $free } ফাঁকা
modal-scan-options-subtitle = বিশ্লেষণের জন্য একটি স্টোরেজ ভলিউম, দ্রুত অবস্থান বা কাস্টম ডিরেক্টরি নির্বাচন করুন।
modal-scan-options-quick-access = 📍 দ্রুত অ্যাক্সেস শর্টকাট
modal-scan-options-path-hint = /path/to/scan
modal-scan-options-hint = ℹ উপরে একটি ড্রাইভ নির্বাচন করুন অথবা একটি ডিরেক্টরির পাথ লিখুন।
modal-scan-options-sandbox-auth = 🔒 স্যান্ডবক্স অ্যাক্সেস প্রয়োজন — অ্যাক্সেস দিতে স্ক্যানে ক্লিক করুন
modal-scan-options-valid-dir = ✅ বৈধ ডিরেক্টরি — স্ক্যানের জন্য প্রস্তুত
modal-scan-options-points-to-file = ⚠ পাথটি একটি ফাইলকে নির্দেশ করছে — অনুগ্রহ করে একটি ফোল্ডার নির্বাচন করুন।
modal-scan-options-dir-not-exist = ⚠ ফাইলসিস্টেমে ডিরেক্টরিটি নেই।
quick-loc-home = 🏠 হোম
quick-loc-documents = 📄 ডকুমেন্টস
quick-loc-downloads = 📥 ডাউনলোডস
quick-loc-desktop = 🖥 ডেস্কটপ
quick-loc-pictures = 🖼 ছবি
search-use-regex = রেগুলার এক্সপ্রেশন (Regex) ব্যবহার করুন
search-match-case = কেস মিলান (কেস-সংবেদনশীল)
dedup-pref-dir-hint = যেমন /home/user/Archive

file-menu-close = স্ক্যান বন্ধ করুন
file-menu-quit = প্রস্থান

badge-dataless-cloud = ক্লাউড / ডেটাবিহীন ফাইল
badge-symlink = সিম্বলিক লিংক
badge-special-file = বিশেষ ফাইল (পাইপ / সকেট / ডিভাইস)
badge-permission-denied = অ্যাক্সেস প্রত্যাখ্যাত
