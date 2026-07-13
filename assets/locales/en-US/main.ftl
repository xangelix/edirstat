# Menu Bar Dropdowns
file = File
view = View
help = Help

# Menu Bar Actions
new-scan = 📁 New Scan
save-snapshot = 💾 Save Snapshot
load-snapshot = 📖 Load Snapshot

# Menu Bar Status
idle = Idle

# View Menu Options
monospace-paths = Monospace Paths
highlight-duplicates = ✨ Highlight Duplicates
treemap-borders = 🔳 Treemap Borders
deletion-confirmation = 🗑 Deletion Confirmation
trash-confirmation = ♻ Trash Confirmation
time-format = 🕒 Time Format
language = 💬 Language
layout-mode = Layout Mode:
classic-layout = Classic Layout
windirstat-layout = WinDirStat Layout
vis-mode-treemap = 📊 Treemap
vis-mode-plots = 📈 Plots
select-plot-label = Select Plot:
vis-mode-deduplicator = 👥 Duplicate File Finder
search-filter-label = 🔍 Filter:

# Panel Toggles
toggle-left-panel = { $collapsed ->
    [true] ▶ Show Left Panel (F9)
   *[false] ◀ Hide Left Panel (F9)
}

toggle-right-panel = { $collapsed ->
    [true] { $is_classic ->
        [true] ◀ Show Right Panel (F11)
       *[false] ▶ Show Extensions Panel (F11)
    }
   *[false] { $is_classic ->
        [true] ▶ Hide Right Panel (F11)
       *[false] ◀ Hide Extensions Panel (F11)
    }
}

collapse-all = ⏏ Collapse All
about = ℹ About

# Status Indicators
scanning-disk = Scanning Disk...
scan-complete = Scan Complete
path-label = Path: { $path }
worker-threads = ⚡ { $count } Worker Threads
worker-threads-hover = The number of parallel, work-stealing CPU cores allocated for directory traversal.

# Stats Panel (Bottom)
directories-count = 📁 Directories: { $count }
files-count = 📄 Files: { $count }
total-size = 💾 Total Size: { $size }
elapsed-time = ⏱ Time: { $time }
scan-speed = ⚡ Speed: { $speed }/s

# Selection Info
selection-path = Selection: { $path }
selection-items = Selection: { $count ->
    [one] 1 item
   *[other] { $count } items
}

# Plot Types
plot-size-distribution = 📊 File Size Distribution
plot-age-size = 🌌 File Age vs. File Size
plot-dir-composition = 🍰 Directory Composition
plot-extension-boxplot = 📦 File Sizes by Extension
plot-temporal-timeline = ⏱ Linked Temporal Timelines
plot-deduplicator-waste = 👥 Duplicate Waste by Extension

# --- Deduplicator Strings ---
dedup-desc = Find and safely remove byte-for-byte identical files using cryptographically secure BLAKE3 hashes.
dedup-how-it-works = ℹ How it Works
dedup-min-size = Min File Size:
dedup-ignore-system = Ignore System Files
dedup-ignore-hidden = Ignore Hidden Files
dedup-start-scan = ⚡ Start Deduplication Scan
dedup-scan-first = Please scan a directory first.
dedup-cancelled-msg = Scan was cancelled. Start a new scan to find duplicates.
dedup-analyzing = Analyzing files...
dedup-no-duplicates = No duplicate groups found. Try reducing the Minimum File Size or scanning a different folder.
no-permission = No Permission
hardlink-badge = Hardlink
dedup-select-items = 🎯 Select items...
dedup-select-all-but-oldest = 🎯 All But Oldest
dedup-select-all-but-newest = 🎯 All But Newest
dedup-select-all-but-shortest = 🎯 All But Shortest Path
dedup-select-all-but-rootmost = 🎯 All But Root-most
dedup-select-all-but-longest = 🎯 All But Longest Path
dedup-pref-dir-pattern = Preferred Directory Pattern:
dedup-select-all-but-pref = 🎯 All But Preferred Directory
dedup-clear-selection = ❌ Clear Selection
dedup-link-menu = 🔗 Link... ({ $count } files)
dedup-link-menu-disabled = 🔗 Link... (0 files)
dedup-link-hardlinks = 🔗 Replace Selected with Hardlinks
dedup-link-softlinks = 🔗 Replace Selected with Softlinks
dedup-remove-menu = 🗑 Remove... ({ $count } files, { $size })
dedup-remove-menu-disabled = 🗑 Remove... (0 files)
dedup-remove-trash = ♻ Move Selected to Trash
dedup-remove-delete = 🗑 Delete Selected Permanently
dedup-warning-title = ⚠ DATA LOSS WARNING
dedup-warning-desc = { $count ->
    [one] Deleting all versions of 1 file
   *[other] Deleting all versions of { $count } files
}
dedup-warning-no-original = No Original Copy Will Remain:
dedup-warning-details = You have checked both the original and all duplicate copies for the files listed below. Deleting them will likely result in permanent data loss:
dedup-cancel-hover = Click to Cancel Scan
dedup-current-label = Current
dedup-phase1-size = Phase 1/7: Grouping all scanned files by size...
dedup-phase1-filter = Phase 1/7: Filtering exclusions on duplicate candidates...
dedup-phase2-prefix = Phase 2/7: Hashing file prefixes (first 4KB)...
dedup-phase3-midpoint = Phase 3/7: Hashing file midpoints...
dedup-phase4-suffix = Phase 4/7: Hashing file suffixes...
dedup-phase5-multirange = Phase 5/7: Multi-range hashing large files...
dedup-phase6-full = Phase 6/7: Full BLAKE3 hashing of remaining candidates...
dedup-phase7-validation = Phase 7/7: Final timestamp validation...
dedup-phase-finished = Finished in { $duration }! Found { $count } duplicate groups. Potential reclaimable space: { $space }
dedup-scan-cancelled-with-error = Scan was cancelled: { $error }

