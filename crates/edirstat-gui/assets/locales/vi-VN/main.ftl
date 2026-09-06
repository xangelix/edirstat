# Menu Bar Dropdowns
file = Tệp
view = Xem
help = Trợ giúp

# Menu Bar Actions
new-scan = 📁 Quét mới
save-snapshot = 💾 Lưu bản chụp nhanh
load-snapshot = 📖 Mở bản chụp nhanh

# Menu Bar Status
idle = Rảnh

# View Menu Options
monospace-paths = 🅰 Đường dẫn đơn cách
highlight-duplicates = ✨ Tô sáng tệp trùng lặp
treemap-borders = 🔳 Viền treemap
treemap-style =  Kiểu treemap
treemap-style-vertical = Chuyển màu dọc
treemap-style-offset-vertical = Chuyển màu dọc lệch
treemap-style-diagonal = Chuyển màu chéo
treemap-style-cushion = Tô bóng đệm
deletion-confirmation = 🗑 Xác nhận khi xóa
trash-confirmation = ♻ Xác nhận khi chuyển vào Thùng Rác
time-format = 🕒 Định dạng thời gian
language = 💬 Ngôn ngữ
layout-mode = Chế độ bố cục:
classic-layout = Bố cục cổ điển
windirstat-layout = Bố cục WinDirStat
vis-mode-treemap = 📊 Treemap
vis-mode-plots = 📈 Biểu đồ
select-plot-label = Chọn biểu đồ:
vis-mode-deduplicator = 👥 Trình tìm tệp trùng lặp
search-filter-label = 🔍 Lọc:

# Panel Toggles
toggle-left-panel = { $collapsed ->
    [true] ▶ Hiện bảng bên trái (F9)
   *[false] ◀ Ẩn bảng bên trái (F9)
}

toggle-right-panel = { $collapsed ->
    [true] { $is_classic ->
        [true] ◀ Hiện bảng bên phải (F11)
       *[false] ▶ Hiện bảng phần mở rộng (F11)
    }
   *[false] { $is_classic ->
        [true] ▶ Ẩn bảng bên phải (F11)
       *[false] ◀ Ẩn bảng phần mở rộng (F11)
    }
}

collapse-all = ⏏ Thu gọn tất cả
about = ℹ Giới thiệu
web-not-available = Tính năng không khả dụng trong phiên bản web

# Status Indicators
scanning-disk = Đang quét đĩa...
scan-complete = Quét hoàn tất
scan-cancelled = Đã hủy quét
path-label = Đường dẫn: { $path }
worker-threads = ⚡ { $count } luồng làm việc
worker-threads-hover = Số nhân CPU song song, theo cơ chế work-stealing, được cấp phát cho việc duyệt thư mục.

# Stats Panel (Bottom)
directories-count = 📁 Thư mục: { $count }
files-count = 📄 Tệp: { $count }
total-size = 💾 Tổng dung lượng: { $size }
elapsed-time = ⏱ Thời gian: { $time }
scan-speed = ⚡ Tốc độ: { $speed }/s

# Selection Info
selection-path = Lựa chọn: { $path }
selection-items = Lựa chọn: { $count } mục

# Plot Types
plot-size-distribution = 📊 Phân bố kích thước tệp
plot-age-size = 🌌 Tuổi tệp so với kích thước tệp
plot-dir-composition = 🍰 Thành phần thư mục
plot-extension-boxplot = 📦 Kích thước tệp theo phần mở rộng
plot-temporal-timeline = ⏱ Dòng thời gian liên kết
plot-deduplicator-waste = 👥 Dung lượng lãng phí do trùng lặp theo phần mở rộng

