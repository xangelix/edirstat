# Menu Bar Dropdowns
file = 文件
view = 视图
help = 帮助

# Menu Bar Actions
new-scan = 📁 新建扫描
save-snapshot = 💾 保存快照
load-snapshot = 📖 加载快照

# Menu Bar Status
idle = 空闲

# View Menu Options
monospace-paths = 🅰 等宽路径
highlight-duplicates = ✨ 高亮重复文件
treemap-borders = 🔳 矩形树图边框
treemap-style =  矩形树图样式
treemap-style-vertical = 垂直渐变
treemap-style-offset-vertical = 偏移垂直渐变
treemap-style-diagonal = 对角渐变
treemap-style-cushion = 软垫着色
deletion-confirmation = 🗑 删除确认
trash-confirmation = ♻ 回收站确认
time-format = 🕒 时间格式
language = 💬 语言
layout-mode = 布局模式：
classic-layout = 经典布局
windirstat-layout = WinDirStat 布局
vis-mode-treemap = 📊 矩形树图
vis-mode-plots = 📈 图表
select-plot-label = 选择图表：
vis-mode-deduplicator = 👥 重复文件查找器
search-filter-label = 🔍 筛选：

# Panel Toggles
toggle-left-panel = { $collapsed ->
    [true] ▶ 显示左侧面板 (F9)
   *[false] ◀ 隐藏左侧面板 (F9)
}

toggle-right-panel = { $collapsed ->
    [true] { $is_classic ->
        [true] ◀ 显示右侧面板 (F11)
       *[false] ▶ 显示扩展名面板 (F11)
    }
   *[false] { $is_classic ->
        [true] ▶ 隐藏右侧面板 (F11)
       *[false] ◀ 隐藏扩展名面板 (F11)
    }
}

collapse-all = ⏏ 全部折叠
about = ℹ 关于
web-not-available = 此功能在网页版中不可用

# Status Indicators
scanning-disk = 正在扫描磁盘…
scan-complete = 扫描完成
scan-cancelled = 扫描已取消
path-label = 路径：{ $path }
worker-threads = ⚡ { $count } 个工作线程
worker-threads-hover = 分配给目录遍历的并行工作窃取式 CPU 核心数量。

# Stats Panel (Bottom)
directories-count = 📁 目录：{ $count }
files-count = 📄 文件：{ $count }
total-size = 💾 总大小：{ $size }
elapsed-time = ⏱ 时间：{ $time }
scan-speed = ⚡ 速度：{ $speed }/s

# Selection Info
selection-path = 选中：{ $path }
selection-items = 选中：{ $count ->
   *[other] { $count } 个项目
}

# Plot Types
plot-size-distribution = 📊 文件大小分布
plot-age-size = 🌌 文件年龄与文件大小
plot-dir-composition = 🍰 目录构成
plot-extension-boxplot = 📦 按扩展名划分的文件大小
plot-temporal-timeline = ⏱ 联动时间线
plot-deduplicator-waste = 👥 按扩展名划分的重复浪费空间

