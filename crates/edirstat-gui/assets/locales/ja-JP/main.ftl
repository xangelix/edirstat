# Menu Bar Dropdowns
file = ファイル
view = 表示
help = ヘルプ

# Menu Bar Actions
new-scan = 📁 新規スキャン
save-snapshot = 💾 スナップショットを保存
load-snapshot = 📖 スナップショットを読み込み

# Menu Bar Status
idle = 待機中

# View Menu Options
monospace-paths = 🅰 等幅パス
highlight-duplicates = ✨ 重複を強調表示
treemap-borders = 🔳 ツリーマップの境界線
treemap-style =  ツリーマップのスタイル
treemap-style-vertical = 垂直グラデーション
treemap-style-offset-vertical = オフセット垂直グラデーション
treemap-style-diagonal = 斜めグラデーション
treemap-style-cushion = クッションシェーディング
deletion-confirmation = 🗑 削除の確認
trash-confirmation = ♻ ゴミ箱の確認
time-format = 🕒 時刻形式
language = 💬 言語
layout-mode = レイアウトモード：
classic-layout = クラシックレイアウト
windirstat-layout = WinDirStat レイアウト
vis-mode-treemap = 📊 ツリーマップ
vis-mode-plots = 📈 グラフ
select-plot-label = グラフを選択：
vis-mode-deduplicator = 👥 重複ファイル検索
search-filter-label = 🔍 フィルター：

# Panel Toggles
toggle-left-panel = { $collapsed ->
    [true] ▶ 左パネルを表示 (F9)
   *[false] ◀ 左パネルを隠す (F9)
}

toggle-right-panel = { $collapsed ->
    [true] { $is_classic ->
        [true] ◀ 右パネルを表示 (F11)
       *[false] ▶ 拡張子パネルを表示 (F11)
    }
   *[false] { $is_classic ->
        [true] ▶ 右パネルを隠す (F11)
       *[false] ◀ 拡張子パネルを隠す (F11)
    }
}

collapse-all = ⏏ すべて折りたたむ
about = ℹ バージョン情報
web-not-available = この機能は Web 版では利用できません

# Status Indicators
scanning-disk = ディスクをスキャンしています…
scan-complete = スキャンが完了しました
scan-cancelled = スキャンがキャンセルされました
path-label = パス： { $path }
worker-threads = ⚡ { $count } ワーカースレッド
worker-threads-hover = ディレクトリ走査に割り当てられる、ワークスティーリング方式の並列 CPU コアの数。

# Stats Panel (Bottom)
directories-count = 📁 ディレクトリ： { $count }
files-count = 📄 ファイル： { $count }
total-size = 💾 合計サイズ： { $size }
elapsed-time = ⏱ 時間： { $time }
scan-speed = ⚡ 速度： { $speed }/s

# Selection Info
selection-path = 選択： { $path }
selection-items = 選択： { $count ->
   *[other] { $count } 個の項目
}

# Plot Types
plot-size-distribution = 📊 ファイルサイズ分布
plot-age-size = 🌌 ファイルの経過期間とサイズ
plot-dir-composition = 🍰 ディレクトリ構成
plot-extension-boxplot = 📦 拡張子別ファイルサイズ
plot-temporal-timeline = ⏱ 連動する時系列タイムライン
plot-deduplicator-waste = 👥 拡張子別の重複による無駄な容量

