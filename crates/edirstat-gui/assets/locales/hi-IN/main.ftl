# Menu Bar Dropdowns
file = फ़ाइल
view = दृश्य
help = मदद

# Menu Bar Actions
new-scan = 📁 नया स्कैन
save-snapshot = 💾 स्नैपशॉट सहेजें
load-snapshot = 📖 स्नैपशॉट लोड करें

# Menu Bar Status
idle = निष्क्रिय

# View Menu Options
monospace-paths = 🅰 मोनोस्पेस पथ
highlight-duplicates = ✨ डुप्लिकेट हाइलाइट करें
treemap-borders = 🔳 ट्रीमैप बॉर्डर
treemap-style =  ट्रीमैप शैली
treemap-style-vertical = ऊर्ध्वाधर ग्रेडिएंट
treemap-style-offset-vertical = ऑफ़सेट ऊर्ध्वाधर ग्रेडिएंट
treemap-style-diagonal = विकर्ण ग्रेडिएंट
treemap-style-cushion = कुशन शेडिंग
deletion-confirmation = 🗑 हटाने की पुष्टि
trash-confirmation = ♻ ट्रैश पुष्टि
time-format = 🕒 समय प्रारूप
language = 💬 भाषा
layout-mode = लेआउट मोड:
classic-layout = क्लासिक लेआउट
windirstat-layout = WinDirStat लेआउट
vis-mode-treemap = 📊 ट्रीमैप
vis-mode-plots = 📈 प्लॉट
select-plot-label = प्लॉट चुनें:
vis-mode-deduplicator = 👥 डुप्लिकेट फ़ाइल खोजक
search-filter-label = 🔍 फ़िल्टर:

# Panel Toggles
toggle-left-panel = { $collapsed ->
    [true] ▶ बायाँ पैनल दिखाएँ (F9)
   *[false] ◀ बायाँ पैनल छिपाएँ (F9)
}

toggle-right-panel = { $collapsed ->
    [true] { $is_classic ->
        [true] ◀ दायाँ पैनल दिखाएँ (F11)
       *[false] ▶ एक्सटेंशन पैनल दिखाएँ (F11)
    }
   *[false] { $is_classic ->
        [true] ▶ दायाँ पैनल छिपाएँ (F11)
       *[false] ◀ एक्सटेंशन पैनल छिपाएँ (F11)
    }
}

collapse-all = ⏏ सभी समेटें
about = ℹ परिचय
web-not-available = वेब संस्करण में यह सुविधा उपलब्ध नहीं है

# Status Indicators
scanning-disk = डिस्क स्कैन की जा रही है...
scan-complete = स्कैन पूर्ण
scan-cancelled = स्कैन रद्द
path-label = पथ: { $path }
worker-threads = ⚡ { $count } वर्कर थ्रेड
worker-threads-hover = निर्देशिका ट्रैवर्सल के लिए आवंटित समानांतर, वर्क-स्टीलिंग CPU कोर की संख्या।

# Stats Panel (Bottom)
directories-count = 📁 निर्देशिकाएँ: { $count }
files-count = 📄 फ़ाइलें: { $count }
total-size = 💾 कुल आकार: { $size }
elapsed-time = ⏱ समय: { $time }
scan-speed = ⚡ गति: { $speed }/s

# Selection Info
selection-path = चयन: { $path }
selection-items = चयन: { $count ->
    [one] 1 आइटम
   *[other] { $count } आइटम
}

# Plot Types
plot-size-distribution = 📊 फ़ाइल आकार वितरण
plot-age-size = 🌌 फ़ाइल आयु बनाम फ़ाइल आकार
plot-dir-composition = 🍰 निर्देशिका संरचना
plot-extension-boxplot = 📦 एक्सटेंशन के अनुसार फ़ाइल आकार
plot-temporal-timeline = ⏱ लिंक की गई टेम्पोरल टाइमलाइनें
plot-deduplicator-waste = 👥 एक्सटेंशन के अनुसार डुप्लिकेट अपव्यय