# --- Deduplicator Strings ---
dedup-desc = 使用加密安全的 BLAKE3 哈希，查找并安全移除字节完全相同的文件。
dedup-how-it-works = ℹ 工作原理
dedup-min-size = 最小文件大小：
dedup-ignore-system = 忽略系统文件
dedup-ignore-hidden = 忽略隐藏文件
dedup-start-scan = ⚡ 开始去重扫描
dedup-scan-first = 请先扫描一个目录。
dedup-cancelled-msg = 扫描已取消。请开始新的扫描以查找重复文件。
dedup-analyzing = 正在分析文件…
dedup-no-duplicates = 未找到重复文件组。请尝试降低最小文件大小，或扫描其他文件夹。
no-permission = 无权限
hardlink-badge = 硬链接
dedup-select-items = 🎯 选择项目…
dedup-select-all-but-oldest = 🎯 除最旧以外全部
dedup-select-all-but-newest = 🎯 除最新以外全部
dedup-select-all-but-shortest = 🎯 除路径最短以外全部
dedup-select-all-but-rootmost = 🎯 除最顶层以外全部
dedup-select-all-but-longest = 🎯 除路径最长以外全部
dedup-pref-dir-pattern = 首选目录模式：
dedup-select-all-but-pref = 🎯 除首选目录以外全部
dedup-clear-selection = ❌ 清除选择
dedup-link-menu = 🔗 链接…（{ $count } 个文件）
dedup-link-menu-disabled = 🔗 链接…（0 个文件）
dedup-link-hardlinks = 🔗 将选中项替换为硬链接
dedup-link-softlinks = 🔗 将选中项替换为软链接
dedup-remove-menu = 🗑 移除…（{ $count } 个文件，{ $size }）
dedup-remove-menu-disabled = 🗑 移除…（0 个文件）
dedup-remove-trash = ♻ 将选中项移至回收站
dedup-remove-delete = 🗑 永久删除选中项
dedup-warning-title = ⚠ 数据丢失警告
dedup-warning-desc = { $count ->
   *[other] 将删除 { $count } 个文件的所有版本
}
dedup-warning-no-original = 不会保留任何原始副本：
dedup-warning-details = 你已勾选下列文件的原始副本和所有重复副本。删除它们很可能导致永久性数据丢失：
dedup-cancel-hover = 单击取消扫描
scan-cancel-hover = 单击取消扫描
dedup-current-label = 当前
dedup-phase1-size = 阶段 1/7：正在按大小对所有已扫描文件分组…
dedup-phase1-filter = 阶段 1/7：正在对重复候选文件应用排除筛选…
dedup-phase2-prefix = 阶段 2/7：正在对文件前缀（前 4KB）进行哈希…
dedup-phase3-midpoint = 阶段 3/7：正在对文件中点进行哈希…
dedup-phase4-suffix = 阶段 4/7：正在对文件后缀进行哈希…
dedup-phase5-multirange = 阶段 5/7：正在对大文件进行多区段哈希…
dedup-phase6-full = 阶段 6/7：正在对剩余候选文件进行完整 BLAKE3 哈希…
dedup-phase7-validation = 阶段 7/7：正在进行最终时间戳验证…
dedup-phase-finished = 已完成，用时 { $duration }！找到 { $count } 组重复文件。潜在可回收空间：{ $space }
dedup-scan-cancelled-with-error = 扫描已取消：{ $error }

# Deduplicator Table Headers
dedup-hdr-checkbox = [     ]
dedup-hdr-filename = 文件名
dedup-hdr-directory = 所在目录
dedup-hdr-size = 大小
dedup-hdr-reclaimable = 可回收
dedup-hdr-created = 创建时间
dedup-hdr-modified = 修改时间
dedup-copies-selected = （{ $count ->
   *[other] 已选中 { $count } 个副本
}）

# --- Explorer Details Panel ---
explorer-details-header = ℹ 详细信息
explorer-deselect-hover = 取消选择项目
explorer-deselect-single-hover = 取消选择该项目
explorer-selected-items-count = { $count ->
   *[other] 已选中 { $count } 个项目
}
explorer-total-size = 总大小：{ $size }
explorer-files = 文件：{ $count }
explorer-directories = 目录：{ $count }
explorer-actions-title = 操作
explorer-actions-operations = 操作：
explorer-action-refresh-hover = 刷新所有选中的目录子树
explorer-grid-type = 类型：
explorer-grid-size = 大小：
explorer-grid-bytes = 字节数：
explorer-grid-items = 项目数：
explorer-grid-files = 文件数：
explorer-grid-subdirs = 子目录数：
explorer-grid-user = 用户：
explorer-grid-group = 用户组：
explorer-grid-permissions = 权限：
explorer-grid-path = 完整路径：

# Explorer Type Names
type-symlink = 符号链接
type-directory = 目录
type-file = 文件

# Explorer Actions
explorer-action-copy-path = 📋 复制路径
explorer-action-open-file = 📄 打开文件
explorer-action-open-manager = 🗁 打开文件管理器
explorer-action-refresh-subtree = 🔄 刷新子树
explorer-action-move-trash = ♻ 移至回收站
explorer-action-delete-permanently = 🗑 永久删除
explorer-action-refresh-directory = 🔄 刷新目录