# Deduplicator Table Headers
dedup-hdr-checkbox = [     ]
dedup-hdr-filename = Filename
dedup-hdr-directory = Parent Directory
dedup-hdr-size = Size
dedup-hdr-reclaimable = Reclaimable
dedup-hdr-created = Created
dedup-hdr-modified = Modified
dedup-copies-selected = ({ $count ->
    [one] 1 copy selected
   *[other] { $count } copies selected
})

# --- Explorer Details Panel ---
explorer-details-header = ℹ Details
explorer-deselect-hover = Deselect items
explorer-deselect-single-hover = Deselect item
explorer-selected-items-count = { $count ->
    [one] 1 Selected Item
   *[other] { $count } Selected Items
}
explorer-total-size = Total Size: { $size }
explorer-files = Files: { $count }
explorer-directories = Directories: { $count }
explorer-actions-title = Actions
explorer-actions-operations = Operations:
explorer-action-refresh-hover = Refresh all selected directory subtrees
explorer-grid-type = Type:
explorer-grid-size = Size:
explorer-grid-bytes = Bytes:
explorer-grid-items = Items:
explorer-grid-files = Files:
explorer-grid-subdirs = Subdirs:
explorer-grid-user = User:
explorer-grid-group = Group:
explorer-grid-permissions = Permissions:
explorer-grid-path = Full Path:

# Explorer Type Names
type-symlink = Symbolic Link
type-directory = Directory
type-file = File

# Explorer Actions
explorer-action-copy-path = 📋 Copy Path
explorer-action-open-manager = 🗁 Open Manager
explorer-action-refresh-subtree = 🔄 Refresh Subtree
explorer-action-move-trash = ♻ Move to Trash
explorer-action-delete-permanently = 🗑 Delete Permanently
explorer-action-refresh-directory = 🔄 Refresh Directory

# Explorer Empty State
explorer-empty-state = Click 'New Scan' to explore disk usage.
placeholder-treemap = Scanned filesystem will be visualized as a treemap here.
placeholder-plots = Scanned filesystem will be plotted here.

# --- Extensions Panel ---
extensions-header = 📂 Extensions
extensions-empty = No statistics gathered yet.
extensions-hover-files = Files: { $count }

# --- Operations (Context Actions) ---
op-up-one-level = Up One Level
op-refresh-entire-scan = Refresh Entire Scan
op-refresh-directory = Refresh Directory
op-open-file-manager = Open in File Manager
op-open-terminal = Open Terminal Here
op-copy-path = Copy Path
op-copy-name = Copy Name
op-move-trash = Move to Trash
op-permanently-delete = Permanently Delete

# Toast Notifications
toast-already-root = Already at the root level
toast-navigated-up = Navigated up one level
toast-refreshing-scan = Refreshing entire scan...
toast-refreshing-dir = Refreshing selected directory/directories...
toast-opened-manager = Opened in file manager: { $path }
toast-failed-open-manager = Failed to open in file manager: { $error }
toast-opened-terminal = Opened terminal at: { $path }
toast-failed-open-terminal = Failed to open terminal: { $error }
toast-copied-paths = Copied { $count ->
    [one] 1 path to clipboard
   *[other] { $count } paths to clipboard
}
toast-copied-names = Copied { $count ->
    [one] 1 name to clipboard
   *[other] { $count } names to clipboard
}