# --- Deduplicator Strings ---
dedup-desc = क्रिप्टोग्राफ़िक रूप से सुरक्षित BLAKE3 हैश का उपयोग करके बाइट-दर-बाइट समान फ़ाइलें खोजें और सुरक्षित रूप से हटाएँ।
dedup-how-it-works = ℹ यह कैसे काम करता है
dedup-min-size = न्यूनतम फ़ाइल आकार:
dedup-ignore-system = सिस्टम फ़ाइलें अनदेखी करें
dedup-ignore-hidden = छिपी हुई फ़ाइलें अनदेखी करें
dedup-start-scan = ⚡ डुप्लिकेशन स्कैन प्रारंभ करें
dedup-scan-first = कृपया पहले कोई निर्देशिका स्कैन करें।
dedup-cancelled-msg = स्कैन रद्द कर दिया गया। डुप्लिकेट खोजने के लिए नया स्कैन शुरू करें।
dedup-analyzing = फ़ाइलों का विश्लेषण किया जा रहा है...
dedup-no-duplicates = कोई डुप्लिकेट समूह नहीं मिला। न्यूनतम फ़ाइल आकार घटाकर या किसी दूसरे फ़ोल्डर को स्कैन करके देखें।
no-permission = अनुमति नहीं
hardlink-badge = हार्डलिंक
dedup-select-items = 🎯 आइटम चुनें...
dedup-select-all-but-oldest = 🎯 सबसे पुरानी के अलावा सभी
dedup-select-all-but-newest = 🎯 सबसे नई के अलावा सभी
dedup-select-all-but-shortest = 🎯 सबसे छोटे पथ के अलावा सभी
dedup-select-all-but-rootmost = 🎯 रूट के सबसे निकट के अलावा सभी
dedup-select-all-but-longest = 🎯 सबसे लंबे पथ के अलावा सभी
dedup-pref-dir-pattern = पसंदीदा निर्देशिका पैटर्न:
dedup-select-all-but-pref = 🎯 पसंदीदा निर्देशिका के अलावा सभी
dedup-clear-selection = ❌ चयन साफ़ करें
dedup-link-menu = 🔗 लिंक... ({ $count } फ़ाइलें)
dedup-link-menu-disabled = 🔗 लिंक... (0 फ़ाइलें)
dedup-link-hardlinks = 🔗 चयनित को हार्डलिंक से बदलें
dedup-link-softlinks = 🔗 चयनित को सॉफ़्टलिंक से बदलें
dedup-remove-menu = 🗑 हटाएँ... ({ $count } फ़ाइलें, { $size })
dedup-remove-menu-disabled = 🗑 हटाएँ... (0 फ़ाइलें)
dedup-remove-trash = ♻ चयनित को ट्रैश में ले जाएँ
dedup-remove-delete = 🗑 चयनित को स्थायी रूप से हटाएँ
dedup-warning-title = ⚠ डेटा हानि चेतावनी
dedup-warning-desc = { $count ->
    [one] 1 फ़ाइल के सभी संस्करण हटाए जा रहे हैं
   *[other] { $count } फ़ाइलों के सभी संस्करण हटाए जा रहे हैं
}
dedup-warning-no-original = कोई मूल प्रति शेष नहीं रहेगी:
dedup-warning-details = आपने नीचे सूचीबद्ध फ़ाइलों के लिए मूल और सभी डुप्लिकेट प्रतियाँ दोनों चेक की हैं। इन्हें हटाने से स्थायी डेटा हानि की संभावना है:
dedup-cancel-hover = स्कैन रद्द करने के लिए क्लिक करें
scan-cancel-hover = स्कैन रद्द करने के लिए क्लिक करें
dedup-current-label = वर्तमान
dedup-phase1-size = चरण 1/7: सभी स्कैन की गई फ़ाइलों को आकार के अनुसार समूहित किया जा रहा है...
dedup-phase1-filter = चरण 1/7: डुप्लिकेट उम्मीदवारों पर अपवर्जन फ़िल्टर किए जा रहे हैं...
dedup-phase2-prefix = चरण 2/7: फ़ाइल प्रीफ़िक्स (पहले 4KB) हैश किए जा रहे हैं...
dedup-phase3-midpoint = चरण 3/7: फ़ाइल मिडपॉइंट हैश किए जा रहे हैं...
dedup-phase4-suffix = चरण 4/7: फ़ाइल सफ़िक्स हैश किए जा रहे हैं...
dedup-phase5-multirange = चरण 5/7: बड़ी फ़ाइलों की मल्टी-रेंज हैशिंग की जा रही है...
dedup-phase6-full = चरण 6/7: शेष उम्मीदवारों की पूर्ण BLAKE3 हैशिंग की जा रही है...
dedup-phase7-validation = चरण 7/7: अंतिम टाइमस्टैम्प सत्यापन...
dedup-phase-finished = { $duration } में पूर्ण! { $count } डुप्लिकेट समूह मिले। संभावित पुनःप्राप्त करने योग्य स्थान: { $space }
dedup-scan-cancelled-with-error = स्कैन रद्द कर दिया गया: { $error }