# Explorer Empty State
explorer-empty-state = 单击“新建扫描”以浏览磁盘使用情况。
choose-an-option = 请选择一个选项
web-viewer = 网页查看器
load-demo = 👁 加载示例演示快照
placeholder-treemap = 扫描后的文件系统将在此以矩形树图可视化。
placeholder-plots = 扫描后的文件系统将在此绘制图表。

# Treemap Zoom & Navigation
zoom-up = ⏶ 向上
zoom-reset = ❌ 重置
zoom-to-dir = 🔍 在树状图中聚焦
zoom-up-level = ⏶ 向上级目录
zoom-empty-dir = 目录为空

# --- Extensions Panel ---
extensions-header = 📂 扩展名
extensions-empty = 尚未收集到统计数据。
extensions-hover-files = 文件：{ $count }

# --- Operations (Context Actions) ---
op-up-one-level = 向上一级
op-zoom-treemap = 在树状图中聚焦
op-refresh-entire-scan = 刷新整个扫描
op-refresh-directory = 刷新目录
op-open-file = 打开文件
op-open-file-manager = 在文件管理器中打开
op-open-terminal = 在此处打开终端
op-copy-path = 复制路径
op-copy-name = 复制名称
op-move-trash = 移至回收站
op-permanently-delete = 永久删除

# Toast Notifications
toast-already-root = 已位于根目录层级
toast-navigated-up = 已向上移动一级
toast-zoomed-treemap = 已在树状图中聚焦到目录
toast-refreshing-scan = 正在刷新整个扫描…
toast-refreshing-dir = 正在刷新所选目录…
toast-opened-file = 已打开：{ $path }
toast-failed-open-file = 打开文件失败：{ $error }
toast-opened-manager = 已在文件管理器中打开：{ $path }
toast-failed-open-manager = 在文件管理器中打开失败：{ $error }
toast-opened-terminal = 已在以下位置打开终端：{ $path }
toast-failed-open-terminal = 打开终端失败：{ $error }
toast-copied-paths = 已将 { $count ->
   *[other] { $count } 个路径复制到剪贴板
}
toast-copied-names = 已将 { $count ->
   *[other] { $count } 个名称复制到剪贴板
}

# --- Modals ---
modal-remember-confirmation = 记住以后所有文件和目录的确认选择
modal-process-multiple = 你将要处理 { $count } 个重复文件/项目：
modal-process-single = 你将要处理以下路径：
# Confirm Deletion/Trash/Link Modals
modal-delete-title = ⚠ 永久删除警告
modal-delete-header = ⚠ 永久删除警告！
modal-delete-info = 总大小：{ $size }
modal-delete-warning = 此操作为递归删除。所选路径下的所有文件、文件夹和子目录都将被永久删除且无法恢复（不经过回收站）。
modal-delete-checkbox = 我理解文件将被永久删除且无法恢复。
modal-delete-confirm = 🗑 是，永久删除

modal-trash-title = ♻ 移至回收站
modal-trash-header = ♻ 移至回收站
modal-trash-info = 总大小：{ $size }
modal-trash-warning = 这将把所选路径及其全部内容移至系统回收站，之后可从中恢复或永久删除。
modal-trash-checkbox = 我确认要将其移至回收站。
modal-trash-confirm = ♻ 是，移至回收站

modal-delete-duplicates-title = ⚠ 永久去重警告
modal-delete-duplicates-header = ⚠ 永久删除重复文件警告！
modal-delete-duplicates-info = 可回收空间总计：{ $size }
modal-delete-duplicates-warning = 所有选中的文件都将被永久删除且无法恢复（不经过回收站）。
modal-delete-duplicates-checkbox = 我理解文件将被永久删除且无法恢复。
modal-delete-duplicates-confirm = 🗑 是，永久删除选中项

modal-trash-duplicates-title = ♻ 将重复文件移至回收站
modal-trash-duplicates-header = ♻ 将重复文件移至回收站
modal-trash-duplicates-info = 可回收空间总计：{ $size }
modal-trash-duplicates-warning = 所有选中的文件都将被移至回收站。
modal-trash-duplicates-checkbox = 我确认要将这些文件移至回收站。
modal-trash-duplicates-confirm = ♻ 是，将选中项移至回收站