# --- Modals ---
modal-remember-confirmation = Remember confirmation for all future files and directories
modal-process-multiple = You are about to process { $count } duplicate files/items:
modal-process-single = You are about to process the following path:
# Confirm Deletion/Trash/Link Modals
modal-delete-title = ⚠ PERMANENT DELETION WARNING
modal-delete-header = ⚠ Permanent Deletion Warning!
modal-delete-info = Total Size: { $size }
modal-delete-warning = This is a recursive deletion. All files, folders, and subdirectories under the selected path(s) will be permanently deleted and cannot be recovered (bypassing the recycle/trash bin).
modal-delete-checkbox = I understand that files will be permanently deleted and cannot be recovered.
modal-delete-confirm = 🗑 Yes, Delete Permanently

modal-trash-title = ♻ MOVE TO TRASH
modal-trash-header = ♻ Move to Trash
modal-trash-info = Total Size: { $size }
modal-trash-warning = This will move the selected path(s) and all their contents to your system recycle bin/trash, where they can be recovered or permanently deleted later.
modal-trash-checkbox = I confirm that I want to move this to the trash.
modal-trash-confirm = ♻ Yes, Move to Trash

modal-delete-duplicates-title = ⚠ PERMANENT DEDUPLICATION WARNING
modal-delete-duplicates-header = ⚠ Permanent Duplicate Deletion Warning!
modal-delete-duplicates-info = Total space to be reclaimed: { $size }
modal-delete-duplicates-warning = All selected files will be permanently deleted and cannot be recovered (bypassing the recycle/trash bin).
modal-delete-duplicates-checkbox = I understand that files will be permanently deleted and cannot be recovered.
modal-delete-duplicates-confirm = 🗑 Yes, Delete Selected Permanently

modal-trash-duplicates-title = ♻ MOVE DUPLICATES TO TRASH
modal-trash-duplicates-header = ♻ Move Duplicates to Trash
modal-trash-duplicates-info = Total space to be reclaimed: { $size }
modal-trash-duplicates-warning = All selected files will be moved to the recycle bin/trash.
modal-trash-duplicates-checkbox = I confirm that I want to move these files to the trash.
modal-trash-duplicates-confirm = ♻ Yes, Move Selected to Trash

modal-hardlink-duplicates-title = 🔗 REPLACE DUPLICATES WITH HARDLINKS
modal-hardlink-duplicates-header = 🔗 Replace Duplicates with Hardlinks
modal-hardlink-duplicates-info = Total files to process: { $count }. Cumulative virtual size: { $size }
modal-hardlink-duplicates-warning = This will delete the selected duplicate files and replace them with filesystem-level hardlinks pointing to the remaining original file in each group. This retains files visually while freeing up actual physical storage.
modal-hardlink-duplicates-checkbox = I confirm that I want to replace selected files with hardlinks.
modal-hardlink-duplicates-confirm = 🔗 Yes, Replace with Hardlinks

modal-softlink-duplicates-title = 🔗 REPLACE DUPLICATES WITH SOFTLINKS
modal-softlink-duplicates-header = 🔗 Replace Duplicates with Softlinks
modal-softlink-duplicates-info = Total files to process: { $count }. Cumulative virtual size: { $size }
modal-softlink-duplicates-warning = This will delete the selected duplicate files and replace them with filesystem-level softlinks (symbolic links) pointing to the remaining original file in each group. This retains files visually while freeing up actual physical storage.
modal-softlink-duplicates-checkbox = I confirm that I want to replace selected files with softlinks.
modal-softlink-duplicates-confirm = 🔗 Yes, Replace with Softlinks

# Path Does Not Exist Modal
modal-path-not-exist-title = ❌ Path Does Not Exist!
modal-path-not-exist-msg = Error: The path you are trying to delete does not exist on disk.
modal-close-btn = Close
modal-details-label = Details: 
modal-cancel-btn = Cancel

# Elevation Recommended Modal
modal-elevation-title = ⚠ Elevation Recommended
modal-elevation-desc = eDirStat runs with standard user privileges by default. However, Windows strictly restricts raw physical disk handle access to administrator accounts.
modal-elevation-mft-disabled = Windows NTFS MFT Driver Disabled
modal-elevation-mft-desc = Without administrative privileges, the direct-to-disk MFT scanner cannot initialize. File analysis will use the fallback standard traversal driver, reducing scan performance by as much as 20x.
modal-elevation-relaunch-prompt = Would you like to relaunch the application with Administrator privileges now?
modal-elevation-continue-std = Continue as Standard User
modal-elevation-relaunch-btn = 🛡 Relaunch as Admin