# --- Deduplicator Strings ---
dedup-desc = 暗号学的に安全な BLAKE3 ハッシュを使用して、バイト単位で完全に同一のファイルを検出し、安全に削除します。
dedup-how-it-works = ℹ 仕組み
dedup-min-size = 最小ファイルサイズ：
dedup-ignore-system = システムファイルを無視
dedup-ignore-hidden = 隠しファイルを無視
dedup-start-scan = ⚡ 重複排除スキャンを開始
dedup-scan-first = 先にディレクトリをスキャンしてください。
dedup-cancelled-msg = スキャンはキャンセルされました。重複を検出するには新しいスキャンを開始してください。
dedup-analyzing = ファイルを解析しています…
dedup-no-duplicates = 重複グループは見つかりませんでした。最小ファイルサイズを小さくするか、別のフォルダをスキャンしてみてください。
no-permission = 権限がありません
hardlink-badge = ハードリンク
dedup-select-items = 🎯 項目を選択…
dedup-select-all-but-oldest = 🎯 最も古いもの以外すべて
dedup-select-all-but-newest = 🎯 最も新しいもの以外すべて
dedup-select-all-but-shortest = 🎯 パスが最も短いもの以外すべて
dedup-select-all-but-rootmost = 🎯 ルートに最も近いもの以外すべて
dedup-select-all-but-longest = 🎯 パスが最も長いもの以外すべて
dedup-pref-dir-pattern = 優先ディレクトリのパターン：
dedup-select-all-but-pref = 🎯 優先ディレクトリ以外すべて
dedup-clear-selection = ❌ 選択をクリア
dedup-link-menu = 🔗 リンク… ({ $count } 個のファイル)
dedup-link-menu-disabled = 🔗 リンク… (0 個のファイル)
dedup-link-hardlinks = 🔗 選択した項目をハードリンクに置き換え
dedup-link-softlinks = 🔗 選択した項目をソフトリンクに置き換え
dedup-remove-menu = 🗑 削除… ({ $count } 個のファイル, { $size })
dedup-remove-menu-disabled = 🗑 削除… (0 個のファイル)
dedup-remove-trash = ♻ 選択した項目をゴミ箱に移動
dedup-remove-delete = 🗑 選択した項目を完全に削除
dedup-warning-title = ⚠ データ損失の警告
dedup-warning-desc = { $count ->
   *[other] { $count } 個のファイルのすべてのバージョンを削除しようとしています
}
dedup-warning-no-original = 元のコピーは残りません：
dedup-warning-details = 以下に一覧表示されたファイルについて、元のファイルとすべての重複コピーの両方にチェックが付けられています。これらを削除すると、永久的なデータ損失が発生する可能性があります：
dedup-cancel-hover = クリックしてスキャンをキャンセル
scan-cancel-hover = クリックしてスキャンをキャンセル
dedup-current-label = 現在
dedup-phase1-size = フェーズ 1/7: スキャンしたすべてのファイルをサイズでグループ化しています…
dedup-phase1-filter = フェーズ 1/7: 重複候補に除外フィルターを適用しています…
dedup-phase2-prefix = フェーズ 2/7: ファイルのプレフィックス (先頭 4KB) をハッシュ化しています…
dedup-phase3-midpoint = フェーズ 3/7: ファイルの中間点をハッシュ化しています…
dedup-phase4-suffix = フェーズ 4/7: ファイルのサフィックスをハッシュ化しています…
dedup-phase5-multirange = フェーズ 5/7: 大きなファイルをマルチレンジでハッシュ化しています…
dedup-phase6-full = フェーズ 6/7: 残りの候補を BLAKE3 で完全にハッシュ化しています…
dedup-phase7-validation = フェーズ 7/7: 最終的なタイムスタンプ検証を実行しています…
dedup-phase-finished = { $duration } で完了しました！ { $count } 個の重複グループが見つかりました。回収可能な容量： { $space }
dedup-scan-cancelled-with-error = スキャンはキャンセルされました： { $error }

# Deduplicator Table Headers
dedup-hdr-checkbox = [     ]
dedup-hdr-filename = ファイル名
dedup-hdr-directory = 親ディレクトリ
dedup-hdr-size = サイズ
dedup-hdr-reclaimable = 回収可能
dedup-hdr-created = 作成日時
dedup-hdr-modified = 変更日時
dedup-copies-selected = ({ $count ->
   *[other] { $count } 個のコピーを選択中
})

