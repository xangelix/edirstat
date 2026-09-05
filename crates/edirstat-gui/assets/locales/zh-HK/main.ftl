# Menu Bar Dropdowns
file = 檔案
view = 檢視
help = 說明

# Menu Bar Actions
new-scan = 📁 新掃瞄
save-snapshot = 💾 儲存快照
load-snapshot = 📖 載入快照

# Menu Bar Status
idle = 閒置

# View Menu Options
monospace-paths = 🅰 等寬路徑
highlight-duplicates = ✨ 醒目顯示重複檔案
treemap-borders = 🔳 矩形樹圖框線
treemap-style =  矩形樹圖樣式
treemap-style-vertical = 垂直漸層
treemap-style-offset-vertical = 偏移垂直漸層
treemap-style-diagonal = 對角漸層
treemap-style-cushion = 緩墊立體效果
deletion-confirmation = 🗑 刪除確認
trash-confirmation = ♻ 回收站確認
time-format = 🕒 時間格式
language = 💬 語言
layout-mode = 版面配置模式：
classic-layout = 傳統版面配置
windirstat-layout = WinDirStat 版面配置
vis-mode-treemap = 📊 矩形樹圖
vis-mode-plots = 📈 圖表
select-plot-label = 選取圖表：
vis-mode-deduplicator = 👥 重複檔案搜尋器
search-filter-label = 🔍 篩選：

# Panel Toggles
toggle-left-panel = { $collapsed ->
    [true] ▶ 顯示左面板 (F9)
   *[false] ◀ 隱藏左面板 (F9)
}

toggle-right-panel = { $collapsed ->
    [true] { $is_classic ->
        [true] ◀ 顯示右面板 (F11)
       *[false] ▶ 顯示副檔名面板 (F11)
    }
   *[false] { $is_classic ->
        [true] ▶ 隱藏右面板 (F11)
       *[false] ◀ 隱藏副檔名面板 (F11)
    }
}

collapse-all = ⏏ 全部摺疊
about = ℹ 關於
web-not-available = 此功能在網頁版本中無法使用

# Status Indicators
scanning-disk = 正在掃瞄磁碟…
scan-complete = 掃瞄完成
scan-cancelled = 掃瞄已取消
path-label = 路徑：{ $path }
worker-threads = ⚡ { $count } 個工作線程
worker-threads-hover = 分配用於目錄遍歷的平行、工作竊取 CPU 核心數目。

# Stats Panel (Bottom)
directories-count = 📁 目錄：{ $count }
files-count = 📄 檔案：{ $count }
total-size = 💾 總大小：{ $size }
elapsed-time = ⏱ 時間：{ $time }
scan-speed = ⚡ 速度：{ $speed }/s

# Selection Info
selection-path = 選取項目：{ $path }
selection-items = 選取項目：{ $count ->
   *[other] { $count } 個項目
}

# Plot Types
plot-size-distribution = 📊 檔案大小分佈
plot-age-size = 🌌 檔案時齡對檔案大小
plot-dir-composition = 🍰 目錄組成
plot-extension-boxplot = 📦 按副檔名分類的檔案大小
plot-temporal-timeline = ⏱ 連動時間軸
plot-deduplicator-waste = 👥 按副檔名分類的重複浪費空間