# --- Deduplicator Strings ---
dedup-desc = Tìm và xóa an toàn các tệp giống hệt nhau từng byte bằng hàm băm BLAKE3 an toàn về mặt mật mã.
dedup-how-it-works = ℹ Cách hoạt động
dedup-min-size = Kích thước tệp tối thiểu:
dedup-ignore-system = Bỏ qua tệp hệ thống
dedup-ignore-hidden = Bỏ qua tệp ẩn
dedup-start-scan = ⚡ Bắt đầu quét loại bỏ trùng lặp
dedup-scan-first = Vui lòng quét một thư mục trước.
dedup-cancelled-msg = Quá trình quét đã bị hủy. Hãy bắt đầu một lượt quét mới để tìm tệp trùng lặp.
dedup-analyzing = Đang phân tích tệp...
dedup-no-duplicates = Không tìm thấy nhóm trùng lặp nào. Hãy thử giảm Kích thước tệp tối thiểu hoặc quét một thư mục khác.
no-permission = Không có quyền
hardlink-badge = Liên kết cứng
dedup-select-items = 🎯 Chọn mục...
dedup-select-all-but-oldest = 🎯 Tất cả trừ cũ nhất
dedup-select-all-but-newest = 🎯 Tất cả trừ mới nhất
dedup-select-all-but-shortest = 🎯 Tất cả trừ đường dẫn ngắn nhất
dedup-select-all-but-rootmost = 🎯 Tất cả trừ gần gốc nhất
dedup-select-all-but-longest = 🎯 Tất cả trừ đường dẫn dài nhất
dedup-pref-dir-pattern = Mẫu thư mục ưu tiên:
dedup-select-all-but-pref = 🎯 Tất cả trừ thư mục ưu tiên
dedup-clear-selection = ❌ Bỏ chọn
dedup-link-menu = 🔗 Liên kết... ({ $count } tệp)
dedup-link-menu-disabled = 🔗 Liên kết... (0 tệp)
dedup-link-hardlinks = 🔗 Thay thế mục đã chọn bằng liên kết cứng
dedup-link-softlinks = 🔗 Thay thế mục đã chọn bằng liên kết mềm
dedup-remove-menu = 🗑 Xóa... ({ $count } tệp, { $size })
dedup-remove-menu-disabled = 🗑 Xóa... (0 tệp)
dedup-remove-trash = ♻ Chuyển mục đã chọn vào Thùng Rác
dedup-remove-delete = 🗑 Xóa vĩnh viễn mục đã chọn
dedup-warning-title = ⚠ CẢNH BÁO MẤT DỮ LIỆU
dedup-warning-desc = Xóa tất cả các phiên bản của { $count } tệp
dedup-warning-no-original = Sẽ không còn lại bản gốc nào:
dedup-warning-details = Bạn đã đánh dấu cả bản gốc lẫn tất cả các bản sao trùng lặp của các tệp liệt kê bên dưới. Việc xóa chúng có thể dẫn đến mất dữ liệu vĩnh viễn:
dedup-cancel-hover = Nhấp để hủy quét
scan-cancel-hover = Nhấp để hủy quét
dedup-current-label = Hiện tại
dedup-phase1-size = Giai đoạn 1/7: Đang nhóm tất cả tệp đã quét theo kích thước...
dedup-phase1-filter = Giai đoạn 1/7: Đang lọc các loại trừ trên ứng viên trùng lặp...
dedup-phase2-prefix = Giai đoạn 2/7: Đang băm tiền tố tệp (4KB đầu tiên)...
dedup-phase3-midpoint = Giai đoạn 3/7: Đang băm điểm giữa tệp...
dedup-phase4-suffix = Giai đoạn 4/7: Đang băm hậu tố tệp...
dedup-phase5-multirange = Giai đoạn 5/7: Đang băm đa phạm vi các tệp lớn...
dedup-phase6-full = Giai đoạn 6/7: Đang băm BLAKE3 toàn bộ các ứng viên còn lại...
dedup-phase7-validation = Giai đoạn 7/7: Đang xác thực dấu thời gian lần cuối...
dedup-phase-finished = Hoàn tất trong { $duration }! Tìm thấy { $count } nhóm trùng lặp. Dung lượng có thể thu hồi: { $space }
dedup-scan-cancelled-with-error = Quá trình quét đã bị hủy: { $error }