# --- Explorer Details Panel ---
explorer-details-header = ℹ 詳細
explorer-deselect-hover = 項目の選択を解除
explorer-deselect-single-hover = この項目の選択を解除
explorer-selected-items-count = { $count ->
   *[other] { $count } 個の選択項目
}
explorer-total-size = 合計サイズ： { $size }
explorer-files = ファイル： { $count }
explorer-directories = ディレクトリ： { $count }
explorer-actions-title = 操作
explorer-actions-operations = 操作：
explorer-action-refresh-hover = 選択したすべてのディレクトリのサブツリーを更新
explorer-grid-type = 種類：
explorer-grid-size = サイズ：
explorer-grid-bytes = バイト数：
explorer-grid-items = 項目数：
explorer-grid-files = ファイル数：
explorer-grid-subdirs = サブディレクトリ数：
explorer-grid-user = ユーザー：
explorer-grid-group = グループ：
explorer-grid-permissions = アクセス許可：
explorer-grid-path = 完全なパス：

# Explorer Type Names
type-symlink = シンボリックリンク
type-directory = ディレクトリ
type-file = ファイル

# Explorer Actions
explorer-action-copy-path = 📋 パスをコピー
explorer-action-open-file = 📄 ファイルを開く
explorer-action-open-manager = 🗁 マネージャーで開く
explorer-action-refresh-subtree = 🔄 サブツリーを更新
explorer-action-move-trash = ♻ ゴミ箱に移動
explorer-action-delete-permanently = 🗑 完全に削除
explorer-action-refresh-directory = 🔄 ディレクトリを更新

# Explorer Empty State
explorer-empty-state = 「新規スキャン」をクリックしてディスク使用量を確認してください。
choose-an-option = オプションを選択
web-viewer = Web ビューアー
load-demo = 👁 サンプルデモのスナップショットを読み込み
placeholder-treemap = スキャンしたファイルシステムがここにツリーマップとして可視化されます。
placeholder-plots = スキャンしたファイルシステムがここにグラフとして表示されます。

# Treemap Zoom & Navigation
zoom-up = ⏶ 上へ
zoom-reset = ❌ リセット
zoom-to-dir = 🔍 ツリーマップでフォーカス
zoom-up-level = ⏶ 1つ上の階層へ
zoom-empty-dir = ディレクトリは空です

# --- Extensions Panel ---
extensions-header = 📂 拡張子
extensions-empty = 統計はまだ収集されていません。
extensions-hover-files = ファイル： { $count }

# --- Operations (Context Actions) ---
op-up-one-level = 1 つ上のレベルへ
op-zoom-treemap = ツリーマップでフォーカス
op-refresh-entire-scan = スキャン全体を更新
op-refresh-directory = ディレクトリを更新
op-open-file = ファイルを開く
op-open-file-manager = ファイルマネージャーで開く
op-open-terminal = ここでターミナルを開く
op-copy-path = パスをコピー
op-copy-name = 名前をコピー
op-move-trash = ゴミ箱に移動
op-permanently-delete = 完全に削除

# Toast Notifications
toast-already-root = すでにルートレベルにあります
toast-navigated-up = 1 つ上のレベルに移動しました
toast-zoomed-treemap = ディレクトリにフォーカスしました
toast-refreshing-scan = スキャン全体を更新しています…
toast-refreshing-dir = 選択したディレクトリを更新しています…
toast-opened-file = 開きました： { $path }
toast-failed-open-file = ファイルを開けませんでした： { $error }
toast-opened-manager = ファイルマネージャーで開きました： { $path }
toast-failed-open-manager = ファイルマネージャーで開けませんでした： { $error }
toast-opened-terminal = ターミナルを開きました： { $path }
toast-failed-open-terminal = ターミナルを開けませんでした： { $error }
toast-copied-paths = { $count ->
   *[other] { $count } 個のパスをクリップボードにコピーしました
}
toast-copied-names = { $count ->
   *[other] { $count } 個の名前をクリップボードにコピーしました
}