# Deduplicator Table Headers
dedup-hdr-checkbox = [     ]
dedup-hdr-filename = फ़ाइल नाम
dedup-hdr-directory = पैरेंट निर्देशिका
dedup-hdr-size = आकार
dedup-hdr-reclaimable = पुनःप्राप्त करने योग्य
dedup-hdr-created = निर्मित
dedup-hdr-modified = संशोधित
dedup-copies-selected = ({ $count ->
    [one] 1 प्रति चयनित
   *[other] { $count } प्रतियाँ चयनित
})

# --- Explorer Details Panel ---
explorer-details-header = ℹ विवरण
explorer-deselect-hover = आइटम अचयनित करें
explorer-deselect-single-hover = आइटम अचयनित करें
explorer-selected-items-count = { $count ->
    [one] 1 चयनित आइटम
   *[other] { $count } चयनित आइटम
}
explorer-total-size = कुल आकार: { $size }
explorer-files = फ़ाइलें: { $count }
explorer-directories = निर्देशिकाएँ: { $count }
explorer-actions-title = क्रियाएँ
explorer-actions-operations = संक्रियाएँ:
explorer-action-refresh-hover = सभी चयनित निर्देशिका सबट्री रीफ़्रेश करें
explorer-grid-type = प्रकार:
explorer-grid-size = आकार:
explorer-grid-bytes = बाइट:
explorer-grid-items = आइटम:
explorer-grid-files = फ़ाइलें:
explorer-grid-subdirs = उपनिर्देशिकाएँ:
explorer-grid-user = उपयोगकर्ता:
explorer-grid-group = समूह:
explorer-grid-permissions = अनुमतियाँ:
explorer-grid-path = पूर्ण पथ:

# Explorer Type Names
type-symlink = सिंबॉलिक लिंक
type-directory = निर्देशिका
type-file = फ़ाइल

# Explorer Actions
explorer-action-copy-path = 📋 पथ कॉपी करें
explorer-action-open-file = 📄 फ़ाइल खोलें
explorer-action-open-manager = 🗁 मैनेजर खोलें
explorer-action-refresh-subtree = 🔄 सबट्री रीफ़्रेश करें
explorer-action-move-trash = ♻ ट्रैश में ले जाएँ
explorer-action-delete-permanently = 🗑 स्थायी रूप से हटाएँ
explorer-action-refresh-directory = 🔄 निर्देशिका रीफ़्रेश करें

# Explorer Empty State
explorer-empty-state = डिस्क उपयोग देखने के लिए 'नया स्कैन' पर क्लिक करें।
choose-an-option = कोई विकल्प चुनें
web-viewer = वेब व्यूअर
load-demo = 👁 नमूना डेमो स्नैपशॉट लोड करें
placeholder-treemap = स्कैन किया गया फ़ाइलसिस्टम यहाँ ट्रीमैप के रूप में प्रदर्शित होगा।
placeholder-plots = स्कैन किया गया फ़ाइलसिस्टम यहाँ प्लॉट किया जाएगा।