# Deduplicator Table Headers
dedup-hdr-checkbox = [     ]
dedup-hdr-filename = Tên tệp
dedup-hdr-directory = Thư mục cha
dedup-hdr-size = Kích thước
dedup-hdr-reclaimable = Có thể thu hồi
dedup-hdr-created = Ngày tạo
dedup-hdr-modified = Ngày sửa đổi
dedup-copies-selected = (đã chọn { $count } bản sao)

# --- Explorer Details Panel ---
explorer-details-header = ℹ Chi tiết
explorer-deselect-hover = Bỏ chọn các mục
explorer-deselect-single-hover = Bỏ chọn mục
explorer-selected-items-count = { $count } mục đã chọn
explorer-total-size = Tổng dung lượng: { $size }
explorer-files = Tệp: { $count }
explorer-directories = Thư mục: { $count }
explorer-actions-title = Hành động
explorer-actions-operations = Thao tác:
explorer-action-refresh-hover = Làm mới tất cả các cây thư mục con đã chọn
explorer-grid-type = Loại:
explorer-grid-size = Kích thước:
explorer-grid-bytes = Byte:
explorer-grid-items = Mục:
explorer-grid-files = Tệp:
explorer-grid-subdirs = Thư mục con:
explorer-grid-user = Người dùng:
explorer-grid-group = Nhóm:
explorer-grid-permissions = Quyền:
explorer-grid-path = Đường dẫn đầy đủ:

# Explorer Type Names
type-symlink = Liên kết tượng trưng
type-directory = Thư mục
type-file = Tệp

# Explorer Actions
explorer-action-copy-path = 📋 Sao chép đường dẫn
explorer-action-open-file = 📄 Mở tệp
explorer-action-open-manager = 🗁 Mở trình quản lý tệp
explorer-action-refresh-subtree = 🔄 Làm mới cây thư mục con
explorer-action-move-trash = ♻ Chuyển vào Thùng Rác
explorer-action-delete-permanently = 🗑 Xóa vĩnh viễn
explorer-action-refresh-directory = 🔄 Làm mới thư mục

# Explorer Empty State
explorer-empty-state = Nhấp 'Quét mới' để khám phá dung lượng đĩa.
choose-an-option = Chọn một tùy chọn
web-viewer = Trình xem web
load-demo = 👁 Tải bản chụp nhanh minh họa mẫu
placeholder-treemap = Hệ thống tệp đã quét sẽ được trực quan hóa dưới dạng treemap tại đây.
placeholder-plots = Hệ thống tệp đã quét sẽ được vẽ biểu đồ tại đây.

# Treemap Zoom & Navigation
zoom-up = ⏶ Lên
zoom-reset = ❌ Đặt lại
zoom-to-dir = 🔍 Tập trung trong treemap
zoom-up-level = ⏶ Đi lên một cấp
zoom-empty-dir = Thư mục trống

# --- Extensions Panel ---
extensions-header = 📂 Phần mở rộng
extensions-empty = Chưa thu thập được thống kê nào.
extensions-hover-files = Tệp: { $count }

# --- Operations (Context Actions) ---
op-up-one-level = Lên một cấp
op-zoom-treemap = Tập trung trong treemap
op-refresh-entire-scan = Làm mới toàn bộ quá trình quét
op-refresh-directory = Làm mới thư mục
op-open-file = Mở tệp
op-open-file-manager = Mở trong trình quản lý tệp
op-open-terminal = Mở Terminal tại đây
op-copy-path = Sao chép đường dẫn
op-copy-name = Sao chép tên
op-move-trash = Chuyển vào Thùng Rác
op-permanently-delete = Xóa vĩnh viễn

