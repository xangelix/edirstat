#!/usr/bin/env python3
"""
sync_devlogs.py — Single source of truth sync between Markdown devlogs and itch.io HTML.

Workflow:
  1. Author devlog release notes in Markdown (`web/content/blog/vX.Y.Z.md`) or `CHANGELOG.md`.
  2. Run `./scripts/sync_devlogs.py` to:
     - Automatically compile clean HTML devlogs into `web/itch/devlog/vX.Y.Z.html` for itch.io.
     - Keep `web/content/blog/` synchronized with `CHANGELOG.md`.
     - Prepare posts for Zola static site generation on `edirstat.com/blog/`.

Usage:
  ./scripts/sync_devlogs.py               # Sync both directions (default)
  ./scripts/sync_devlogs.py --export-itch # Export markdown blog posts -> itch.io HTML
  ./scripts/sync_devlogs.py --from-changelog # Import all releases from CHANGELOG.md
"""

import argparse
import re
import sys
from pathlib import Path

try:
    import markdown
except ImportError:
    print("ERROR: 'markdown' package is required. Install via: pip install markdown", file=sys.stderr)
    sys.exit(1)

REPO_ROOT = Path(__file__).resolve().parent.parent
CHANGELOG_PATH = REPO_ROOT / "CHANGELOG.md"
BLOG_DIR = REPO_ROOT / "web" / "content" / "blog"
ITCH_DIR = REPO_ROOT / "web" / "itch" / "devlog"

RELEASE_TITLES = {
    "2.2.0": "eDirStat v2.2.0 — Treemap Zoom, 18 Languages & Mac App Store Release",
    "2.1.0": "eDirStat v2.1.0 — 8 Languages, Themes, Web Snapshot Viewer & Linux MFT",
    "2.0.1": "eDirStat v2.0.1 — Snapshot Status Bar & Target-CPU Optimizations",
    "2.0.0": "eDirStat v2.0.0 — Major 2.0 Overhaul, Direct MFT & Classic Layout",
}


def parse_changelog(changelog_path: Path):
    """Parse CHANGELOG.md into a list of release dictionaries."""
    if not changelog_path.exists():
        print(f"Warning: {changelog_path} not found.")
        return []

    content = changelog_path.read_text(encoding="utf-8")
    # Matches: ## [v2.2.0] - 2026-09-06
    sections = re.split(r"^##\s+\[v?([^\]]+)\]\s*-\s*([0-9]{4}-[0-9]{2}-[0-9]{2})", content, flags=re.MULTILINE)

    releases = []
    # split results: [header_before, ver1, date1, body1, ver2, date2, body2, ...]
    for i in range(1, len(sections), 3):
        ver = sections[i].strip()
        date = sections[i + 1].strip()
        body = sections[i + 2].strip()

        # Extract opening summary paragraph for description
        desc_match = re.search(r"^\*\*(.*?)\*\*", body, flags=re.DOTALL)
        if desc_match:
            desc = desc_match.group(1).replace("\n", " ").strip()
        else:
            # Fallback to first non-empty, non-header line, stripping list bullets
            candidate_lines = []
            for l in body.splitlines():
                s = l.strip()
                if not s or s.startswith(">") or s.startswith("#"):
                    continue
                s = re.sub(r"^[-*+]\s+", "", s)
                s = s.replace("**", "").replace("`", "")
                candidate_lines.append(s)
                if len(" ".join(candidate_lines)) >= 120:
                    break
            if candidate_lines:
                desc = " ".join(candidate_lines)
            else:
                desc = f"eDirStat {ver} release notes, improvements, and updates."

        title = RELEASE_TITLES.get(ver, f"eDirStat v{ver} Release Notes")

        releases.append({
            "version": ver,
            "date": date,
            "title": title,
            "description": desc,
            "body": body
        })

    return releases