# Treemap Zoom & Navigation
zoom-up = ⏶ ऊपर
zoom-reset = ❌ रीसेट
zoom-to-dir = 🔍 ट्रीमैप में फ़ोकस करें
zoom-up-level = ⏶ एक स्तर ऊपर जाएँ
zoom-empty-dir = निर्देशिका खाली है

# --- Extensions Panel ---
extensions-header = 📂 एक्सटेंशन
extensions-empty = अभी तक कोई आँकड़े एकत्र नहीं हुए।
extensions-hover-files = फ़ाइलें: { $count }

# --- Operations (Context Actions) ---
op-up-one-level = एक स्तर ऊपर
op-zoom-treemap = ट्रीमैप में फ़ोकस करें
op-refresh-entire-scan = पूरा स्कैन रीफ़्रेश करें
op-refresh-directory = निर्देशिका रीफ़्रेश करें
op-open-file = फ़ाइल खोलें
op-open-file-manager = फ़ाइल मैनेजर में खोलें
op-open-terminal = यहाँ टर्मिनल खोलें
op-copy-path = पथ कॉपी करें
op-copy-name = नाम कॉपी करें
op-move-trash = ट्रैश में ले जाएँ
op-permanently-delete = स्थायी रूप से हटाएँ

# Toast Notifications
toast-already-root = पहले से ही रूट स्तर पर हैं
toast-navigated-up = एक स्तर ऊपर गए
toast-zoomed-treemap = ट्रीमैप को निर्देशिका पर फ़ोकस किया गया
toast-refreshing-scan = पूरा स्कैन रीफ़्रेश किया जा रहा है...
toast-refreshing-dir = चयनित निर्देशिका/निर्देशिकाएँ रीफ़्रेश की जा रही हैं...
toast-opened-file = खोला गया: { $path }
toast-failed-open-file = फ़ाइल खोलने में विफल: { $error }
toast-opened-manager = फ़ाइल मैनेजर में खोला गया: { $path }
toast-failed-open-manager = फ़ाइल मैनेजर में खोलने में विफल: { $error }
toast-opened-terminal = इस स्थान पर टर्मिनल खोला गया: { $path }
toast-failed-open-terminal = टर्मिनल खोलने में विफल: { $error }
toast-copied-paths = क्लिपबोर्ड पर { $count ->
    [one] 1 पथ कॉपी किया गया
   *[other] { $count } पथ कॉपी किए गए
}
toast-copied-names = क्लिपबोर्ड पर { $count ->
    [one] 1 नाम कॉपी किया गया
   *[other] { $count } नाम कॉपी किए गए
}

# --- Modals ---
modal-remember-confirmation = भविष्य की सभी फ़ाइलों और निर्देशिकाओं के लिए पुष्टि याद रखें
modal-process-multiple = आप { $count } डुप्लिकेट फ़ाइलों/आइटमों को प्रोसेस करने वाले हैं:
modal-process-single = आप निम्न पथ को प्रोसेस करने वाले हैं:
# Confirm Deletion/Trash/Link Modals
modal-delete-title = ⚠ स्थायी हटाने की चेतावनी
modal-delete-header = ⚠ स्थायी हटाने की चेतावनी!
modal-delete-info = कुल आकार: { $size }
modal-delete-warning = यह एक रिकर्सिव हटाने की प्रक्रिया है। चयनित पथ(ों) के अंतर्गत सभी फ़ाइलें, फ़ोल्डर और उपनिर्देशिकाएँ स्थायी रूप से हटा दी जाएँगी और इन्हें पुनःप्राप्त नहीं किया जा सकेगा (रीसायकल बिन/ट्रैश को बायपास करते हुए)।
modal-delete-checkbox = मैं समझता हूँ कि फ़ाइलें स्थायी रूप से हटा दी जाएँगी और इन्हें पुनःप्राप्त नहीं किया जा सकेगा।
modal-delete-confirm = 🗑 हाँ, स्थायी रूप से हटाएँ