# Toast Notifications
toast-already-root = Đã ở cấp gốc
toast-navigated-up = Đã đi lên một cấp
toast-zoomed-treemap = Đã tập trung treemap vào thư mục
toast-refreshing-scan = Đang làm mới toàn bộ quá trình quét...
toast-refreshing-dir = Đang làm mới thư mục đã chọn...
toast-opened-file = Đã mở: { $path }
toast-failed-open-file = Không thể mở tệp: { $error }
toast-opened-manager = Đã mở trong trình quản lý tệp: { $path }
toast-failed-open-manager = Không thể mở trong trình quản lý tệp: { $error }
toast-opened-terminal = Đã mở terminal tại: { $path }
toast-failed-open-terminal = Không thể mở terminal: { $error }
toast-copied-paths = Đã sao chép { $count } đường dẫn vào clipboard
toast-copied-names = Đã sao chép { $count } tên vào clipboard

# --- Modals ---
modal-remember-confirmation = Ghi nhớ xác nhận cho tất cả các tệp và thư mục trong tương lai
modal-process-multiple = Bạn sắp xử lý { $count } tệp/mục trùng lặp:
modal-process-single = Bạn sắp xử lý đường dẫn sau:
# Confirm Deletion/Trash/Link Modals
modal-delete-title = ⚠ CẢNH BÁO XÓA VĨNH VIỄN
modal-delete-header = ⚠ Cảnh báo xóa vĩnh viễn!
modal-delete-info = Tổng dung lượng: { $size }
modal-delete-warning = Đây là thao tác xóa đệ quy. Tất cả tệp, thư mục và thư mục con trong (các) đường dẫn đã chọn sẽ bị xóa vĩnh viễn và không thể khôi phục (bỏ qua Thùng Rác).
modal-delete-checkbox = Tôi hiểu rằng các tệp sẽ bị xóa vĩnh viễn và không thể khôi phục.
modal-delete-confirm = 🗑 Vâng, xóa vĩnh viễn

modal-trash-title = ♻ CHUYỂN VÀO THÙNG RÁC
modal-trash-header = ♻ Chuyển vào Thùng Rác
modal-trash-info = Tổng dung lượng: { $size }
modal-trash-warning = Thao tác này sẽ chuyển (các) đường dẫn đã chọn cùng toàn bộ nội dung của chúng vào Thùng Rác của hệ thống, nơi chúng có thể được khôi phục hoặc xóa vĩnh viễn sau.
modal-trash-checkbox = Tôi xác nhận muốn chuyển mục này vào Thùng Rác.
modal-trash-confirm = ♻ Vâng, chuyển vào Thùng Rác

modal-delete-duplicates-title = ⚠ CẢNH BÁO LOẠI BỎ TRÙNG LẶP VĨNH VIỄN
modal-delete-duplicates-header = ⚠ Cảnh báo xóa vĩnh viễn tệp trùng lặp!
modal-delete-duplicates-info = Tổng dung lượng sẽ thu hồi: { $size }
modal-delete-duplicates-warning = Tất cả tệp đã chọn sẽ bị xóa vĩnh viễn và không thể khôi phục (bỏ qua Thùng Rác).
modal-delete-duplicates-checkbox = Tôi hiểu rằng các tệp sẽ bị xóa vĩnh viễn và không thể khôi phục.
modal-delete-duplicates-confirm = 🗑 Vâng, xóa vĩnh viễn mục đã chọn

modal-trash-duplicates-title = ♻ CHUYỂN TỆP TRÙNG LẶP VÀO THÙNG RÁC
modal-trash-duplicates-header = ♻ Chuyển tệp trùng lặp vào Thùng Rác
modal-trash-duplicates-info = Tổng dung lượng sẽ thu hồi: { $size }
modal-trash-duplicates-warning = Tất cả tệp đã chọn sẽ được chuyển vào Thùng Rác.
modal-trash-duplicates-checkbox = Tôi xác nhận muốn chuyển các tệp này vào Thùng Rác.
modal-trash-duplicates-confirm = ♻ Vâng, chuyển mục đã chọn vào Thùng Rác