# --- Modals ---
modal-remember-confirmation = 今後すべてのファイルとディレクトリでこの確認内容を記憶する
modal-process-multiple = { $count } 個の重複ファイル/項目を処理しようとしています：
modal-process-single = 次のパスを処理しようとしています：
# Confirm Deletion/Trash/Link Modals
modal-delete-title = ⚠ 完全削除の警告
modal-delete-header = ⚠ 完全削除の警告！
modal-delete-info = 合計サイズ： { $size }
modal-delete-warning = これは再帰的な削除です。選択したパス以下のすべてのファイル、フォルダ、およびサブディレクトリが完全に削除され、復元できません (ゴミ箱をバイパスします)。
modal-delete-checkbox = ファイルが完全に削除され、復元できないことを理解しました。
modal-delete-confirm = 🗑 はい、完全に削除します

modal-trash-title = ♻ ゴミ箱に移動
modal-trash-header = ♻ ゴミ箱に移動
modal-trash-info = 合計サイズ： { $size }
modal-trash-warning = 選択したパスとそのすべての内容がシステムのゴミ箱に移動され、後で復元または完全に削除できます。
modal-trash-checkbox = ゴミ箱に移動することを確認します。
modal-trash-confirm = ♻ はい、ゴミ箱に移動します

modal-delete-duplicates-title = ⚠ 重複の完全削除に関する警告
modal-delete-duplicates-header = ⚠ 重複ファイルの完全削除の警告！
modal-delete-duplicates-info = 回収可能な合計容量： { $size }
modal-delete-duplicates-warning = 選択したすべてのファイルが完全に削除され、復元できません (ゴミ箱をバイパスします)。
modal-delete-duplicates-checkbox = ファイルが完全に削除され、復元できないことを理解しました。
modal-delete-duplicates-confirm = 🗑 はい、選択した項目を完全に削除します

modal-trash-duplicates-title = ♻ 重複ファイルをゴミ箱に移動
modal-trash-duplicates-header = ♻ 重複ファイルをゴミ箱に移動
modal-trash-duplicates-info = 回収可能な合計容量： { $size }
modal-trash-duplicates-warning = 選択したすべてのファイルがゴミ箱に移動されます。
modal-trash-duplicates-checkbox = これらのファイルをゴミ箱に移動することを確認します。
modal-trash-duplicates-confirm = ♻ はい、選択した項目をゴミ箱に移動します

modal-hardlink-duplicates-title = 🔗 重複ファイルをハードリンクに置き換え
modal-hardlink-duplicates-header = 🔗 重複ファイルをハードリンクに置き換え
modal-hardlink-duplicates-info = 処理するファイルの合計： { $count }。累積仮想サイズ： { $size }
modal-hardlink-duplicates-warning = 選択した重複ファイルが削除され、各グループに残る元のファイルを指すファイルシステムレベルのハードリンクに置き換えられます。ファイルの見た目は維持されながら、実際の物理ストレージが解放されます。
modal-hardlink-duplicates-checkbox = 選択したファイルをハードリンクに置き換えることを確認します。
modal-hardlink-duplicates-confirm = 🔗 はい、ハードリンクに置き換えます

modal-softlink-duplicates-title = 🔗 重複ファイルをソフトリンクに置き換え
modal-softlink-duplicates-header = 🔗 重複ファイルをソフトリンクに置き換え
modal-softlink-duplicates-info = 処理するファイルの合計： { $count }。累積仮想サイズ： { $size }
modal-softlink-duplicates-warning = 選択した重複ファイルが削除され、各グループに残る元のファイルを指すファイルシステムレベルのソフトリンク (シンボリックリンク) に置き換えられます。ファイルの見た目は維持されながら、実際の物理ストレージが解放されます。
modal-softlink-duplicates-checkbox = 選択したファイルをソフトリンクに置き換えることを確認します。
modal-softlink-duplicates-confirm = 🔗 はい、ソフトリンクに置き換えます

# Path Does Not Exist Modal
modal-path-not-exist-title = ❌ パスが存在しません！
modal-path-not-exist-msg = エラー： 削除しようとしているパスはディスク上に存在しません。
modal-close-btn = 閉じる
modal-details-label = 詳細： 
modal-cancel-btn = キャンセル