modal-trash-title = ♻ ट्रैश में ले जाएँ
modal-trash-header = ♻ ट्रैश में ले जाएँ
modal-trash-info = कुल आकार: { $size }
modal-trash-warning = यह चयनित पथ(ों) और उनकी सभी सामग्री को आपके सिस्टम के रीसायकल बिन/ट्रैश में ले जाएगा, जहाँ इन्हें बाद में पुनःप्राप्त या स्थायी रूप से हटाया जा सकता है।
modal-trash-checkbox = मैं पुष्टि करता हूँ कि मैं इसे ट्रैश में ले जाना चाहता हूँ।
modal-trash-confirm = ♻ हाँ, ट्रैश में ले जाएँ

modal-delete-duplicates-title = ⚠ स्थायी डुप्लिकेशन चेतावनी
modal-delete-duplicates-header = ⚠ स्थायी डुप्लिकेट हटाने की चेतावनी!
modal-delete-duplicates-info = पुनःप्राप्त किया जाने वाला कुल स्थान: { $size }
modal-delete-duplicates-warning = सभी चयनित फ़ाइलें स्थायी रूप से हटा दी जाएँगी और इन्हें पुनःप्राप्त नहीं किया जा सकेगा (रीसायकल बिन/ट्रैश को बायपास करते हुए)।
modal-delete-duplicates-checkbox = मैं समझता हूँ कि फ़ाइलें स्थायी रूप से हटा दी जाएँगी और इन्हें पुनःप्राप्त नहीं किया जा सकेगा।
modal-delete-duplicates-confirm = 🗑 हाँ, चयनित को स्थायी रूप से हटाएँ

modal-trash-duplicates-title = ♻ डुप्लिकेट को ट्रैश में ले जाएँ
modal-trash-duplicates-header = ♻ डुप्लिकेट को ट्रैश में ले जाएँ
modal-trash-duplicates-info = पुनःप्राप्त किया जाने वाला कुल स्थान: { $size }
modal-trash-duplicates-warning = सभी चयनित फ़ाइलों को रीसायकल बिन/ट्रैश में ले जाया जाएगा।
modal-trash-duplicates-checkbox = मैं पुष्टि करता हूँ कि मैं इन फ़ाइलों को ट्रैश में ले जाना चाहता हूँ।
modal-trash-duplicates-confirm = ♻ हाँ, चयनित को ट्रैश में ले जाएँ

modal-hardlink-duplicates-title = 🔗 डुप्लिकेट को हार्डलिंक से बदलें
modal-hardlink-duplicates-header = 🔗 डुप्लिकेट को हार्डलिंक से बदलें
modal-hardlink-duplicates-info = प्रोसेस की जाने वाली कुल फ़ाइलें: { $count }। संचित वर्चुअल आकार: { $size }
modal-hardlink-duplicates-warning = यह चयनित डुप्लिकेट फ़ाइलों को हटा देगा और उन्हें प्रत्येक समूह में शेष मूल फ़ाइल की ओर इशारा करते फ़ाइलसिस्टम-स्तरीय हार्डलिंक से बदल देगा। इससे फ़ाइलें दृश्यतः बनी रहती हैं जबकि वास्तविक भौतिक स्टोरेज खाली हो जाता है।
modal-hardlink-duplicates-checkbox = मैं पुष्टि करता हूँ कि मैं चयनित फ़ाइलों को हार्डलिंक से बदलना चाहता हूँ।
modal-hardlink-duplicates-confirm = 🔗 हाँ, हार्डलिंक से बदलें

modal-softlink-duplicates-title = 🔗 डुप्लिकेट को सॉफ़्टलिंक से बदलें
modal-softlink-duplicates-header = 🔗 डुप्लिकेट को सॉफ़्टलिंक से बदलें
modal-softlink-duplicates-info = प्रोसेस की जाने वाली कुल फ़ाइलें: { $count }। संचित वर्चुअल आकार: { $size }
modal-softlink-duplicates-warning = यह चयनित डुप्लिकेट फ़ाइलों को हटा देगा और उन्हें प्रत्येक समूह में शेष मूल फ़ाइल की ओर इशारा करते फ़ाइलसिस्टम-स्तरीय सॉफ़्टलिंक (सिंबॉलिक लिंक) से बदल देगा। इससे फ़ाइलें दृश्यतः बनी रहती हैं जबकि वास्तविक भौतिक स्टोरेज खाली हो जाता है।
modal-softlink-duplicates-checkbox = मैं पुष्टि करता हूँ कि मैं चयनित फ़ाइलों को सॉफ़्टलिंक से बदलना चाहता हूँ।
modal-softlink-duplicates-confirm = 🔗 हाँ, सॉफ़्टलिंक से बदलें