modal-hardlink-duplicates-title = 🔗 THAY THẾ TỆP TRÙNG LẶP BẰNG LIÊN KẾT CỨNG
modal-hardlink-duplicates-header = 🔗 Thay thế tệp trùng lặp bằng liên kết cứng
modal-hardlink-duplicates-info = Tổng số tệp cần xử lý: { $count }. Dung lượng ảo tích lũy: { $size }
modal-hardlink-duplicates-warning = Thao tác này sẽ xóa các tệp trùng lặp đã chọn và thay thế chúng bằng các liên kết cứng ở cấp hệ thống tệp trỏ đến tệp gốc còn lại trong mỗi nhóm. Điều này giữ nguyên các tệp về mặt hiển thị đồng thời giải phóng dung lượng lưu trữ vật lý thực tế.
modal-hardlink-duplicates-checkbox = Tôi xác nhận muốn thay thế các tệp đã chọn bằng liên kết cứng.
modal-hardlink-duplicates-confirm = 🔗 Vâng, thay bằng liên kết cứng

modal-softlink-duplicates-title = 🔗 THAY THẾ TỆP TRÙNG LẶP BẰNG LIÊN KẾT MỀM
modal-softlink-duplicates-header = 🔗 Thay thế tệp trùng lặp bằng liên kết mềm
modal-softlink-duplicates-info = Tổng số tệp cần xử lý: { $count }. Dung lượng ảo tích lũy: { $size }
modal-softlink-duplicates-warning = Thao tác này sẽ xóa các tệp trùng lặp đã chọn và thay thế chúng bằng các liên kết mềm (liên kết tượng trưng) ở cấp hệ thống tệp trỏ đến tệp gốc còn lại trong mỗi nhóm. Điều này giữ nguyên các tệp về mặt hiển thị đồng thời giải phóng dung lượng lưu trữ vật lý thực tế.
modal-softlink-duplicates-checkbox = Tôi xác nhận muốn thay thế các tệp đã chọn bằng liên kết mềm.
modal-softlink-duplicates-confirm = 🔗 Vâng, thay bằng liên kết mềm

# Path Does Not Exist Modal
modal-path-not-exist-title = ❌ Đường dẫn không tồn tại!
modal-path-not-exist-msg = Lỗi: Đường dẫn bạn đang cố xóa không tồn tại trên đĩa.
modal-close-btn = Đóng
modal-details-label = Chi tiết: 
modal-cancel-btn = Hủy

# Elevation Recommended Modal
modal-elevation-title = ⚠ Khuyến nghị nâng quyền
modal-elevation-desc = eDirStat chạy với quyền người dùng chuẩn theo mặc định. Tuy nhiên, Windows hạn chế nghiêm ngặt quyền truy cập handle đĩa vật lý thô chỉ dành cho tài khoản quản trị viên.
modal-elevation-mft-disabled = Trình điều khiển MFT NTFS của Windows đã bị tắt
modal-elevation-mft-desc = Nếu không có quyền quản trị, trình quét MFT truy cập đĩa trực tiếp không thể khởi tạo. Việc phân tích tệp sẽ sử dụng trình điều khiển duyệt thư mục chuẩn dự phòng, làm giảm hiệu suất quét tới 20 lần.
modal-elevation-relaunch-prompt = Bạn có muốn khởi chạy lại ứng dụng với quyền Quản trị viên ngay bây giờ không?
modal-elevation-continue-std = Tiếp tục với tư cách Người dùng chuẩn
modal-elevation-relaunch-btn = 🛡 Khởi chạy lại với quyền Quản trị

# About Modal
modal-about-title = ℹ Giới thiệu eDirStat
modal-about-author = Tác giả: Cody Wyatt Neiman (xangelix) <neiman@cody.to>
modal-about-desc1 = Công cụ phân tích dung lượng đĩa và loại bỏ trùng lặp hiệu năng cao được xây dựng bằng Rust.
modal-about-desc2 = Có khả năng duyệt thư mục song song theo cơ chế work-stealing, bản chụp nhanh nén với giải tuần tự hóa bố cục không cần phân tích cú pháp, và các treemap tương tác, phản hồi nhanh.
modal-about-desc3 = Trình loại bỏ trùng lặp tích hợp chạy một quy trình băm mật mã nhiều giai đoạn để tách biệt an toàn các nhóm trùng lặp, tính toán dung lượng có thể thu hồi và tôn trọng các liên kết cứng cấp hệ thống.
modal-about-licenses-btn = Xem giấy phép mã nguồn mở
modal-about-version = v{ $version }

