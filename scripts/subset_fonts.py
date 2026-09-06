#!/usr/bin/env python3
"""Subset the Noto fallback fonts against the fluent-zero charset corpus.

Usage: subset_fonts.py <charset-corpus> <fonts-dir> <output-dir>

The charset corpus is the deterministic character set exported by
fluent-zero-build during the GUI crate's build (every character used across
all .ftl translations). Each expected source font in <fonts-dir> is subsetted
down to exactly those codepoints and written to <output-dir> under the same
name, keeping all OpenType layout features so complex-script shaping
(Devanagari/Bengali conjuncts) keeps working.

Exit status is non-zero if any expected font is missing or fails to subset.
The GUI build relies on this: absent fonts are skipped gracefully by build.rs
before this script runs, but once fonts are present, a broken font or
toolchain must fail the build loudly.
"""

import sys
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont

# Expected sources in <fonts-dir> (downloaded by scripts/fetch_fonts.sh).
# To cover another script, add the font here and to fetch_fonts.sh.
FONTS = [
    "NotoSansSC-Regular.otf",
    "NotoSansTC-Regular.otf",
    "NotoSansJP-Regular.otf",
    "NotoSansKR-Regular.otf",
    "NotoSansBengali-Regular.ttf",
    "NotoSansDevanagari-Regular.ttf",
    "NotoSansArabic-Regular.ttf",
    "NotoSansSymbols2-Regular.ttf",
]

KEYBOARD_SYMBOLS = "⌥⌫⏎⇧⌘⎋⬅⬆⮨☁🔒⛓⮡⚡🛠"


def human_size(num_bytes: int) -> str:
    if num_bytes >= 1 << 20:
        return f"{num_bytes / (1 << 20):.2f} MiB"
    if num_bytes >= 1 << 10:
        return f"{num_bytes / (1 << 10):.1f} KiB"
    return f"{num_bytes} B"


def subset_font(corpus: Path, src: Path, dst: Path) -> tuple[int, int]:
    """Subset src onto the corpus codepoints; return (codepoints, glyphs) kept."""
    import tempfile

    corpus_text = corpus.read_text(encoding="utf-8") + KEYBOARD_SYMBOLS
    with tempfile.NamedTemporaryFile(mode="w", encoding="utf-8", delete=False) as tf:
        tf.write(corpus_text)
        temp_corpus = tf.name

    try:
        subset.main(
            [
                str(src),
                f"--text-file={temp_corpus}",
                f"--output-file={dst}",
                # Keep shaping tables for the retained glyphs (Indic conjuncts).
                "--layout-features=*",
            ]
        )
    finally:
        Path(temp_corpus).unlink(missing_ok=True)

    font = TTFont(dst)
    codepoints = len(font.getBestCmap())
    glyphs = font["maxp"].numGlyphs
    font.close()
    return codepoints, glyphs


def find_charset_corpus(repo_root: Path) -> Path | None:
    """Finds the most recently generated fluent_charset.txt in target/ if available."""
    target_dir = repo_root / "target"
    candidates = []
    if target_dir.is_dir():
        for path in target_dir.glob("**/fluent_charset.txt"):
            if path.is_file() and path.stat().st_size > 0:
                candidates.append((path.stat().st_mtime, path))
    if candidates:
        candidates.sort(key=lambda x: x[0], reverse=True)
        return candidates[0][1]
    return None


def main(argv: list[str]) -> int:
    repo_root = Path(__file__).resolve().parent.parent

    if len(argv) == 4:
        corpus = Path(argv[1])
        fonts_dir = Path(argv[2])
        out_dir = Path(argv[3])
    elif len(argv) == 1:
        corpus_opt = find_charset_corpus(repo_root)
        if corpus_opt is None:
            print(
                "error: charset corpus not found in target/. Run `cargo check -p edirstat-gui` first or pass paths explicitly.",
                file=sys.stderr,
            )
            print(__doc__, file=sys.stderr)
            return 2
        corpus = corpus_opt
        fonts_dir = repo_root / "crates/edirstat-gui/assets/fonts/raw"
        out_dir = repo_root / "crates/edirstat-gui/assets/fonts"
    else:
        print(__doc__, file=sys.stderr)
        return 2

    if not corpus.is_file():
        print(f"error: charset corpus not found: {corpus}", file=sys.stderr)
        return 2
    if not fonts_dir.is_dir():
        print(f"error: fonts directory not found: {fonts_dir}", file=sys.stderr)
        return 2
    out_dir.mkdir(parents=True, exist_ok=True)

    failures = []
    for name in FONTS:
        src = fonts_dir / name
        if not src.is_file():
            print(
                f"error: expected font missing: {src} (run scripts/fetch_fonts.sh)",
                file=sys.stderr,
            )
            failures.append(name)
            continue
        dst = out_dir / name
        try:
            codepoints, glyphs = subset_font(corpus, src, dst)
        except Exception as e:  # noqa: BLE001 — report per font, then fail the run
            print(f"error: subsetting {name} failed: {e}", file=sys.stderr)
            failures.append(name)
            continue
        before = src.stat().st_size
        after = dst.stat().st_size
        print(
            f"{name}: {human_size(before)} -> {human_size(after)} "
            f"({after / before:.2%}), {codepoints} codepoints / {glyphs} glyphs"
        )

    if failures:
        print(
            f"error: {len(failures)}/{len(FONTS)} fonts failed: {', '.join(failures)}",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
