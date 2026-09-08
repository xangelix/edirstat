# Menu Bar Dropdowns
file = 파일
view = 보기
help = 도움말

# Menu Bar Actions
new-scan = 📁 새 검사
save-snapshot = 💾 스냅샷 저장
load-snapshot = 📖 스냅샷 불러오기

# Menu Bar Status
idle = 대기 중

# View Menu Options
monospace-paths = 🅰 고정폭 경로
highlight-duplicates = ✨ 중복 강조 표시
treemap-borders = 🔳 트리맵 테두리
treemap-style =  트리맵 스타일
treemap-style-vertical = 세로 그라데이션
treemap-style-offset-vertical = 오프셋 세로 그라데이션
treemap-style-diagonal = 대각선 그라데이션
treemap-style-cushion = 쿠션 음영
deletion-confirmation = 🗑 삭제 확인
trash-confirmation = ♻ 휴지통 이동 확인
time-format = 🕒 시간 형식
language = 💬 언어
layout-mode = 레이아웃 모드:
classic-layout = 클래식 레이아웃
windirstat-layout = WinDirStat 레이아웃
vis-mode-treemap = 📊 트리맵
vis-mode-plots = 📈 차트
select-plot-label = 차트 선택:
vis-mode-deduplicator = 👥 중복 파일 찾기
search-filter-label = 🔍 필터:

# Panel Toggles
toggle-left-panel = { $collapsed ->
    [true] ▶ 왼쪽 패널 표시 (F9)
   *[false] ◀ 왼쪽 패널 숨기기 (F9)
}

toggle-right-panel = { $collapsed ->
    [true] { $is_classic ->
        [true] ◀ 오른쪽 패널 표시 (F11)
       *[false] ▶ 확장자 패널 표시 (F11)
    }
   *[false] { $is_classic ->
        [true] ▶ 오른쪽 패널 숨기기 (F11)
       *[false] ◀ 확장자 패널 숨기기 (F11)
    }
}

collapse-all = ⏏ 모두 축소
about = ℹ 정보
web-not-available = 웹 버전에서는 사용할 수 없는 기능입니다

# Status Indicators
scanning-disk = 디스크를 검사하는 중입니다...
scan-complete = 검사가 완료되었습니다
scan-cancelled = 검사가 취소되었습니다
path-label = 경로: { $path }
worker-threads = ⚡ { $count }개의 작업 스레드
worker-threads-hover = 디렉터리 탐색에 할당된 병렬 작업 가로채기 CPU 코어 수입니다.

# Stats Panel (Bottom)
directories-count = 📁 디렉터리: { $count }
files-count = 📄 파일: { $count }
total-size = 💾 총 크기: { $size }
elapsed-time = ⏱ 시간: { $time }
scan-speed = ⚡ 속도: { $speed }/s

# Selection Info
selection-path = 선택: { $path }
selection-items = 선택: { $count ->
   *[other] { $count }개 항목
}

# Plot Types
plot-size-distribution = 📊 파일 크기 분포
plot-age-size = 🌌 파일 연령 대비 파일 크기
plot-dir-composition = 🍰 디렉터리 구성
plot-extension-boxplot = 📦 확장자별 파일 크기
plot-temporal-timeline = ⏱ 연결된 시간 타임라인
plot-deduplicator-waste = 👥 확장자별 중복 낭비 공간