# How Deduplication Works Modal
modal-how-dedup-title = ℹ Cách hoạt động của loại bỏ trùng lặp
modal-how-dedup-desc1 = Thay vì so sánh trực tiếp byte của mọi tệp (đòi hỏi các lượt quét từng cặp O(N²) chậm chạp), hệ thống này sử dụng một quy trình 7 giai đoạn được tối ưu hóa cao để xác định nội dung giống hệt nhau một cách an toàn và hiệu quả.
modal-how-dedup-pipeline-title = Quy trình 7 giai đoạn:
modal-how-dedup-why-title = Tại sao như vậy là đủ?
modal-how-dedup-why-desc1 = Bộ lọc nhiều giai đoạn này đảm bảo chỉ những tệp có kích thước, tiền tố, điểm giữa, hậu tố và các mẫu khối phân tán giống hệt nhau mới được đọc toàn bộ. Cuối cùng, việc so sánh hàm băm mật mã BLAKE3 256-bit mang lại mức độ an toàn ngang với các giao thức truyền tải bảo mật cấp công nghiệp, loại bỏ nhu cầu so sánh từng byte theo cặp chậm chạp.

# How Deduplication Works Steps
modal-how-dedup-step1-title = 1. Phân vùng theo kích thước
modal-how-dedup-step1-desc = Các tệp được nhóm theo kích thước chính xác tính bằng byte. Bất kỳ tệp nào có kích thước duy nhất sẽ bị loại bỏ ngay lập tức, bỏ qua hoàn toàn I/O đĩa.
modal-how-dedup-step2-title = 2. Băm tiền tố
modal-how-dedup-step2-desc = 4KB đầu tiên của các ứng viên còn lại được băm. Điều này nhanh chóng lọc ra các tệp có tiêu đề hoặc định dạng siêu dữ liệu khác nhau.
modal-how-dedup-step3-title = 3. Băm điểm giữa
modal-how-dedup-step3-desc = Một khối 4KB từ trung tâm của các tệp còn lại được băm, phát hiện các khác biệt cấu trúc bên trong.
modal-how-dedup-step4-title = 4. Băm hậu tố
modal-how-dedup-step4-desc = 4KB dữ liệu cuối cùng được băm. Điều này rất hiệu quả trong việc xác định khác biệt ở nội dung hoặc siêu dữ liệu cuối tệp.
modal-how-dedup-step5-title = 5. Băm đa phạm vi
modal-how-dedup-step5-desc = Các tệp lớn (trên 100MB) được lấy mẫu khối định kỳ trên toàn bộ chiều dài để xác minh tính nhất quán của nội dung mà không cần đọc toàn bộ tệp.
modal-how-dedup-step6-title = 6. Băm BLAKE3 toàn bộ
modal-how-dedup-step6-desc = Đối với các ứng viên còn lại, một hàm băm mật mã BLAKE3 toàn bộ được tính toán. Nhờ khả năng chống va chạm cao của không gian 256-bit, các hàm băm trùng khớp cho thấy khả năng hai tệp khác nhau là nhỏ đến mức phi thực tế, cung cấp bằng chứng nhận dạng rất đáng tin cậy mà không cần so sánh từng cặp.
modal-how-dedup-step7-title = 7. Xác thực dấu thời gian
modal-how-dedup-step7-desc = Ngay trước khi hiển thị hoặc thực thi bất kỳ hành động loại bỏ trùng lặp nào, ứng dụng xác minh dấu thời gian của các tệp trên đĩa để bảo vệ khỏi các thay đổi đã xảy ra kể từ khi tạo bản chụp nhanh.

# Open Source Licenses Modal
modal-licenses-title = 📜 Giấy phép mã nguồn mở
modal-licenses-desc = Các thư viện và crate bên thứ ba sau đây được sử dụng trong ứng dụng này:

# Processing Modal
modal-processing-title = ⏳ Đang xử lý...
modal-processing-deletion = Đang xóa tệp và thư mục...
modal-processing-trash = Đang chuyển tệp và thư mục vào Thùng Rác...
modal-processing-hardlink = Đang thay thế tệp trùng lặp bằng liên kết cứng...
modal-processing-softlink = Đang thay thế tệp trùng lặp bằng liên kết mềm...

# Explorer Column Headers
explorer-hdr-name = Tên
explorer-hdr-percentage = Phần trăm
explorer-hdr-size = Kích thước
explorer-hdr-items = Mục
explorer-hdr-files = Tệp
explorer-hdr-subdirs = Thư mục con
explorer-hdr-created = Ngày tạo
explorer-hdr-modified = Ngày sửa đổi

# Update Checker
update-checking = Đang kiểm tra cập nhật...
update-available = Đã có phiên bản mới { $version }!
update-up-to-date = Bạn đang dùng phiên bản mới nhất
update-failed = Kiểm tra cập nhật thất bại: { $error }

# Themes
theme = 🎨 Chủ đề
theme-dark = Tối
theme-high-contrast = Độ tương phản cao
theme-light = Sáng
theme-system = Hệ thống

# New Scan Options Modal
modal-scan-options-title = Tùy chọn quét mới
modal-scan-options-header = Bắt đầu một lượt quét mới
modal-scan-options-path-label = Đường dẫn thư mục cần quét:
modal-scan-options-paste-tooltip = Dán từ clipboard
modal-scan-options-browse-tooltip = Duyệt thư mục...
modal-scan-options-scan-btn = Quét
modal-scan-options-cancel-btn = Hủy
modal-scan-options-same-filesystem = Giới hạn quét trong cùng một hệ thống tệp/ổ đĩa
modal-scan-options-drives-header = 💽 Ổ đĩa & phân vùng lưu trữ
modal-scan-options-refresh-tooltip = Làm mới ổ đĩa lưu trữ
modal-scan-options-root-system = Hệ thống gốc
modal-scan-options-selected-badge = ✅ Đã chọn
modal-scan-options-free-of = { $free } trống trong tổng số { $total }
modal-scan-options-subtitle = Chọn một ổ lưu trữ, vị trí nhanh hoặc thư mục tùy chỉnh để phân tích.
modal-scan-options-quick-access = 📍 Lối tắt truy cập nhanh
modal-scan-options-path-hint = /đường/dẫn/cần/quét
modal-scan-options-hint = ℹ Chọn một ổ đĩa ở trên hoặc nhập đường dẫn thư mục.
modal-scan-options-sandbox-auth = 🔒 Cần cấp quyền truy cập Sandbox — Nhấp Quét để cấp quyền
modal-scan-options-valid-dir = ✅ Thư mục hợp lệ — Sẵn sàng quét
modal-scan-options-points-to-file = ⚠ Đường dẫn trỏ đến một tệp — vui lòng chọn một thư mục.
modal-scan-options-dir-not-exist = ⚠ Thư mục không tồn tại trên hệ thống tệp.
quick-loc-home = 🏠 Trang chủ
quick-loc-documents = 📄 Tài liệu
quick-loc-downloads = 📥 Tải xuống
quick-loc-desktop = 🖥 Màn hình nền
quick-loc-pictures = 🖼 Hình ảnh
search-use-regex = Sử dụng biểu thức chính quy (Regex)
search-match-case = Phân biệt chữ hoa chữ thường
dedup-pref-dir-hint = VD: /home/user/Archive

file-menu-close = Đóng quá trình quét
file-menu-quit = Thoát

badge-dataless-cloud = Tệp đám mây / không chứa dữ liệu
badge-symlink = Liên kết tượng trưng
badge-special-file = Tệp đặc biệt (Pipe / Socket / Thiết bị)
badge-permission-denied = Truy cập bị từ chối