# Elevation Recommended Modal
modal-elevation-title = ⚠ 管理者権限を推奨
modal-elevation-desc = eDirStat はデフォルトでは標準ユーザー権限で実行されます。ただし、Windows では物理ディスクハンドルへの直接アクセスは管理者アカウントに厳しく制限されています。
modal-elevation-mft-disabled = Windows NTFS MFT ドライバーが無効です
modal-elevation-mft-desc = 管理者権限がないと、ディスク直接アクセスの MFT スキャナーを初期化できません。ファイル解析にはフォールバックの標準走査ドライバーが使用され、スキャン性能が最大 20 倍低下します。
modal-elevation-relaunch-prompt = 管理者権限でアプリケーションを今すぐ再起動しますか？
modal-elevation-continue-std = 標準ユーザーとして続行
modal-elevation-relaunch-btn = 🛡 管理者として再起動

# About Modal
modal-about-title = ℹ eDirStat のバージョン情報
modal-about-author = By: Cody Wyatt Neiman (xangelix) <neiman@cody.to>
modal-about-desc1 = Rust で構築された高性能なディスク容量アナライザーおよび重複排除ツールキット。
modal-about-desc2 = ワークスティーリング方式の並列ディレクトリ走査、ゼロ解析レイアウトデシリアライズによる圧縮スナップショット、応答性の高いインタラクティブなツリーマップを備えています。
modal-about-desc3 = 統合された重複排除ツールは、多段の暗号学的ハッシュパイプラインを実行して重複グループを安全に特定し、回収可能な容量を計算し、システムレベルのハードリンクを尊重します。
modal-about-licenses-btn = オープンソースライセンスを表示
modal-about-version = v{ $version }

# How Deduplication Works Modal
modal-how-dedup-title = ℹ 重複排除の仕組み
modal-how-dedup-desc1 = すべてのファイルのバイトを直接比較する代わりに (低速なペアワイズの O(N²) スキャンが必要になります)、このシステムは高度に最適化された 7 段階のパイプラインを使用して、同一のコンテンツを安全かつ効率的に特定します。
modal-how-dedup-pipeline-title = 7 段階のパイプライン：
modal-how-dedup-why-title = なぜこれで十分なのか？
modal-how-dedup-why-desc1 = この多段フィルターにより、サイズ、プレフィックス、中間点、サフィックス、分散ブロックサンプルがすべて一致するファイルのみが全体を読み取られます。最後に 256-bit の BLAKE3 暗号学的ハッシュを比較することで、業界標準の安全な転送プロトコルと同等の安全性が確保され、低速なペアワイズのバイト単位比較が不要になります。

# How Deduplication Works Steps
modal-how-dedup-step1-title = 1. サイズによる分割
modal-how-dedup-step1-desc = ファイルはバイト単位の正確なサイズでグループ化されます。一意のサイズを持つファイルは即座に破棄され、ディスク I/O が完全に回避されます。
modal-how-dedup-step2-title = 2. プレフィックスのハッシュ化
modal-how-dedup-step2-desc = 残った候補の先頭 4KB がハッシュ化されます。これにより、ヘッダーやメタデータ形式が異なるファイルが迅速に除外されます。
modal-how-dedup-step3-title = 3. 中間点のハッシュ化
modal-how-dedup-step3-desc = 残ったファイルの中央から 4KB のブロックがハッシュ化され、内部構造の違いが検出されます。
modal-how-dedup-step4-title = 4. サフィックスのハッシュ化
modal-how-dedup-step4-desc = データの最後の 4KB がハッシュ化されます。末尾の内容やメタデータの違いを特定するのに非常に効果的です。
modal-how-dedup-step5-title = 5. マルチレンジハッシュ化
modal-how-dedup-step5-desc = 大きなファイル (100MB 超) では、全長にわたって定期的なブロックサンプリングを行い、ファイル全体を読み取らずにコンテンツの一貫性を検証します。
modal-how-dedup-step6-title = 6. 完全な BLAKE3 ハッシュ化
modal-how-dedup-step6-desc = 残った候補について、完全な BLAKE3 暗号学的ハッシュが計算されます。256-bit 空間の高い衝突耐性により、ハッシュが一致する場合にファイルが異なる可能性は天文学的に低く、ペアワイズ比較を必要とせず、同一性の非常に信頼性の高い証明が得られます。
modal-how-dedup-step7-title = 7. タイムスタンプの検証
modal-how-dedup-step7-desc = 重複排除アクションを表示または実行する直前に、アプリケーションはディスク上のファイルのタイムスタンプを検証し、スナップショット生成以降に発生した変更から保護します。