# --- Deduplicator Strings ---
dedup-desc = 使用密碼學安全的 BLAKE3 雜湊，尋找並安全移除逐位元組完全相同的檔案。
dedup-how-it-works = ℹ 運作方式
dedup-min-size = 最小檔案大小：
dedup-ignore-system = 忽略系統檔案
dedup-ignore-hidden = 忽略隱藏檔案
dedup-start-scan = ⚡ 開始去重掃瞄
dedup-scan-first = 請先掃瞄目錄。
dedup-cancelled-msg = 掃瞄已取消。請開始新的掃瞄以尋找重複檔案。
dedup-analyzing = 正在分析檔案…
dedup-no-duplicates = 找不到重複群組。請嘗試調低最小檔案大小，或掃瞄其他資料夾。
no-permission = 沒有權限
hardlink-badge = 硬連結
dedup-select-items = 🎯 選取項目…
dedup-select-all-but-oldest = 🎯 除最舊外全選
dedup-select-all-but-newest = 🎯 除最新外全選
dedup-select-all-but-shortest = 🎯 除最短路徑外全選
dedup-select-all-but-rootmost = 🎯 除最靠近根目錄外全選
dedup-select-all-but-longest = 🎯 除最長路徑外全選
dedup-pref-dir-pattern = 偏好目錄模式：
dedup-select-all-but-pref = 🎯 除偏好目錄外全選
dedup-clear-selection = ❌ 清除選取
dedup-link-menu = 🔗 連結…（{ $count } 個檔案）
dedup-link-menu-disabled = 🔗 連結…（0 個檔案）
dedup-link-hardlinks = 🔗 以硬連結取代所選檔案
dedup-link-softlinks = 🔗 以軟連結取代所選檔案
dedup-remove-menu = 🗑 移除…（{ $count } 個檔案，{ $size }）
dedup-remove-menu-disabled = 🗑 移除…（0 個檔案）
dedup-remove-trash = ♻ 將所選項目移至回收站
dedup-remove-delete = 🗑 永久刪除所選項目
dedup-warning-title = ⚠ 資料遺失警告
dedup-warning-desc = { $count ->
   *[other] 即將刪除 { $count } 個檔案的所有版本
}
dedup-warning-no-original = 將不會保留原始複本：
dedup-warning-details = 您已同時勾選下列檔案的原始複本及所有重複複本。刪除這些檔案很可能導致永久性資料遺失：
dedup-cancel-hover = 按一下以取消掃瞄
scan-cancel-hover = 按一下以取消掃瞄
dedup-current-label = 目前
dedup-phase1-size = 階段 1/7：正在按大小將所有已掃瞄的檔案分組…
dedup-phase1-filter = 階段 1/7：正在篩選重複候選檔案的排除項目…
dedup-phase2-prefix = 階段 2/7：正在對檔案前置區塊（首 4KB）計算雜湊…
dedup-phase3-midpoint = 階段 3/7：正在對檔案中點計算雜湊…
dedup-phase4-suffix = 階段 4/7：正在對檔案後置區塊計算雜湊…
dedup-phase5-multirange = 階段 5/7：正在對大型檔案進行多範圍雜湊…
dedup-phase6-full = 階段 6/7：正在對其餘候選檔案進行完整 BLAKE3 雜湊…
dedup-phase7-validation = 階段 7/7：正在進行最終時間戳記驗證…
dedup-phase-finished = 已在 { $duration } 內完成！找到 { $count } 個重複群組。潛在可回收空間：{ $space }
dedup-scan-cancelled-with-error = 掃瞄已取消：{ $error }

# Deduplicator Table Headers
dedup-hdr-checkbox = [     ]
dedup-hdr-filename = 檔案名稱
dedup-hdr-directory = 上層目錄
dedup-hdr-size = 大小
dedup-hdr-reclaimable = 可回收
dedup-hdr-created = 建立時間
dedup-hdr-modified = 修改時間
dedup-copies-selected = ({ $count ->
   *[other] 已選取 { $count } 個複本
})

# --- Explorer Details Panel ---
explorer-details-header = ℹ 詳細資料
explorer-deselect-hover = 取消選取項目
explorer-deselect-single-hover = 取消選取此項目
explorer-selected-items-count = { $count ->
   *[other] 已選取 { $count } 個項目
}
explorer-total-size = 總大小：{ $size }
explorer-files = 檔案：{ $count }
explorer-directories = 目錄：{ $count }
explorer-actions-title = 動作
explorer-actions-operations = 操作：
explorer-action-refresh-hover = 重新整理所有選取的目錄子樹
explorer-grid-type = 類型：
explorer-grid-size = 大小：
explorer-grid-bytes = 位元組：
explorer-grid-items = 項目：
explorer-grid-files = 檔案：
explorer-grid-subdirs = 子目錄：
explorer-grid-user = 使用者：
explorer-grid-group = 群組：
explorer-grid-permissions = 權限：
explorer-grid-path = 完整路徑：

# Explorer Type Names
type-symlink = 符號連結
type-directory = 目錄
type-file = 檔案

# Explorer Actions
explorer-action-copy-path = 📋 複製路徑
explorer-action-open-file = 📄 開啟檔案
explorer-action-open-manager = 🗁 開啟檔案管理員
explorer-action-refresh-subtree = 🔄 重新整理子樹
explorer-action-move-trash = ♻ 移至回收站
explorer-action-delete-permanently = 🗑 永久刪除
explorer-action-refresh-directory = 🔄 重新整理目錄