# Path Does Not Exist Modal
modal-path-not-exist-title = ❌ पथ मौजूद नहीं है!
modal-path-not-exist-msg = त्रुटि: जिस पथ को आप हटाने का प्रयास कर रहे हैं वह डिस्क पर मौजूद नहीं है।
modal-close-btn = बंद करें
modal-details-label = विवरण: 
modal-cancel-btn = रद्द करें

# Elevation Recommended Modal
modal-elevation-title = ⚠ व्यवस्थापक अधिकार अनुशंसित
modal-elevation-desc = eDirStat डिफ़ॉल्ट रूप से मानक उपयोगकर्ता विशेषाधिकारों के साथ चलता है। हालांकि, Windows रॉ फ़िज़िकल डिस्क हैंडल एक्सेस को व्यवस्थापक खातों तक सीमित रखता है।
modal-elevation-mft-disabled = Windows NTFS MFT ड्राइवर अक्षम
modal-elevation-mft-desc = प्रशासकीय विशेषाधिकारों के बिना, डायरेक्ट-टू-डिस्क MFT स्कैनर प्रारंभ नहीं हो सकता। फ़ाइल विश्लेषण फ़ॉलबैक मानक ट्रैवर्सल ड्राइवर का उपयोग करेगा, जिससे स्कैन प्रदर्शन 20x तक कम हो सकता है।
modal-elevation-relaunch-prompt = क्या आप अभी एप्लिकेशन को व्यवस्थापक विशेषाधिकारों के साथ पुनः लॉन्च करना चाहेंगे?
modal-elevation-continue-std = मानक उपयोगकर्ता के रूप में जारी रखें
modal-elevation-relaunch-btn = 🛡 एडमिन के रूप में पुनः लॉन्च करें

# About Modal
modal-about-title = ℹ eDirStat के बारे में
modal-about-author = लेखक: Cody Wyatt Neiman (xangelix) <neiman@cody.to>
modal-about-desc1 = Rust में निर्मित एक उच्च-प्रदर्शन डिस्क स्पेस विश्लेषक और डुप्लिकेशन टूलकिट।
modal-about-desc2 = इसमें समानांतर, वर्क-स्टीलिंग निर्देशिका ट्रैवर्सल, ज़ीरो-पार्सिंग लेआउट डीसीरियलाइज़ेशन वाले संपीड़ित स्नैपशॉट, और प्रतिक्रियाशील, इंटरैक्टिव ट्रीमैप शामिल हैं।
modal-about-desc3 = एकीकृत डुप्लिकेटर डुप्लिकेट समूहों को सुरक्षित रूप से अलग करने, पुनःप्राप्त करने योग्य स्थान की गणना करने और सिस्टम-स्तरीय हार्डलिंक का सम्मान करने के लिए बहु-चरणीय क्रिप्टोग्राफ़िक हैशिंग पाइपलाइन चलाता है।
modal-about-licenses-btn = ओपन सोर्स लाइसेंस देखें
modal-about-version = v{ $version }

# How Deduplication Works Modal
modal-how-dedup-title = ℹ डुप्लिकेशन कैसे काम करता है
modal-how-dedup-desc1 = प्रत्येक फ़ाइल के बाइट की सीधे तुलना करने के बजाय (जिसके लिए धीमी, युग्मित O(N²) स्कैन चाहिए), यह सिस्टम समान सामग्री की पहचान सुरक्षित और कुशलतापूर्वक करने के लिए अत्यधिक अनुकूलित 7-चरणीय पाइपलाइन का उपयोग करता है।
modal-how-dedup-pipeline-title = 7-चरणीय पाइपलाइन:
modal-how-dedup-why-title = यह पर्याप्त क्यों है?
modal-how-dedup-why-desc1 = यह बहु-चरणीय फ़िल्टर सुनिश्चित करता है कि केवल समान आकार, प्रीफ़िक्स, मिडपॉइंट, सफ़िक्स और वितरित ब्लॉक नमूनों वाली फ़ाइलें ही पूरी तरह पढ़ी जाएँ। अंत में, 256-bit BLAKE3 क्रिप्टोग्राफ़िक हैश की तुलना उद्योग-स्तरीय सुरक्षित ट्रांसफ़र प्रोटोकॉल जैसी सुरक्षा प्रदान करती है, जिससे धीमी, युग्मित बाइट-दर-बाइट तुलना की आवश्यकता समाप्त हो जाती है।