# Open Source Licenses Modal
modal-licenses-title = 📜 オープンソースライセンス
modal-licenses-desc = このアプリケーションでは以下のサードパーティライブラリとクレートが使用されています：

# Processing Modal
modal-processing-title = ⏳ 処理しています…
modal-processing-deletion = ファイルとディレクトリを削除しています…
modal-processing-trash = ファイルとディレクトリをゴミ箱に移動しています…
modal-processing-hardlink = 重複ファイルをハードリンクに置き換えています…
modal-processing-softlink = 重複ファイルをソフトリンクに置き換えています…

# Explorer Column Headers
explorer-hdr-name = 名前
explorer-hdr-percentage = 割合
explorer-hdr-size = サイズ
explorer-hdr-items = 項目
explorer-hdr-files = ファイル
explorer-hdr-subdirs = サブディレクトリ
explorer-hdr-created = 作成日時
explorer-hdr-modified = 変更日時

# Update Checker
update-checking = 更新を確認しています…
update-available = 新しいバージョン { $version } が利用可能です！
update-up-to-date = 最新の状態です
update-failed = 更新の確認に失敗しました： { $error }

# Themes
theme = 🎨 テーマ
theme-dark = ダーク
theme-high-contrast = ハイコントラスト
theme-light = ライト
theme-system = システム

# New Scan Options Modal
modal-scan-options-title = 新規スキャンのオプション
modal-scan-options-header = 新しいスキャンを開始
modal-scan-options-path-label = スキャンするディレクトリのパス：
modal-scan-options-paste-tooltip = クリップボードから貼り付け
modal-scan-options-browse-tooltip = フォルダを参照…
modal-scan-options-scan-btn = スキャン
modal-scan-options-cancel-btn = キャンセル
modal-scan-options-same-filesystem = スキャンを同じファイルシステム/ボリュームに限定
modal-scan-options-drives-header = 💽 ストレージドライブとボリューム
modal-scan-options-refresh-tooltip = ストレージドライブを更新
modal-scan-options-root-system = ルートシステム
modal-scan-options-selected-badge = ✅ 選択済み
modal-scan-options-free-of = { $total } 中 { $free } が空き
modal-scan-options-subtitle = 分析するストレージボリューム、クイックアクセス、またはカスタムディレクトリを選択してください。
modal-scan-options-quick-access = 📍 クイックアクセス
modal-scan-options-path-hint = /スキャンするパス
modal-scan-options-hint = ℹ 上記のドライブを選択するか、ディレクトリパスを入力してください。
modal-scan-options-sandbox-auth = 🔒 サンドボックスのアクセス許可が必要です — スキャンをクリックして権限を付与してください
modal-scan-options-valid-dir = ✅ 有効なディレクトリ — スキャン可能
modal-scan-options-points-to-file = ⚠ パスがファイルを指しています — フォルダを選択してください。
modal-scan-options-dir-not-exist = ⚠ ディレクトリがファイルシステムに存在しません。
quick-loc-home = 🏠 ホーム
quick-loc-documents = 📄 ドキュメント
quick-loc-downloads = 📥 ダウンロード
quick-loc-desktop = 🖥 デスクトップ
quick-loc-pictures = 🖼 ピクチャ
search-use-regex = 正規表現を使用 (Regex)
search-match-case = 大文字・小文字を区別
dedup-pref-dir-hint = 例: /home/user/Archive

file-menu-close = スキャンを閉じる
file-menu-quit = 終了
