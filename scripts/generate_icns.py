#!/usr/bin/env python3
"""Assemble assets/img/icon.icns (the macOS app icon seaworthy.toml names)
from the committed PNG ladder, byte for byte reproducibly, on any OS.

    python3 scripts/generate_icns.py            # writes assets/img/icon.icns
    python3 scripts/generate_icns.py --check    # fails if it is out of date

An .icns is the "icns" magic and total length, then one (type, length, PNG)
element per size; macOS 10.7+ reads PNG data in every element used here.
Regenerate after scripts/generate_icons_from_svg.sh changes the PNGs.
"""

import struct
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
IMG = ROOT / "assets" / "img"
OUT = IMG / "icon.icns"

# (element type, source PNG): 1x sizes, then the @2x variants.
ELEMENTS = [
    (b"icp4", "icon_16x.png"),  # 16x16
    (b"icp5", "icon_32x.png"),  # 32x32
    (b"icp6", "icon_64x.png"),  # 64x64
    (b"ic07", "icon_128x.png"),  # 128x128
    (b"ic08", "icon_256x.png"),  # 256x256
    (b"ic09", "icon_512x.png"),  # 512x512
    (b"ic11", "icon_32x.png"),  # 16x16@2x
    (b"ic12", "icon_64x.png"),  # 32x32@2x
    (b"ic13", "icon_256x.png"),  # 128x128@2x
    (b"ic14", "icon_512x.png"),  # 256x256@2x
    (b"ic10", "icon_1024x.png"),  # 512x512@2x
]

PNG_MAGIC = b"\x89PNG\r\n\x1a\n"


def build() -> bytes:
    body = b""
    for kind, name in ELEMENTS:
        data = (IMG / name).read_bytes()
        if not data.startswith(PNG_MAGIC):
            sys.exit(f"{name}: not a PNG")
        body += kind + struct.pack(">I", 8 + len(data)) + data
    return b"icns" + struct.pack(">I", 8 + len(body)) + body


def main() -> None:
    icns = build()
    if sys.argv[1:] == ["--check"]:
        if not OUT.exists() or OUT.read_bytes() != icns:
            sys.exit(f"{OUT.relative_to(ROOT)} is out of date: run scripts/generate_icns.py")
        return
    OUT.write_bytes(icns)
    print(f"wrote {OUT.relative_to(ROOT)} ({len(icns)} bytes)")


if __name__ == "__main__":
    main()