def convert_markdown_to_itch_html(md_text: str) -> str:
    """Convert markdown into clean, self-contained HTML suitable for itch.io devlogs."""
    # Convert markdown using standard extensions
    html = markdown.markdown(
        md_text,
        extensions=[
            "extra",
            "sane_lists",
            "smarty"
        ]
    )

    # Format tweaks for itch.io aesthetic:
    # Ensure paragraphs, blockquotes, and lists render cleanly
    html = re.sub(r">\s*\n", "> ", html)
    footer = '<p>🌐 <em>Visit the official website and launch the interactive web viewer at <a href="https://edirstat.com" target="_blank">edirstat.com</a></em></p>'
    return html.strip() + "\n" + footer + "\n"


def promote_headings_for_post(body: str) -> str:
    """Promote ### to ## and #### to ### for compliant h1 -> h2 -> h3 hierarchy in blog posts."""
    lines = []
    for line in body.splitlines():
        if line.startswith("#### "):
            lines.append("### " + line[5:])
        elif line.startswith("### "):
            lines.append("## " + line[4:])
        else:
            lines.append(line)
    return "\n".join(lines)


def import_changelog_to_blog():
    """Extract release notes from CHANGELOG.md and create Zola blog posts."""
    BLOG_DIR.mkdir(parents=True, exist_ok=True)
    releases = parse_changelog(CHANGELOG_PATH)
    count = 0

    for rel in releases:
        ver = rel["version"]
        date = rel["date"]
        title = rel["title"].replace('"', '\\"')
        desc = rel["description"].replace('"', '\\"')
        body = promote_headings_for_post(rel["body"])

        post_path = BLOG_DIR / f"v{ver}.md"
        frontmatter = f"""+++
title = "{title}"
date = {date}
description = "{desc}"

[extra]
version = "{ver}"
badge = "Release"
+++
"""
        post_path.write_text(frontmatter + body + "\n", encoding="utf-8")
        print(f"  ✓ Blog Post: {post_path.relative_to(REPO_ROOT)}")
        count += 1

    return count


def export_blog_to_itch():
    """Read Zola blog posts and compile them into itch.io devlog HTML files."""
    ITCH_DIR.mkdir(parents=True, exist_ok=True)
    count = 0

    for md_path in sorted(BLOG_DIR.glob("v*.md")):
        content = md_path.read_text(encoding="utf-8")
        # Split Zola frontmatter +++ ... +++
        parts = re.split(r"^\+\+\+\s*$", content, flags=re.MULTILINE)
        if len(parts) >= 3:
            body = parts[2].strip()
        else:
            body = content.strip()

        html_out = ITCH_DIR / f"{md_path.stem}.html"
        converted = convert_markdown_to_itch_html(body)
        html_out.write_text(converted, encoding="utf-8")
        print(f"  ✓ itch.io HTML: {html_out.relative_to(REPO_ROOT)}")
        count += 1

    return count


def main():
    parser = argparse.ArgumentParser(description="Sync devlogs between CHANGELOG.md, blog posts, and itch.io HTML.")
    parser.add_argument("--from-changelog", action="store_true", help="Import releases from CHANGELOG.md into web/content/blog/")
    parser.add_argument("--export-itch", action="store_true", help="Export web/content/blog/*.md to web/itch/devlog/*.html")

    args = parser.parse_args()

    if args.from_changelog:
        print("==> Importing from CHANGELOG.md to Blog...")
        n = import_changelog_to_blog()
        print(f"Done! Imported {n} releases.")
        return

    if args.export_itch:
        print("==> Exporting Blog posts to itch.io HTML devlogs...")
        n = export_blog_to_itch()
        print(f"Done! Exported {n} devlogs.")
        return

    # Default: sync both
    print("==> Synchronizing devlogs (CHANGELOG.md -> Blog -> itch.io HTML)...")
    n_blog = import_changelog_to_blog()
    n_itch = export_blog_to_itch()
    print(f"==> Sync complete! {n_blog} blog posts and {n_itch} itch.io devlogs synchronized.")


if __name__ == "__main__":
    main()
