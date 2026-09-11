+++
title = "eDirStat is Now Available on the Mac App Store"
date = 2026-09-10
description = "We are ecstatic to announce that eDirStat has officially arrived on the Mac App Store! Following full Apple App Review approval, Mac users can now install our blazing-fast, sandboxed disk usage analyzer with one click, automatic updates, and zero telemetry."

[extra]
version = "2.2.0"
badge = "Announcement"
image = "assets/blog/mac-app-store.png"
image_alt = "eDirStat Now Available on the Mac App Store"
+++
**We are thrilled to announce that eDirStat has been officially approved and published on the Apple Mac App Store!**

Bringing eDirStat to the Mac App Store represents a major milestone in our mission to deliver the fastest, most reliable disk space analyzer for modern operating systems. macOS users can now install eDirStat with a single click, enjoy seamless background updates managed directly by macOS, and experience the highest standard of platform security through Apple's App Sandbox.

<div style="text-align: center; margin: 2rem 0;">
    <a href="https://apps.apple.com/us/app/edirstat/id6805133585" target="_blank" rel="noopener noreferrer" class="app-store-badge-btn" aria-label="Download eDirStat on the Mac App Store">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 156 40" width="180" height="46" class="app-store-badge-svg" style="vertical-align: middle; filter: drop-shadow(0 4px 12px rgba(0, 0, 0, 0.4));" role="img" aria-label="Download on the Mac App Store">
            <defs>
                <style>
                    .badge-bg { fill: #000000; stroke: #a6a6a6; stroke-width: 1px; rx: 7px; ry: 7px; }
                    .apple-icon { fill: #ffffff; }
                    .badge-subtext {
                        fill: #ffffff;
                        font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
                        font-size: 8.5px;
                        font-weight: 400;
                        letter-spacing: 0.15px;
                    }
                    .badge-title {
                        fill: #ffffff;
                        font-family: -apple-system, BlinkMacSystemFont, "SF Pro Display", "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
                        font-size: 14.5px;
                        font-weight: 600;
                        letter-spacing: -0.25px;
                    }
                </style>
            </defs>
            <rect class="badge-bg" x="0.5" y="0.5" width="155" height="39" />
            <g class="apple-icon" transform="translate(14, 9) scale(0.92)">
                <path d="M15.22 3.12c.74-.91 1.24-2.18 1.1-3.44-1.06.05-2.36.71-3.12 1.61-.68.78-1.27 2.05-1.11 3.28 1.19.1 2.41-.58 3.13-1.45z"/>
                <path d="M16.32 5.21c-1.72-.1-3.19.97-4.01.97-.82 0-2.04-.92-3.37-.89-1.73.03-3.33 1.01-4.22 2.56-1.81 3.12-.46 7.75 1.3 10.28.86 1.24 1.88 2.62 3.22 2.57 1.29-.05 1.78-.83 3.34-.83 1.55 0 2 .83 3.34.8 1.38-.02 2.25-1.25 3.1-2.48.98-1.43 1.38-2.81 1.4-2.88-.03-.02-2.69-1.03-2.72-4.07-.02-2.54 2.07-3.75 2.17-3.83-1.21-1.77-3.08-2.02-3.55-2.28z"/>
            </g>
            <text class="badge-subtext" x="43" y="14.8">Download on the</text>
            <text class="badge-title" x="43" y="29.8">Mac App Store</text>
        </svg>
    </a>
</div>

> **✨ Highlights of the Mac App Store Release**
>
> - 🍏 **Official Mac App Store Release** — One-click installation and seamless automated updates through the native macOS App Store.
> - 🛡️ **App Sandbox Security** — Kernel-enforced security with zero network entitlements; completely air-gapped from outbound sockets.
> - 🔒 **Zero Telemetry & 100% Offline** — No analytics, no beacons, no cloud uploads, and no crash reporters. Your disk contents remain strictly private on your device.
> - ⚡ **Apple Silicon Native** — Highly optimized Universal binary running with blazing multi-threaded performance on M1, M2, M3, M4, and Intel Macs.
> - 📂 **Drag-and-Drop Scanning** — Simply drag and drop any folder or drive onto eDirStat to grant security-scoped permissions and start scanning immediately.
> - 🔍 **Interactive Treemap Zoom** — Double-click any directory to zoom into its subtree and navigate back via interactive breadcrumbs.
> - 👯 **7-Stage Duplicate Hunter** — Cryptographic multi-stage duplicate identification with sub-block hashing and safety locks.
> - 🌍 **18 Localized Languages** — Full localization with automatic macOS system language detection.

---

## The Mac App Store Experience

While pre-compiled direct binaries have been available on itch.io and GitHub, the Mac App Store edition provides macOS users with the most integrated, secure, and hassle-free experience possible.

### 1. App Sandbox Hardening & Kernel-Enforced Privacy

The Mac App Store build of eDirStat is compiled strictly with `--no-default-features` under Apple's App Sandbox. In this configuration:
- **No Outbound Networking**: The binary's code signature completely omits the `com.apple.security.network.client` and `com.apple.security.network.server` entitlements. The macOS kernel physically denies any socket connection or network activity.
- **Zero Telemetry**: eDirStat has zero third-party tracking libraries, analytics SDKs, or diagnostic beacons. All traversal, size indexing, and treemap packing computations run strictly in RAM on your CPU.
- **Explicit User Consent**: Under sandbox rules, eDirStat accesses only the folders and volumes you explicitly choose to scan via native Cocoa `NSOpenPanel` dialogues or direct drag-and-drop.

For complete details on our security architecture, see our official [Privacy Policy](https://github.com/xangelix/edirstat/blob/main/PRIVACY.md).

### 2. Drag-and-Drop & Native macOS Integration

We designed eDirStat to feel completely at home on macOS:
- **Drag-and-Drop Folders**: Need to analyze a specific external drive, user folder, or cache directory? Simply drag any folder directly from Finder onto the eDirStat window to grant immediate security-scoped scan access.
- **Native Finder Reveal**: Highlight any selected file or directory directly in macOS Finder with native selection via `open -R`.
- **APFS Cloud Placeholder Badges**: eDirStat recognizes Apple APFS `UF_DATALESS` file flags, showing visual `☁` cloud badges and tooltips for files stored in iCloud Drive, OneDrive, or Dropbox that do not consume local disk space.
- **Dark Mode & Crisp Typography**: Fully supports macOS Dark Mode and Light Mode with native typography and crisp vector icons.

### 3. Apple Silicon (ARM64) Performance

eDirStat is built from the ground up in safe, concurrent Rust. Leveraging a lock-free, work-stealing directory walker and zero-copy columnar arena allocations, eDirStat scans hundreds of thousands of files across APFS and HFS+ volumes in seconds. Whether you are running an M1 MacBook Air or an M4 Max Mac Studio, eDirStat delivers near-instantaneous visualization of your disk usage.

### 4. 18 Languages Built In

The Mac App Store edition automatically detects your macOS system language preferences on first launch. It features comprehensive translations across 18 languages worldwide:
- **English**, **Arabic** (`ar-SA`), **Bengali** (`bn-BD`), **Chinese (Simplified)** (`zh-CN`), **Chinese (Traditional)** (`zh-HK`), **Dutch** (`nl-NL`), **French** (`fr-FR`), **German** (`de-DE`), **Hindi** (`hi-IN`), **Italian** (`it-IT`), **Japanese** (`ja-JP`), **Korean** (`ko-KR`), **Polish** (`pl-PL`), **Portuguese** (`pt-BR`), **Russian** (`ru-RU`), **Spanish** (`es-ES`), **Turkish** (`tr-TR`), and **Vietnamese** (`vi-VN`).

---

## Frequently Asked Questions

### What is the difference between the Mac App Store and itch.io versions?

The Mac App Store version runs inside Apple's mandatory App Sandbox, meaning it can only scan files and folders you explicitly grant it access to through the system file dialog — the itch.io version is not sandboxed and can scan any path directly. The App Store version is individually reviewed and approved by Apple and receives automatic updates through the App Store, while the itch.io version is Apple-notarized but distributed directly.

If you purchased the Mac App Store version and need the unrestricted build, email <button type="button" class="support-email-btn" data-contact="c3VwcG9ydEBlZGlyc3RhdC5jb20=" data-subject="TWFjIEFwcCBTdG9yZSBMaWNlbnNlIFZlcmlmaWNhdGlvbg==" title="Click to copy email &amp; open mail app"><span class="email-text">support [at] edirstat.com</span><span class="email-copy-badge"><svg viewBox="0 0 24 24" width="12" height="12" stroke="currentColor" stroke-width="2" fill="none" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg> Copy</span></button> with your receipt and we'll provide the itch.io version at no additional cost — unfortunately, the reverse is not possible, as we cannot issue Mac App Store copies to itch.io customers at this time.

### Why does the Mac App Store version require permission for folders?

Under macOS App Sandbox security rules, sandboxed applications start with zero access to your user files. When you choose a directory via the native open file dialog or drag and drop a folder onto eDirStat, macOS grants security-scoped read permissions strictly for that selected directory hierarchy. Once granted, eDirStat scans your files entirely offline in memory.

### Does the Mac App Store build collect any analytics or telemetry?

**No.** The Mac App Store edition of eDirStat is compiled with zero network entitlements (`com.apple.security.network.client` is omitted from the application signature). The macOS kernel strictly blocks any outbound network sockets. eDirStat contains no analytics SDKs, no tracking beacons, and no crash reporters.

---

## Download eDirStat Today

The Mac App Store edition is available now for all Macs running macOS 12.0 (Monterey) and higher:

<div style="text-align: center; margin: 2.5rem 0;">
    <a href="https://apps.apple.com/us/app/edirstat/id6805133585" target="_blank" rel="noopener noreferrer" class="app-store-badge-btn" aria-label="Download eDirStat on the Mac App Store">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 156 40" width="200" height="51" class="app-store-badge-svg" style="vertical-align: middle; filter: drop-shadow(0 6px 16px rgba(0, 0, 0, 0.5));" role="img" aria-label="Download on the Mac App Store">
            <defs>
                <style>
                    .badge-bg { fill: #000000; stroke: #a6a6a6; stroke-width: 1px; rx: 7px; ry: 7px; }
                    .apple-icon { fill: #ffffff; }
                    .badge-subtext {
                        fill: #ffffff;
                        font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
                        font-size: 8.5px;
                        font-weight: 400;
                        letter-spacing: 0.15px;
                    }
                    .badge-title {
                        fill: #ffffff;
                        font-family: -apple-system, BlinkMacSystemFont, "SF Pro Display", "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
                        font-size: 14.5px;
                        font-weight: 600;
                        letter-spacing: -0.25px;
                    }
                </style>
            </defs>
            <rect class="badge-bg" x="0.5" y="0.5" width="155" height="39" />
            <g class="apple-icon" transform="translate(14, 9) scale(0.92)">
                <path d="M15.22 3.12c.74-.91 1.24-2.18 1.1-3.44-1.06.05-2.36.71-3.12 1.61-.68.78-1.27 2.05-1.11 3.28 1.19.1 2.41-.58 3.13-1.45z"/>
                <path d="M16.32 5.21c-1.72-.1-3.19.97-4.01.97-.82 0-2.04-.92-3.37-.89-1.73.03-3.33 1.01-4.22 2.56-1.81 3.12-.46 7.75 1.3 10.28.86 1.24 1.88 2.62 3.22 2.57 1.29-.05 1.78-.83 3.34-.83 1.55 0 2 .83 3.34.8 1.38-.02 2.25-1.25 3.1-2.48.98-1.43 1.38-2.81 1.4-2.88-.03-.02-2.69-1.03-2.72-4.07-.02-2.54 2.07-3.75 2.17-3.83-1.21-1.77-3.08-2.02-3.55-2.28z"/>
            </g>
            <text class="badge-subtext" x="43" y="14.8">Download on the</text>
            <text class="badge-title" x="43" y="29.8">Mac App Store</text>
        </svg>
    </a>
</div>

For users on Windows and Linux, or those who prefer direct unsandboxed binaries, eDirStat continues to be available on [itch.io](https://xangelix.itch.io/edirstat) and open-source on [GitHub](https://github.com/xangelix/edirstat).

Thank you to everyone who tested early builds, reported issues, and supported our release through Apple App Review!