# Explorer Empty State
explorer-empty-state = 按一下「新掃瞄」以瀏覽磁碟使用情況。
choose-an-option = 請選擇一個選項
web-viewer = 網頁檢視器
load-demo = 👁 載入範例示範快照
placeholder-treemap = 掃瞄後的檔案系統將在此以矩形樹圖顯示。
placeholder-plots = 掃瞄後的檔案系統將在此繪製成圖表。

# Treemap Zoom & Navigation
zoom-up = ⏶ 向上
zoom-reset = ❌ 重設
zoom-to-dir = 🔍 在樹狀圖中聚焦
zoom-up-level = ⏶ 向上級目錄
zoom-empty-dir = 目錄為空

# --- Extensions Panel ---
extensions-header = 📂 副檔名
extensions-empty = 尚未收集統計資料。
extensions-hover-files = 檔案：{ $count }

# --- Operations (Context Actions) ---
op-up-one-level = 向上一層
op-zoom-treemap = 在樹狀圖中聚焦
op-refresh-entire-scan = 重新整理整個掃瞄
op-refresh-directory = 重新整理目錄
op-open-file = 開啟檔案
op-open-file-manager = 在檔案管理員中開啟
op-open-terminal = 在此開啟終端機
op-copy-path = 複製路徑
op-copy-name = 複製名稱
op-move-trash = 移至回收站
op-permanently-delete = 永久刪除

# Toast Notifications
toast-already-root = 已在根層級
toast-navigated-up = 已移至上一層
toast-zoomed-treemap = 已在樹狀圖中聚焦至目錄
toast-refreshing-scan = 正在重新整理整個掃瞄…
toast-refreshing-dir = 正在重新整理選取的目錄…
toast-opened-file = 已開啟：{ $path }
toast-failed-open-file = 無法開啟檔案：{ $error }
toast-opened-manager = 已在檔案管理員中開啟：{ $path }
toast-failed-open-manager = 無法在檔案管理員中開啟：{ $error }
toast-opened-terminal = 已開啟終端機：{ $path }
toast-failed-open-terminal = 無法開啟終端機：{ $error }
toast-copied-paths = 已複製 { $count ->
   *[other] { $count } 個路徑到剪貼簿
}
toast-copied-names = 已複製 { $count ->
   *[other] { $count } 個名稱到剪貼簿
}

# --- Modals ---
modal-remember-confirmation = 記住此確認，並套用至日後的所有檔案及目錄
modal-process-multiple = 您即將處理 { $count } 個重複檔案/項目：
modal-process-single = 您即將處理下列路徑：
# Confirm Deletion/Trash/Link Modals
modal-delete-title = ⚠ 永久刪除警告
modal-delete-header = ⚠ 永久刪除警告！
modal-delete-info = 總大小：{ $size }
modal-delete-warning = 這是遞迴刪除。所選路徑下的所有檔案、資料夾及子目錄都會被永久刪除，且無法復原（不會經過回收站）。
modal-delete-checkbox = 我明白檔案將會被永久刪除，且無法復原。
modal-delete-confirm = 🗑 是，永久刪除

modal-trash-title = ♻ 移至回收站
modal-trash-header = ♻ 移至回收站
modal-trash-info = 總大小：{ $size }
modal-trash-warning = 這會將所選路徑及其所有內容移至系統回收站，您之後可以在回收站復原或永久刪除這些項目。
modal-trash-checkbox = 我確認要將其移至回收站。
modal-trash-confirm = ♻ 是，移至回收站

modal-delete-duplicates-title = ⚠ 永久去重警告
modal-delete-duplicates-header = ⚠ 永久刪除重複檔案警告！
modal-delete-duplicates-info = 可回收的總空間：{ $size }
modal-delete-duplicates-warning = 所有選取的檔案都會被永久刪除，且無法復原（不會經過回收站）。
modal-delete-duplicates-checkbox = 我明白檔案將會被永久刪除，且無法復原。
modal-delete-duplicates-confirm = 🗑 是，永久刪除所選項目

modal-trash-duplicates-title = ♻ 將重複檔案移至回收站
modal-trash-duplicates-header = ♻ 將重複檔案移至回收站
modal-trash-duplicates-info = 可回收的總空間：{ $size }
modal-trash-duplicates-warning = 所有選取的檔案都會移至回收站。
modal-trash-duplicates-checkbox = 我確認要將這些檔案移至回收站。
modal-trash-duplicates-confirm = ♻ 是，將所選項目移至回收站