modal-hardlink-duplicates-title = 🔗 用硬链接替换重复文件
modal-hardlink-duplicates-header = 🔗 用硬链接替换重复文件
modal-hardlink-duplicates-info = 待处理文件总数：{ $count }。累计虚拟大小：{ $size }
modal-hardlink-duplicates-warning = 这将删除选中的重复文件，并将其替换为指向每组中保留的原始文件的文件系统级硬链接。文件在视觉上保持不变，同时释放实际的物理存储空间。
modal-hardlink-duplicates-checkbox = 我确认要将选中的文件替换为硬链接。
modal-hardlink-duplicates-confirm = 🔗 是，替换为硬链接

modal-softlink-duplicates-title = 🔗 用软链接替换重复文件
modal-softlink-duplicates-header = 🔗 用软链接替换重复文件
modal-softlink-duplicates-info = 待处理文件总数：{ $count }。累计虚拟大小：{ $size }
modal-softlink-duplicates-warning = 这将删除选中的重复文件，并将其替换为指向每组中保留的原始文件的文件系统级软链接（符号链接）。文件在视觉上保持不变，同时释放实际的物理存储空间。
modal-softlink-duplicates-checkbox = 我确认要将选中的文件替换为软链接。
modal-softlink-duplicates-confirm = 🔗 是，替换为软链接

# Path Does Not Exist Modal
modal-path-not-exist-title = ❌ 路径不存在！
modal-path-not-exist-msg = 错误：你尝试删除的路径在磁盘上不存在。
modal-close-btn = 关闭
modal-details-label = 详细信息： 
modal-cancel-btn = 取消

# Elevation Recommended Modal
modal-elevation-title = ⚠ 建议提升权限
modal-elevation-desc = eDirStat 默认以标准用户权限运行。但是，Windows 严格限制只有管理员账户才能访问原始物理磁盘句柄。
modal-elevation-mft-disabled = Windows NTFS MFT 驱动程序已禁用
modal-elevation-mft-desc = 没有管理员权限，直接访问磁盘的 MFT 扫描器将无法初始化。文件分析将改用备用的标准遍历驱动程序，扫描性能最多可能降低 20 倍。
modal-elevation-relaunch-prompt = 是否现在以管理员权限重新启动应用程序？
modal-elevation-continue-std = 以标准用户身份继续
modal-elevation-relaunch-btn = 🛡 以管理员身份重新启动

# About Modal
modal-about-title = ℹ 关于 eDirStat
modal-about-author = By: Cody Wyatt Neiman (xangelix) <neiman@cody.to>
modal-about-desc1 = 一款使用 Rust 构建的高性能磁盘空间分析与去重工具包。
modal-about-desc2 = 具备并行工作窃取式目录遍历、采用零解析布局反序列化的压缩快照，以及响应迅速的交互式矩形树图。
modal-about-desc3 = 内置的去重器运行多阶段加密哈希流水线，可安全地隔离重复文件组、计算可回收空间，并正确处理系统级硬链接。
modal-about-licenses-btn = 查看开源许可证
modal-about-version = v{ $version }

# How Deduplication Works Modal
modal-how-dedup-title = ℹ 去重工作原理
modal-how-dedup-desc1 = 本系统并不直接逐字节比较每个文件（那样需要缓慢的两两 O(N²) 扫描），而是采用高度优化的 7 阶段流水线，安全高效地识别相同内容。
modal-how-dedup-pipeline-title = 7 阶段流水线：
modal-how-dedup-why-title = 为什么这样就足够了？
modal-how-dedup-why-desc1 = 这一多阶段筛选确保只有大小、前缀、中点、后缀以及分布式块采样全部相同的文件才会被完整读取。最终通过比较 256-bit BLAKE3 加密哈希，其安全性可与工业级安全传输协议相媲美，从而无需缓慢的两两逐字节比较。