# --- Deduplicator Strings ---
dedup-desc = 암호학적으로 안전한 BLAKE3 해시를 사용하여 바이트 단위로 동일한 파일을 찾아 안전하게 제거합니다.
dedup-how-it-works = ℹ 작동 방식
dedup-min-size = 최소 파일 크기:
dedup-ignore-system = 시스템 파일 무시
dedup-ignore-hidden = 숨겨진 파일 무시
dedup-start-scan = ⚡ 중복 제거 검사 시작
dedup-scan-first = 먼저 디렉터리를 검사하세요.
dedup-cancelled-msg = 검사가 취소되었습니다. 중복을 찾으려면 새 검사를 시작하세요.
dedup-analyzing = 파일을 분석하는 중...
dedup-no-duplicates = 중복 그룹을 찾지 못했습니다. 최소 파일 크기를 줄이거나 다른 폴더를 검사해 보세요.
no-permission = 권한 없음
hardlink-badge = 하드 링크
dedup-select-items = 🎯 항목 선택...
dedup-select-all-but-oldest = 🎯 가장 오래된 항목 제외
dedup-select-all-but-newest = 🎯 가장 최신 항목 제외
dedup-select-all-but-shortest = 🎯 경로가 가장 짧은 항목 제외
dedup-select-all-but-rootmost = 🎯 루트에 가장 가까운 항목 제외
dedup-select-all-but-longest = 🎯 경로가 가장 긴 항목 제외
dedup-pref-dir-pattern = 우선 디렉터리 패턴:
dedup-select-all-but-pref = 🎯 우선 디렉터리 제외
dedup-clear-selection = ❌ 선택 해제
dedup-link-menu = 🔗 링크... ({ $count }개의 파일)
dedup-link-menu-disabled = 🔗 링크... (0개의 파일)
dedup-link-hardlinks = 🔗 선택 항목을 하드 링크로 교체
dedup-link-softlinks = 🔗 선택 항목을 소프트 링크로 교체
dedup-remove-menu = 🗑 삭제... ({ $count }개의 파일, { $size })
dedup-remove-menu-disabled = 🗑 삭제... (0개의 파일)
dedup-remove-trash = ♻ 선택 항목을 휴지통으로 이동
dedup-remove-delete = 🗑 선택 항목을 영구적으로 삭제
dedup-warning-title = ⚠ 데이터 손실 경고
dedup-warning-desc = { $count ->
   *[other] { $count }개 파일의 모든 버전을 삭제합니다
}
dedup-warning-no-original = 원본 복사본이 남지 않습니다:
dedup-warning-details = 아래 나열된 파일의 원본과 모든 중복 복사본을 선택했습니다. 이 파일들을 삭제하면 영구적인 데이터 손실이 발생할 가능성이 높습니다:
dedup-cancel-hover = 클릭하여 검사 취소
scan-cancel-hover = 클릭하여 검사 취소
dedup-current-label = 현재
dedup-phase1-size = 단계 1/7: 검사된 모든 파일을 크기별로 그룹화하는 중...
dedup-phase1-filter = 단계 1/7: 중복 후보에서 제외 항목을 필터링하는 중...
dedup-phase2-prefix = 단계 2/7: 파일 접두사 해시 계산 중(처음 4KB)...
dedup-phase3-midpoint = 단계 3/7: 파일 중간 지점 해시 계산 중...
dedup-phase4-suffix = 단계 4/7: 파일 접미사 해시 계산 중...
dedup-phase5-multirange = 단계 5/7: 대용량 파일 다중 범위 해시 계산 중...
dedup-phase6-full = 단계 6/7: 나머지 후보의 전체 BLAKE3 해시 계산 중...
dedup-phase7-validation = 단계 7/7: 최종 타임스탬프 확인 중...
dedup-phase-finished = { $duration } 만에 완료되었습니다! 중복 그룹 { $count }개를 찾았습니다. 확보 가능한 공간: { $space }
dedup-scan-cancelled-with-error = 검사가 취소되었습니다: { $error }

# Deduplicator Table Headers
dedup-hdr-checkbox = [     ]
dedup-hdr-filename = 파일 이름
dedup-hdr-directory = 상위 디렉터리
dedup-hdr-size = 크기
dedup-hdr-reclaimable = 확보 가능
dedup-hdr-created = 만든 날짜
dedup-hdr-modified = 수정한 날짜
dedup-copies-selected = ({ $count ->
   *[other] { $count }개 복사본 선택됨
})

# --- Explorer Details Panel ---
explorer-details-header = ℹ 세부 정보
explorer-deselect-hover = 선택한 항목 모두 해제
explorer-deselect-single-hover = 선택한 항목 해제
explorer-selected-items-count = { $count ->
   *[other] { $count }개 항목 선택됨
}
explorer-total-size = 총 크기: { $size }
explorer-files = 파일: { $count }
explorer-directories = 디렉터리: { $count }
explorer-actions-title = 작업
explorer-actions-operations = 작업:
explorer-action-refresh-hover = 선택한 모든 디렉터리 하위 트리를 새로 고칩니다
explorer-grid-type = 유형:
explorer-grid-size = 크기:
explorer-grid-bytes = 바이트:
explorer-grid-items = 항목:
explorer-grid-files = 파일:
explorer-grid-subdirs = 하위 디렉터리:
explorer-grid-user = 사용자:
explorer-grid-group = 그룹:
explorer-grid-permissions = 권한:
explorer-grid-path = 전체 경로:

# Explorer Type Names
type-symlink = 심볼릭 링크
type-directory = 디렉터리
type-file = 파일

# Explorer Actions
explorer-action-copy-path = 📋 경로 복사
explorer-action-open-file = 📄 파일 열기
explorer-action-open-manager = 🗁 파일 관리자에서 열기
explorer-action-refresh-subtree = 🔄 하위 트리 새로 고침
explorer-action-move-trash = ♻ 휴지통으로 이동
explorer-action-delete-permanently = 🗑 영구적으로 삭제
explorer-action-refresh-directory = 🔄 디렉터리 새로 고침

# Explorer Empty State
explorer-empty-state = 디스크 사용량을 살보려면 '새 검사'를 클릭하세요.
choose-an-option = 옵션 선택
web-viewer = 웹 뷰어
load-demo = 👁 샘플 데모 스냅샷 불러오기
placeholder-treemap = 검사된 파일 시스템이 여기에 트리맵으로 시각화됩니다.
placeholder-plots = 검사된 파일 시스템이 여기에 차트로 표시됩니다.

# Treemap Zoom & Navigation
zoom-up = ⏶ 위로
zoom-reset = ❌ 재설정
zoom-to-dir = 🔍 트리맵에서 포커스
zoom-up-level = ⏶ 한 수준 위로
zoom-empty-dir = 디렉터리가 비어 있습니다

# --- Extensions Panel ---
extensions-header = 📂 확장자
extensions-empty = 아직 수집된 통계가 없습니다.
extensions-hover-files = 파일: { $count }

# --- Operations (Context Actions) ---
op-up-one-level = 한 수준 위로
op-zoom-treemap = 트리맵에서 포커스
op-refresh-entire-scan = 전체 검사 새로 고침
op-refresh-directory = 디렉터리 새로 고침
op-open-file = 파일 열기
op-open-file-manager = 파일 관리자에서 열기
op-open-terminal = 여기에서 터미널 열기
op-copy-path = 경로 복사
op-copy-name = 이름 복사
op-move-trash = 휴지통으로 이동
op-permanently-delete = 영구적으로 삭제

# Toast Notifications
toast-already-root = 이미 루트 수준입니다
toast-navigated-up = 한 수준 위로 이동했습니다
toast-zoomed-treemap = 디렉터리로 트리맵 포커스됨
toast-refreshing-scan = 전체 검사를 새로 고치는 중...
toast-refreshing-dir = 선택한 디렉터리를 새로 고치는 중...
toast-opened-file = 열었습니다: { $path }
toast-failed-open-file = 파일을 열 수 없습니다: { $error }
toast-opened-manager = 파일 관리자에서 열었습니다: { $path }
toast-failed-open-manager = 파일 관리자에서 열 수 없습니다: { $error }
toast-opened-terminal = 터미널을 열었습니다: { $path }
toast-failed-open-terminal = 터미널을 열 수 없습니다: { $error }
toast-copied-paths = { $count ->
   *[other] 경로 { $count }개를 클립보드에 복사했습니다
}
toast-copied-names = { $count ->
   *[other] 이름 { $count }개를 클립보드에 복사했습니다
}

# --- Modals ---
modal-remember-confirmation = 앞으로 모든 파일 및 디렉터리에 대해 이 확인을 기억합니다
modal-process-multiple = { $count }개의 중복 파일/항목을 처리하려고 합니다:
modal-process-single = 다음 경로를 처리하려고 합니다:
# Confirm Deletion/Trash/Link Modals
modal-delete-title = ⚠ 영구적 삭제 경고
modal-delete-header = ⚠ 영구적 삭제 경고!
modal-delete-info = 총 크기: { $size }
modal-delete-warning = 이 작업은 재귀적 삭제입니다. 선택한 경로 아래의 모든 파일, 폴더 및 하위 디렉터리가 영구적으로 삭제되며 복구할 수 없습니다(휴지통을 거치지 않습니다).
modal-delete-checkbox = 파일이 영구적으로 삭제되며 복구할 수 없음을 이해했습니다.
modal-delete-confirm = 🗑 예, 영구적으로 삭제