modal-hardlink-duplicates-title = 🔗 以硬連結取代重複檔案
modal-hardlink-duplicates-header = 🔗 以硬連結取代重複檔案
modal-hardlink-duplicates-info = 要處理的檔案總數：{ $count }。累計虛擬大小：{ $size }
modal-hardlink-duplicates-warning = 這會刪除所選的重複檔案，並以指向各群組中所保留原始檔案的檔案系統層級硬連結取代。這樣可以在保留檔案外觀的同時，釋放實際的實體儲存空間。
modal-hardlink-duplicates-checkbox = 我確認要以硬連結取代所選檔案。
modal-hardlink-duplicates-confirm = 🔗 是，以硬連結取代

modal-softlink-duplicates-title = 🔗 以軟連結取代重複檔案
modal-softlink-duplicates-header = 🔗 以軟連結取代重複檔案
modal-softlink-duplicates-info = 要處理的檔案總數：{ $count }。累計虛擬大小：{ $size }
modal-softlink-duplicates-warning = 這會刪除所選的重複檔案，並以指向各群組中所保留原始檔案的檔案系統層級軟連結（符號連結）取代。這樣可以在保留檔案外觀的同時，釋放實際的實體儲存空間。
modal-softlink-duplicates-checkbox = 我確認要以軟連結取代所選檔案。
modal-softlink-duplicates-confirm = 🔗 是，以軟連結取代

# Path Does Not Exist Modal
modal-path-not-exist-title = ❌ 路徑不存在！
modal-path-not-exist-msg = 錯誤：您嘗試刪除的路徑不存在於磁碟上。
modal-close-btn = 關閉
modal-details-label = 詳細資料： 
modal-cancel-btn = 取消

# Elevation Recommended Modal
modal-elevation-title = ⚠ 建議提升權限
modal-elevation-desc = eDirStat 預設以標準使用者權限執行。不過，Windows 嚴格限制僅系統管理員帳戶可存取原始實體磁碟控制代碼。
modal-elevation-mft-disabled = Windows NTFS MFT 驅動程式已停用
modal-elevation-mft-desc = 沒有系統管理員權限時，直接讀取磁碟的 MFT 掃瞄器將無法初始化。檔案分析會改用後備的標準遍歷驅動程式，掃瞄效能最多可降低達 20 倍。
modal-elevation-relaunch-prompt = 要立即以系統管理員權限重新啟動應用程式嗎？
modal-elevation-continue-std = 以標準使用者身分繼續
modal-elevation-relaunch-btn = 🛡 以系統管理員身分重新啟動

# About Modal
modal-about-title = ℹ 關於 eDirStat
modal-about-author = By: Cody Wyatt Neiman (xangelix) <neiman@cody.to>
modal-about-desc1 = 以 Rust 構建的高效能磁碟空間分析與去重工具組。
modal-about-desc2 = 具備平行的工作竊取目錄遍歷、採用零剖析配置還原序列化的壓縮快照，以及反應靈敏的互動式矩形樹圖。
modal-about-desc3 = 內建的去重工具會執行多階段密碼學雜湊管線，安全地隔離重複群組、計算可回收空間，並尊重系統層級的硬連結。
modal-about-licenses-btn = 檢視開放原始碼授權
modal-about-version = v{ $version }

# How Deduplication Works Modal
modal-how-dedup-title = ℹ 去重運作方式
modal-how-dedup-desc1 = 本系統不會直接比較每個檔案的位元組（這需要緩慢的兩兩 O(N²) 掃瞄），而是採用高度最佳化的 7 階段管線，安全又有效率地識別相同內容。
modal-how-dedup-pipeline-title = 7 階段管線：
modal-how-dedup-why-title = 為何這樣已經足夠？
modal-how-dedup-why-desc1 = 這個多階段篩選可確保只有大小、前置區塊、中點、後置區塊及分散式區塊取樣都完全相同的檔案，才會被完整讀取。最後，比對 256-bit BLAKE3 密碼學雜湊可提供媲美業界級安全傳輸通訊協定的安全性，免卻緩慢的逐位元組兩兩比較。