# How Deduplication Works Steps
modal-how-dedup-step1-title = 1. 按大小分组
modal-how-dedup-step1-desc = 文件按其精确字节大小分组。大小唯一的文件会被立即排除，完全无需磁盘 I/O。
modal-how-dedup-step2-title = 2. 前缀哈希
modal-how-dedup-step2-desc = 对剩余候选文件的前 4KB 进行哈希。这可以快速筛除头部或元数据格式不同的文件。
modal-how-dedup-step3-title = 3. 中点哈希
modal-how-dedup-step3-desc = 对剩余文件中部的一个 4KB 块进行哈希，以捕捉内部结构差异。
modal-how-dedup-step4-title = 4. 后缀哈希
modal-how-dedup-step4-desc = 对数据的最后 4KB 进行哈希。这对于识别尾部内容或元数据的差异非常有效。
modal-how-dedup-step5-title = 5. 多区段哈希
modal-how-dedup-step5-desc = 对大文件（超过 100MB）在其整个长度上进行周期性块采样，无需读取整个文件即可验证内容一致性。
modal-how-dedup-step6-title = 6. 完整 BLAKE3 哈希
modal-how-dedup-step6-desc = 对剩余候选文件计算完整的 BLAKE3 加密哈希。由于 256-bit 空间具有极高的抗碰撞性，哈希匹配意味着文件不同的概率微乎其微，无需两两比较即可提供高度可靠的一致性证明。
modal-how-dedup-step7-title = 7. 时间戳验证
modal-how-dedup-step7-desc = 在显示或执行任何去重操作之前，应用程序会验证文件在磁盘上的时间戳，以防自快照生成以来发生的更改。

# Open Source Licenses Modal
modal-licenses-title = 📜 开源许可证
modal-licenses-desc = 本应用程序使用了以下第三方库和 crate：

# Processing Modal
modal-processing-title = ⏳ 正在处理…
modal-processing-deletion = 正在删除文件和目录…
modal-processing-trash = 正在将文件和目录移至回收站…
modal-processing-hardlink = 正在用硬链接替换重复文件…
modal-processing-softlink = 正在用软链接替换重复文件…

# Explorer Column Headers
explorer-hdr-name = 名称
explorer-hdr-percentage = 百分比
explorer-hdr-size = 大小
explorer-hdr-items = 项目
explorer-hdr-files = 文件
explorer-hdr-subdirs = 子目录
explorer-hdr-created = 创建时间
explorer-hdr-modified = 修改时间

# Update Checker
update-checking = 正在检查更新…
update-available = 新版本 { $version } 可用！
update-up-to-date = 已是最新版本
update-failed = 更新检查失败：{ $error }

# Themes
theme = 🎨 主题
theme-dark = 深色
theme-high-contrast = 高对比度
theme-light = 浅色
theme-system = 跟随系统

# New Scan Options Modal
modal-scan-options-title = 新建扫描选项
modal-scan-options-header = 开始新的扫描
modal-scan-options-path-label = 要扫描的目录路径：
modal-scan-options-paste-tooltip = 从剪贴板粘贴
modal-scan-options-browse-tooltip = 浏览文件夹…
modal-scan-options-scan-btn = 扫描
modal-scan-options-cancel-btn = 取消
modal-scan-options-same-filesystem = 将扫描限制在同一文件系统/卷内
modal-scan-options-drives-header = 💽 存储驱动器和卷
modal-scan-options-refresh-tooltip = 刷新存储驱动器
modal-scan-options-root-system = 根文件系统
modal-scan-options-selected-badge = ✅ 已选中
modal-scan-options-free-of = { $free } 可用，共 { $total }
modal-scan-options-subtitle = 选择要分析的存储卷、快捷位置或自定义目录。
modal-scan-options-quick-access = 📍 快捷访问
modal-scan-options-path-hint = /要扫描的路径
modal-scan-options-hint = ℹ 请在上方选择驱动器或输入目录路径。
modal-scan-options-sandbox-auth = 🔒 需要沙盒访问权限 — 点击“扫描”以授予访问权限
modal-scan-options-valid-dir = ✅ 有效目录 — 随时可扫描
modal-scan-options-points-to-file = ⚠ 路径指向一个文件 — 请选择文件夹。
modal-scan-options-dir-not-exist = ⚠ 目录在文件系统中不存在。
quick-loc-home = 🏠 用户目录
quick-loc-documents = 📄 文档
quick-loc-downloads = 📥 下载
quick-loc-desktop = 🖥 桌面
quick-loc-pictures = 🖼 图片
search-use-regex = 使用正则表达式 (Regex)
search-match-case = 区分大小写
dedup-pref-dir-hint = 例如 /home/user/Archive