modal-trash-title = ♻ 휴지통으로 이동
modal-trash-header = ♻ 휴지통으로 이동
modal-trash-info = 총 크기: { $size }
modal-trash-warning = 선택한 경로와 그 안의 모든 내용이 시스템 휴지통으로 이동되며, 나중에 복구하거나 영구적으로 삭제할 수 있습니다.
modal-trash-checkbox = 휴지통으로 이동할 것을 확인합니다.
modal-trash-confirm = ♻ 예, 휴지통으로 이동

modal-delete-duplicates-title = ⚠ 영구적 중복 제거 경고
modal-delete-duplicates-header = ⚠ 중복 항목 영구 삭제 경고!
modal-delete-duplicates-info = 확보할 총 공간: { $size }
modal-delete-duplicates-warning = 선택한 모든 파일이 영구적으로 삭제되며 복구할 수 없습니다(휴지통을 거치지 않습니다).
modal-delete-duplicates-checkbox = 파일이 영구적으로 삭제되며 복구할 수 없음을 이해했습니다.
modal-delete-duplicates-confirm = 🗑 예, 선택 항목을 영구적으로 삭제

modal-trash-duplicates-title = ♻ 중복 항목 휴지통으로 이동
modal-trash-duplicates-header = ♻ 중복 항목 휴지통으로 이동
modal-trash-duplicates-info = 확보할 총 공간: { $size }
modal-trash-duplicates-warning = 선택한 모든 파일이 휴지통으로 이동됩니다.
modal-trash-duplicates-checkbox = 이 파일들을 휴지통으로 이동할 것을 확인합니다.
modal-trash-duplicates-confirm = ♻ 예, 선택 항목을 휴지통으로 이동

modal-hardlink-duplicates-title = 🔗 중복 항목을 하드 링크로 교체
modal-hardlink-duplicates-header = 🔗 중복 항목을 하드 링크로 교체
modal-hardlink-duplicates-info = 처리할 총 파일 수: { $count }. 누적 가상 크기: { $size }
modal-hardlink-duplicates-warning = 선택한 중복 파일을 삭제하고 각 그룹에 남아 있는 원본 파일을 가리키는 파일 시스템 수준의 하드 링크로 교체합니다. 이렇게 하면 파일은 시각적으로 유지되면서 실제 물리적 저장 공간을 확보할 수 있습니다.
modal-hardlink-duplicates-checkbox = 선택한 파일을 하드 링크로 교체할 것을 확인합니다.
modal-hardlink-duplicates-confirm = 🔗 예, 하드 링크로 교체

modal-softlink-duplicates-title = 🔗 중복 항목을 소프트 링크로 교체
modal-softlink-duplicates-header = 🔗 중복 항목을 소프트 링크로 교체
modal-softlink-duplicates-info = 처리할 총 파일 수: { $count }. 누적 가상 크기: { $size }
modal-softlink-duplicates-warning = 선택한 중복 파일을 삭제하고 각 그룹에 남아 있는 원본 파일을 가리키는 파일 시스템 수준의 소프트 링크(심볼릭 링크)로 교체합니다. 이렇게 하면 파일은 시각적으로 유지되면서 실제 물리적 저장 공간을 확보할 수 있습니다.
modal-softlink-duplicates-checkbox = 선택한 파일을 소프트 링크로 교체할 것을 확인합니다.
modal-softlink-duplicates-confirm = 🔗 예, 소프트 링크로 교체

# Path Does Not Exist Modal
modal-path-not-exist-title = ❌ 경로가 존재하지 않습니다!
modal-path-not-exist-msg = 오류: 삭제하려는 경로가 디스크에 존재하지 않습니다.
modal-close-btn = 닫기
modal-details-label = 세부 정보: 
modal-cancel-btn = 취소

# Elevation Recommended Modal
modal-elevation-title = ⚠ 관리자 권한 권장
modal-elevation-desc = eDirStat은 기본적으로 표준 사용자 권한으로 실행됩니다. 그러나 Windows에서는 물리적 디스크 핸들에 대한 직접 액세스가 관리자 계정으로 엄격히 제한됩니다.
modal-elevation-mft-disabled = Windows NTFS MFT 드라이버 비활성화됨
modal-elevation-mft-desc = 관리자 권한이 없으면 디스크 직접 액세스 MFT 검사기를 초기화할 수 없습니다. 파일 분석에는 대체 표준 탐색 드라이버가 사용되며 검사 성능이 최대 20배 저하될 수 있습니다.
modal-elevation-relaunch-prompt = 지금 관리자 권한으로 애플리케이션을 다시 시작하시겠습니까?
modal-elevation-continue-std = 표준 사용자로 계속
modal-elevation-relaunch-btn = 🛡 관리자로 다시 시작