# How Deduplication Works Steps
modal-how-dedup-step1-title = 1. 大小分割
modal-how-dedup-step1-desc = 檔案會依其精確的位元組大小分組。任何大小獨一無二的檔案都會立即被捨棄，完全略過磁碟 I/O。
modal-how-dedup-step2-title = 2. 前置雜湊
modal-how-dedup-step2-desc = 系統會對其餘候選檔案的首 4KB 計算雜湊。這能快速篩除標頭或中繼資料格式不同的檔案。
modal-how-dedup-step3-title = 3. 中點雜湊
modal-how-dedup-step3-desc = 系統會對其餘檔案中央的一個 4KB 區塊計算雜湊，以捕捉內部結構差異。
modal-how-dedup-step4-title = 4. 後置雜湊
modal-how-dedup-step4-desc = 系統會對資料的最後 4KB 計算雜湊。這對識別結尾內容或中繼資料的差異非常有效。
modal-how-dedup-step5-title = 5. 多範圍雜湊
modal-how-dedup-step5-desc = 大型檔案（超過 100MB）會在其整個長度範圍內進行定期區塊取樣，無需讀取整個檔案即可驗證內容一致性。
modal-how-dedup-step6-title = 6. 完整 BLAKE3 雜湊
modal-how-dedup-step6-desc = 系統會為其餘候選檔案計算完整的 BLAKE3 密碼學雜湊。由於 256-bit 空間具備極高的抗碰撞性，雜湊相符即表示檔案內容不同的可能性微乎其微，無需兩兩比較即可提供高度可靠的身分證明。
modal-how-dedup-step7-title = 7. 時間戳記驗證
modal-how-dedup-step7-desc = 在顯示或執行任何去重動作之前，應用程式會先驗證檔案在磁碟上的時間戳記，以防範快照產生後所發生的變更。

# Open Source Licenses Modal
modal-licenses-title = 📜 開放原始碼授權
modal-licenses-desc = 此應用程式使用了下列第三方程式庫及 crates：

# Processing Modal
modal-processing-title = ⏳ 正在處理…
modal-processing-deletion = 正在刪除檔案及目錄…
modal-processing-trash = 正在將檔案及目錄移至回收站…
modal-processing-hardlink = 正在以硬連結取代重複檔案…
modal-processing-softlink = 正在以軟連結取代重複檔案…

# Explorer Column Headers
explorer-hdr-name = 名稱
explorer-hdr-percentage = 百分比
explorer-hdr-size = 大小
explorer-hdr-items = 項目
explorer-hdr-files = 檔案
explorer-hdr-subdirs = 子目錄
explorer-hdr-created = 建立時間
explorer-hdr-modified = 修改時間

# Update Checker
update-checking = 正在檢查更新…
update-available = 有新版本 { $version } 可供使用！
update-up-to-date = 已是最新版本
update-failed = 更新檢查失敗：{ $error }

# Themes
theme = 🎨 主題
theme-dark = 深色
theme-high-contrast = 高對比度
theme-light = 淺色
theme-system = 系統

# New Scan Options Modal
modal-scan-options-title = 新掃瞄選項
modal-scan-options-header = 開始新掃瞄
modal-scan-options-path-label = 要掃瞄的目錄路徑：
modal-scan-options-paste-tooltip = 從剪貼簿貼上
modal-scan-options-browse-tooltip = 瀏覽資料夾…
modal-scan-options-scan-btn = 掃瞄
modal-scan-options-cancel-btn = 取消
modal-scan-options-same-filesystem = 將掃瞄限制於相同的檔案系統/磁碟區
modal-scan-options-drives-header = 💽 儲存磁碟機及磁碟區
modal-scan-options-refresh-tooltip = 重新整理儲存磁碟機
modal-scan-options-root-system = 根系統
modal-scan-options-selected-badge = ✅ 已選取
modal-scan-options-free-of = { $free } 可用，共 { $total }
modal-scan-options-subtitle = 選擇要分析的儲存卷、捷徑位置或自訂目錄。
modal-scan-options-quick-access = 📍 捷徑存取
modal-scan-options-path-hint = /要掃描的路徑
modal-scan-options-hint = ℹ 請在上方選取磁碟機或輸入目錄路徑。
modal-scan-options-sandbox-auth = 🔒 需要沙盒存取權限 — 點擊「掃描」以授予存取權限
modal-scan-options-valid-dir = ✅ 有效目錄 — 隨時可掃描
modal-scan-options-points-to-file = ⚠ 路徑指向一個檔案 — 請選取資料夾。
modal-scan-options-dir-not-exist = ⚠ 目錄在檔案系統中不存在。
quick-loc-home = 🏠 用戶目錄
quick-loc-documents = 📄 文件
quick-loc-downloads = 📥 下載
quick-loc-desktop = 🖥 桌面
quick-loc-pictures = 🖼 圖片
search-use-regex = 使用正規表示式 (Regex)
search-match-case = 區分大小寫
dedup-pref-dir-hint = 例如 /home/user/Archive