# About Modal
modal-about-title = ℹ About eDirStat
modal-about-author = By: Cody Wyatt Neiman (xangelix) <neiman@cody.to>
modal-about-desc1 = A high-performance disk space analyzer and deduplication toolkit built in Rust.
modal-about-desc2 = Features parallel, work-stealing directory traversal, compressed snapshots with zero-parsing layout deserialization, and responsive, interactive treemaps.
modal-about-desc3 = The integrated deduplicator runs a multi-stage cryptographic hashing pipeline to safely isolate duplicate groups, calculate reclaimable space, and respect system-level hardlinks.
modal-about-licenses-btn = View Open Source Licenses
modal-about-version = v{ $version }

# How Deduplication Works Modal
modal-how-dedup-title = ℹ How Deduplication Works
modal-how-dedup-desc1 = Rather than comparing every file's bytes directly (which requires slow, pairwise O(N²) scans), this system utilizes a highly optimized 7-stage pipeline to identify identical content safely and efficiently.
modal-how-dedup-pipeline-title = The 7-Stage Pipeline:
modal-how-dedup-why-title = Why is this sufficient?
modal-how-dedup-why-desc1 = This multi-stage filter ensures that only files with identical size, prefix, midpoint, suffix, and distributed block samples are read in full. Finally, comparing a 256-bit BLAKE3 cryptographic hash offers a safety profile on par with industry-grade secure transfer protocols, eliminating the need for slow, pairwise byte-by-byte comparisons.

# How Deduplication Works Steps
modal-how-dedup-step1-title = 1. Size Partitioning
modal-how-dedup-step1-desc = Files are grouped by their exact size in bytes. Any file with a unique size is discarded immediately, bypassing disk I/O entirely.
modal-how-dedup-step2-title = 2. Prefix Hashing
modal-how-dedup-step2-desc = The first 4KB of remaining candidates are hashed. This quickly filters out files with different headers or metadata formats.
modal-how-dedup-step3-title = 3. Midpoint Hashing
modal-how-dedup-step3-desc = A 4KB block from the center of the remaining files is hashed, catching internal structural differences.
modal-how-dedup-step4-title = 4. Suffix Hashing
modal-how-dedup-step4-desc = The last 4KB of data is hashed. This is highly effective at identifying differences in trailing contents or metadata.
modal-how-dedup-step5-title = 5. Multi-Range Hashing
modal-how-dedup-step5-desc = Large files (over 100MB) undergo periodic block sampling across their entire length to verify content consistency without reading the entire file.
modal-how-dedup-step6-title = 6. Full BLAKE3 Hashing
modal-how-dedup-step6-desc = For remaining candidates, a full BLAKE3 cryptographic hash is computed. Due to the high collision resistance of a 256-bit space, matching hashes indicate an astronomical unlikeliness that the files differ, providing a highly reliable proof of identity without requiring pairwise comparisons.
modal-how-dedup-step7-title = 7. Timestamp Validation
modal-how-dedup-step7-desc = Right before displaying or executing any deduplication action, the application verifies the files' timestamps on disk to protect against changes that occurred since snapshot generation.

# Open Source Licenses Modal
modal-licenses-title = 📜 Open Source Licenses
modal-licenses-desc = The following third-party libraries and crates are used in this application:

# Processing Modal
modal-processing-title = ⏳ Processing...
modal-processing-deletion = Deleting files and directories...
modal-processing-trash = Moving files and directories to trash...
modal-processing-hardlink = Replacing duplicates with hardlinks...
modal-processing-softlink = Replacing duplicates with softlinks...

# Explorer Column Headers
explorer-hdr-name = Name
explorer-hdr-percentage = Percentage
explorer-hdr-size = Size
explorer-hdr-items = Items
explorer-hdr-files = Files
explorer-hdr-subdirs = Subdirs
explorer-hdr-created = Created
explorer-hdr-modified = Modified

# Update Checker
update-checking = Checking for updates...
update-available = New version { $version } available!
update-up-to-date = You are up to date
update-failed = Update check failed: { $error }

# Themes
theme = 🎨 Theme
theme-dark = Dark
theme-high-contrast = High Contrast
theme-light = Light
theme-system = System

# New Scan Options Modal
modal-scan-options-title = New Scan Options
modal-scan-options-header = Start a New Scan
modal-scan-options-path-label = Directory path to scan:
modal-scan-options-paste-tooltip = Paste from clipboard
modal-scan-options-browse-tooltip = Browse folder...
modal-scan-options-scan-btn = Scan
modal-scan-options-cancel-btn = Cancel
modal-scan-options-same-filesystem = Limit scan to the same filesystem/volume