# About Modal
modal-about-title = ℹ eDirStat 정보
modal-about-author = By: Cody Wyatt Neiman (xangelix) <neiman@cody.to>
modal-about-license-btn = 📜 라이선스 (MIT)
modal-about-desc1 = Rust로 만든 고성능 디스크 공간 분석 및 중복 제거 도구 모음입니다.
modal-about-desc2 = 병렬 작업 가로채기 디렉터리 탐색, 파싱 없는 레이아웃 역직렬화를 사용하는 압축 스냅샷, 빠르게 반응하는 대화형 트리맵을 제공합니다.
modal-about-desc3 = 통합 중복 제거 도구는 다단계 암호화 해싱 파이프라인을 실행하여 중복 그룹을 안전하게 격리하고, 확보 가능한 공간을 계산하며, 시스템 수준 하드 링크를 보존합니다.
modal-about-licenses-btn = 오픈 소스 라이선스 보기
modal-about-version = v{ $version }

# How Deduplication Works Modal
modal-how-dedup-title = ℹ 중복 제거 작동 방식
modal-how-dedup-desc1 = 모든 파일의 바이트를 일일이 직접 비교하는 대신(느린 쌍별 O(N²) 검사 필요), 이 시스템은 고도로 최적화된 7단계 파이프라인을 활용하여 동일한 콘텐츠를 안전하고 효율적으로 식별합니다.
modal-how-dedup-pipeline-title = 7단계 파이프라인:
modal-how-dedup-why-title = 이 방식으로 충분한 이유
modal-how-dedup-why-desc1 = 이 다단계 필터는 크기, 접두사, 중간 지점, 접미사 및 분산 블록 샘플이 모두 동일한 파일만 전체를 읽도록 보장합니다. 마지막으로 256-bit BLAKE3 암호화 해시를 비교하면 업계 수준의 보안 전송 프로토콜에 필적하는 안전성을 제공하므로 느린 쌍별 바이트 단위 비교가 필요 없습니다.

# How Deduplication Works Steps
modal-how-dedup-step1-title = 1. 크기 분할
modal-how-dedup-step1-desc = 파일은 정확한 바이트 크기별로 그룹화됩니다. 크기가 고유한 파일은 즉시 제외되므로 디스크 I/O를 완전히 건너뜁니다.
modal-how-dedup-step2-title = 2. 접두사 해싱
modal-how-dedup-step2-desc = 남은 후보의 처음 4KB를 해시합니다. 이렇게 하면 헤더나 메타데이터 형식이 다른 파일을 빠르게 걸러냅니다.
modal-how-dedup-step3-title = 3. 중간 지점 해싱
modal-how-dedup-step3-desc = 남은 파일 중앙의 4KB 블록을 해시하여 내부 구조적 차이를 포착합니다.
modal-how-dedup-step4-title = 4. 접미사 해싱
modal-how-dedup-step4-desc = 마지막 4KB 데이터를 해시합니다. 이는 끝부분 내용이나 메타데이터의 차이를 식별하는 데 매우 효과적입니다.
modal-how-dedup-step5-title = 5. 다중 범위 해싱
modal-how-dedup-step5-desc = 대용량 파일(100MB 초과)은 전체 길이에 걸쳐 주기적으로 블록을 샘플링하여 파일 전체를 읽지 않고도 콘텐츠 일관성을 확인합니다.
modal-how-dedup-step6-title = 6. 전체 BLAKE3 해싱
modal-how-dedup-step6-desc = 남은 후보에 대해 전체 BLAKE3 암호화 해시를 계산합니다. 256-bit 공간의 높은 충돌 저항성 덕분에 해시가 일치하면 파일이 다를 가능성이 천문학적으로 낮으므로, 쌍별 비교 없이도 매우 신뢰할 수 있는 동일성 증거가 됩니다.
modal-how-dedup-step7-title = 7. 타임스탬프 확인
modal-how-dedup-step7-desc = 중복 제거 작업을 표시하거나 실행하기 직전에 애플리케이션은 디스크에 있는 파일의 타임스탬프를 확인하여 스냅샷 생성 이후 발생한 변경으로부터 보호합니다.