# How Deduplication Works Steps
modal-how-dedup-step1-title = 1. आकार विभाजन
modal-how-dedup-step1-desc = फ़ाइलों को बाइट में उनके सटीक आकार के अनुसार समूहित किया जाता है। अद्वितीय आकार वाली किसी भी फ़ाइल को तुरंत छोड़ दिया जाता है, जिससे डिस्क I/O पूरी तरह बायपास हो जाता है।
modal-how-dedup-step2-title = 2. प्रीफ़िक्स हैशिंग
modal-how-dedup-step2-desc = शेष उम्मीदवारों के पहले 4KB को हैश किया जाता है। इससे भिन्न हेडर या मेटाडेटा प्रारूप वाली फ़ाइलें जल्दी से फ़िल्टर हो जाती हैं।
modal-how-dedup-step3-title = 3. मिडपॉइंट हैशिंग
modal-how-dedup-step3-desc = शेष फ़ाइलों के केंद्र से एक 4KB ब्लॉक हैश किया जाता है, जिससे आंतरिक संरचनात्मक अंतर पकड़े जाते हैं।
modal-how-dedup-step4-title = 4. सफ़िक्स हैशिंग
modal-how-dedup-step4-desc = डेटा के अंतिम 4KB को हैश किया जाता है। यह अंतिम सामग्री या मेटाडेटा में अंतर की पहचान करने में अत्यंत प्रभावी है।
modal-how-dedup-step5-title = 5. मल्टी-रेंज हैशिंग
modal-how-dedup-step5-desc = बड़ी फ़ाइलों (100MB से अधिक) की पूरी लंबाई में आवधिक ब्लॉक सैम्पलिंग की जाती है, ताकि पूरी फ़ाइल पढ़े बिना सामग्री की एकरूपता सत्यापित हो सके।
modal-how-dedup-step6-title = 6. पूर्ण BLAKE3 हैशिंग
modal-how-dedup-step6-desc = शेष उम्मीदवारों के लिए पूर्ण BLAKE3 क्रिप्टोग्राफ़िक हैश की गणना की जाती है। 256-bit स्पेस के उच्च टकराव-प्रतिरोध के कारण, मेल खाते हैश यह दर्शाते हैं कि फ़ाइलों के भिन्न होने की संभावना खगोलीय रूप से नगण्य है, जो युग्मित तुलना के बिना अत्यंत विश्वसनीय पहचान प्रमाण प्रदान करता है।
modal-how-dedup-step7-title = 7. टाइमस्टैम्प सत्यापन
modal-how-dedup-step7-desc = कोई भी डुप्लिकेशन कार्रवाई प्रदर्शित या निष्पादित करने से ठीक पहले, एप्लिकेशन स्नैपशॉट निर्माण के बाद हुए परिवर्तनों से बचाव के लिए डिस्क पर फ़ाइलों के टाइमस्टैम्प सत्यापित करता है।

# Open Source Licenses Modal
modal-licenses-title = 📜 ओपन सोर्स लाइसेंस
modal-licenses-desc = इस एप्लिकेशन में निम्नलिखित थर्ड-पार्टी लाइब्रेरी और क्रेट उपयोग किए गए हैं:

# Processing Modal
modal-processing-title = ⏳ प्रोसेस किया जा रहा है...
modal-processing-deletion = फ़ाइलें और निर्देशिकाएँ हटाई जा रही हैं...
modal-processing-trash = फ़ाइलें और निर्देशिकाएँ ट्रैश में ले जाई जा रही हैं...
modal-processing-hardlink = डुप्लिकेट को हार्डलिंक से बदला जा रहा है...
modal-processing-softlink = डुप्लिकेट को सॉफ़्टलिंक से बदला जा रहा है...

# Explorer Column Headers
explorer-hdr-name = नाम
explorer-hdr-percentage = प्रतिशत
explorer-hdr-size = आकार
explorer-hdr-items = आइटम
explorer-hdr-files = फ़ाइलें
explorer-hdr-subdirs = उपनिर्देशिकाएँ
explorer-hdr-created = निर्मित
explorer-hdr-modified = संशोधित

# Update Checker
update-checking = अपडेट की जाँच की जा रही है...
update-available = नया संस्करण { $version } उपलब्ध है!
update-up-to-date = आप अप-टू-डेट हैं
update-failed = अपडेट जाँच विफल: { $error }

# Themes
theme = 🎨 थीम
theme-dark = डार्क
theme-high-contrast = हाई कॉन्ट्रास्ट
theme-light = लाइट
theme-system = सिस्टम

# New Scan Options Modal
modal-scan-options-title = नए स्कैन विकल्प
modal-scan-options-header = नया स्कैन शुरू करें
modal-scan-options-path-label = स्कैन करने के लिए निर्देशिका पथ:
modal-scan-options-paste-tooltip = क्लिपबोर्ड से पेस्ट करें
modal-scan-options-browse-tooltip = फ़ोल्डर ब्राउज़ करें...
modal-scan-options-scan-btn = स्कैन
modal-scan-options-cancel-btn = रद्द करें
modal-scan-options-same-filesystem = स्कैन को उसी फ़ाइलसिस्टम/वॉल्यूम तक सीमित रखें
modal-scan-options-drives-header = 💽 स्टोरेज ड्राइव और वॉल्यूम
modal-scan-options-refresh-tooltip = स्टोरेज ड्राइव रीफ़्रेश करें
modal-scan-options-root-system = रूट सिस्टम
modal-scan-options-selected-badge = ✅ चयनित
modal-scan-options-free-of = { $total } में से { $free } खाली
modal-scan-options-subtitle = विश्लेषण के लिए कोई स्टोरेज वॉल्यूम, त्वरित स्थान या कस्टम निर्देशिका चुनें।
modal-scan-options-quick-access = 📍 त्वरित एक्सेस शॉर्टकट
modal-scan-options-path-hint = /path/to/scan
modal-scan-options-hint = ℹ ऊपर कोई ड्राइव चुनें या निर्देशिका पथ दर्ज करें।
modal-scan-options-sandbox-auth = 🔒 सैंडबॉक्स एक्सेस आवश्यक — एक्सेस देने के लिए स्कैन पर क्लिक करें
modal-scan-options-valid-dir = ✅ मान्य निर्देशिका — स्कैन के लिए तैयार
modal-scan-options-points-to-file = ⚠ पथ किसी फ़ाइल की ओर इशारा करता है — कृपया कोई फ़ोल्डर चुनें।
modal-scan-options-dir-not-exist = ⚠ निर्देशिका फ़ाइलसिस्टम पर मौजूद नहीं है।
quick-loc-home = 🏠 होम
quick-loc-documents = 📄 दस्तावेज़
quick-loc-downloads = 📥 डाउनलोड
quick-loc-desktop = 🖥 डेस्कटॉप
quick-loc-pictures = 🖼 चित्र
search-use-regex = रेगुलर एक्सप्रेशन (Regex) का उपयोग करें
search-match-case = केस का मिलान करें (केस संवेदनशील)
dedup-pref-dir-hint = उदा. /home/user/Archive

file-menu-close = स्कैन बंद करें
file-menu-quit = बाहर निकलें

badge-dataless-cloud = क्लाउड / डेटालेस फ़ाइल
badge-symlink = सिंबॉलिक लिंक
badge-special-file = विशेष फ़ाइल (पाइप / सॉकेट / डिवाइस)
badge-permission-denied = पहुँच अस्वीकृत