# Open Source Licenses Modal
modal-licenses-title = 📜 오픈 소스 라이선스
modal-licenses-tab-app = eDirStat (MIT)
modal-licenses-tab-deps = 타사 라이브러리
modal-licenses-app-desc = eDirStat은 MIT 라이선스로 배포되는 오픈 소스 애플리케이션입니다:
modal-licenses-desc = 이 애플리케이션에는 다음 타사 라이브러리 및 크레이트가 사용되었습니다:
modal-licenses-copy-btn = 📋 라이선스 복사
modal-licenses-copy-all-btn = 📋 라이선스 복사

# Processing Modal
modal-processing-title = ⏳ 처리 중...
modal-processing-deletion = 파일 및 디렉터리를 삭제하는 중...
modal-processing-trash = 파일 및 디렉터리를 휴지통으로 이동하는 중...
modal-processing-hardlink = 중복 항목을 하드 링크로 교체하는 중...
modal-processing-softlink = 중복 항목을 소프트 링크로 교체하는 중...

# Explorer Column Headers
explorer-hdr-name = 이름
explorer-hdr-percentage = 백분율
explorer-hdr-size = 크기
explorer-hdr-items = 항목
explorer-hdr-files = 파일
explorer-hdr-subdirs = 하위 디렉터리
explorer-hdr-created = 만든 날짜
explorer-hdr-modified = 수정한 날짜

# Update Checker
update-checking = 업데이트를 확인하는 중...
update-available = 새 버전 { $version }을 사용할 수 있습니다!
update-up-to-date = 최신 상태입니다
update-failed = 업데이트 확인에 실패했습니다: { $error }

# Themes
theme = 🎨 테마
theme-dark = 다크
theme-high-contrast = 고대비
theme-light = 라이트
theme-system = 시스템

# New Scan Options Modal
modal-scan-options-title = 새 검사 옵션
modal-scan-options-header = 새 검사 시작
modal-scan-options-path-label = 검사할 디렉터리 경로:
modal-scan-options-paste-tooltip = 클립보드에서 붙여넣기
modal-scan-options-browse-tooltip = 폴더 찾아보기...
modal-scan-options-scan-btn = 검사
modal-scan-options-cancel-btn = 취소
modal-scan-options-same-filesystem = 동일한 파일 시스템/볼륨으로 검사 제한
modal-scan-options-drives-header = 💽 저장소 드라이브 및 볼륨
modal-scan-options-refresh-tooltip = 저장소 드라이브 새로 고침
modal-scan-options-root-system = 루트 시스템
modal-scan-options-selected-badge = ✅ 선택됨
modal-scan-options-free-of = { $total } 중 { $free } 사용 가능
modal-scan-options-subtitle = 분석할 저장 볼륨, 빠른 위치 또는 사용자 지정 디렉터리를 선택하세요.
modal-scan-options-quick-access = 📍 빠른 접근 바로가기
modal-scan-options-path-hint = /스캔할/경로
modal-scan-options-hint = ℹ 위에서 드라이브를 선택하거나 디렉터리 경로를 입력하세요.
modal-scan-options-sandbox-auth = 🔒 샌드박스 접근 권한 필요 — 스캔을 클릭하여 접근 권한 부여
modal-scan-options-valid-dir = ✅ 유효한 디렉터리 — 스캔 준비 완료
modal-scan-options-points-to-file = ⚠ 경로가 파일을 가리키고 있습니다 — 폴더를 선택하세요.
modal-scan-options-dir-not-exist = ⚠ 파일 시스템에 디렉터리가 존재하지 않습니다.
quick-loc-home = 🏠 홈
quick-loc-documents = 📄 문서
quick-loc-downloads = 📥 다운로드
quick-loc-desktop = 🖥 바탕화면
quick-loc-pictures = 🖼 사진
search-use-regex = 정규식 사용 (Regex)
search-match-case = 대소문자 구분
dedup-pref-dir-hint = 예: /home/user/Archive

file-menu-close = 스캔 닫기
file-menu-quit = 종료

badge-dataless-cloud = 클라우드 / 데이터리스 파일
badge-symlink = 심볼릭 링크
badge-special-file = 특수 파일 (파이프 / 소켓 / 디바이스)
badge-permission-denied = 액세스 거부됨
