# Privacy Policy for eDirStat (Offline / No-Features Distribution)

**Last Updated:** September 6, 2026

---

## 1. Important Scope Notice

> [!IMPORTANT]
> **This Privacy Policy applies SOLELY and EXCLUSIVELY to any distribution of `eDirStat` compiled without features (`--no-default-features`), which includes the official Mac App Store release as well as standalone offline and air-gapped builds.**

### What This Policy Covers
- **Any version of `eDirStat` compiled without features (`--no-default-features`)**, across any operating system or platform.
- **The official Mac App Store release**, which is compiled strictly with `--no-default-features` and distributed under Apple's App Sandbox security model.
- Any custom, embedded, or air-gapped build running with all optional network features disabled.

### What This Policy Does NOT Cover
This policy does **not** govern:
1. **Online-Enabled Builds (e.g., standard GitHub Releases / portable desktop binaries)**: Standard desktop release binaries compiled with the `online` feature enabled include an automatic or user-initiated software update checker. That checker queries the public GitHub Releases API over HTTPS (`api.github.com`) to determine whether a newer version of the software is available.
2. **Upcoming Cloud / SaaS Extensions & Companion Services**: Future extensions, cloud companion plugins, and web services that may provide remote snapshot storage, multi-machine comparison, historical preservation, or synchronization via SaaS. Any such networked services will be governed by their own distinct, separate Privacy Policy and Terms of Service upon public availability.

---

## 2. Zero Data Collection

For any distribution of `eDirStat` compiled without features (including the Mac App Store release):

- **No Personal Information**: We do not collect, solicit, process, or store names, email addresses, phone numbers, physical addresses, or any other personally identifiable information (PII).
- **No Telemetry or Analytics**: There are zero analytics frameworks, usage trackers, behavioral beacons, or telemetry SDKs embedded in the application (e.g., no Google Analytics, Firebase, Mixpanel, Segment, or similar services).
- **No Crash Beacons**: The application does not transmit automated crash reports, performance metrics, or diagnostic stack traces over the network.
- **No Identifiers**: We do not collect, generate, or track device identifiers, hardware serial numbers, MAC addresses, IDFA/IDFV, or persistent UUIDs.
- **No Advertising**: The application is completely free of advertising, ad trackers, cross-site trackers, and marketing cookies.

---

## 3. 100% Local Filesystem Processing

`eDirStat` is architected as a private, local-first disk visualization utility:

- **Ephemeral In-Memory Scanning**: When you scan a drive, volume, or directory, all file traversal, metadata indexing (file sizes, modified timestamps, file counts, and inode attributes), and treemap layout computations are performed **strictly in local device memory (RAM) by your CPU**.
- **No Cloud Offloading**: Filesystem structures, directory paths, file contents, file names, and disk metrics are **never** uploaded, streamed, or mirrored to external servers.
- **Zero Content Inspection**: `eDirStat` inspects only filesystem metadata (file sizes, timestamps, attributes, and file names). It never reads, parses, analyzes, or transmits the underlying content of your personal files, documents, photos, or media.

---

## 4. Sandboxing & Network Isolation

Security and network isolation are built directly into the binary:

- **Compilation Without Networking**: In any build compiled with `--no-default-features`, HTTP client libraries (`reqwest`), asynchronous network runtimes, and background network routines are completely stripped from the compilation graph at compile time. The resulting binary contains no code capable of initiating outbound network connections.
- **App Sandbox Hardening (Mac App Store Release)**: For users installing through Apple's Mac App Store, the application operates under Apple's kernel-enforced App Sandbox:
  - **No Network Entitlements**: The binary's code signature explicitly omits both `com.apple.security.network.client` and `com.apple.security.network.server`. The OS kernel denies all raw socket and network system calls.
  - **User-Directed Folder Access**: Under sandbox constraints, the application cannot freely access arbitrary filesystem paths. Access to directories is granted solely when you explicitly select a directory via the native file picker dialog (`com.apple.security.files.user-selected.read-write`).
  - **Subprocess Lockdown**: The application does not invoke shells (`/bin/sh`, `/bin/bash`, `/bin/zsh`), command interpreters, or arbitrary external processes.

---

## 5. Local Snapshot Files (`.edst.zst`)

`eDirStat` allows users to save and load compressed filesystem snapshots for offline disk analysis:

- **Saved on Local Storage Only**: Any snapshot file (`.edst.zst`) created through the application is saved solely to the local file path that you choose.
- **Under Your Exclusive Control**: Snapshot files remain under your complete ownership and control. They are never sent to external servers or remote endpoints by this edition of the application.

---

## 6. Information Sharing and Disclosure

Because `eDirStat` (No-Features / Mac App Store Release) does not collect or transmit any data:

- We do not share data with third parties.
- We do not sell, rent, monetize, or trade data to data brokers, advertising networks, or commercial entities.
- We have no access to your files, directories, disk statistics, or usage metrics.

---

## 7. Open Source Verification & In-App Transparency

The source code for `eDirStat` is open source and available for public audit under the MIT license. Anyone may inspect the code repository, build configurations, and entitlements to verify that the no-features build contains zero telemetry, zero trackers, and zero network capabilities:

- **Repository**: [https://github.com/xangelix/edirstat](https://github.com/xangelix/edirstat)
- **Manifest / Feature Definitions**: [`crates/edirstat/Cargo.toml`](https://github.com/xangelix/edirstat/blob/main/crates/edirstat/Cargo.toml)
- **App Sandbox Entitlements**: [`sandbox.entitlements`](https://github.com/xangelix/edirstat/blob/main/sandbox.entitlements)

### Accessing Licenses and Privacy Notices in the Application
For complete transparency and user convenience, all third-party legal notices and this Privacy Policy are directly accessible within the running application:
- **Open Source Licenses**: Open the **About** window (by clicking **About** or pressing `F1`) and click **"View Open Source Licenses"**. This opens an offline, embedded viewer displaying the complete license texts, copyright notices, and dependency disclosures without requiring internet access.
- **Latest Privacy Policy**: The **About** window also features an explicit **"Privacy Policy"** link. Clicking this link instructs your operating system's default browser to load the latest live revision of this document directly from the official repository.

---

## 8. Changes to This Privacy Policy

If we make updates to this Privacy Policy (for example, to reflect new regulatory requirements, packaging adjustments, or policy clarifications), the updated document will be published to the official repository with a revised "Last Updated" date. Because the application operates offline without telemetry or user accounts, users can verify the latest terms at any time via the in-app Privacy Policy link.

---

## 9. Contact & Inquiries

If you have questions regarding this Privacy Policy or the security and privacy practices of `eDirStat`, please contact:

- **Author / Developer**: Cody Wyatt Neiman (xangelix)
- **Email**: [neiman@cody.to](mailto:neiman@cody.to)
- **Project Issues & Audits**: [https://github.com/xangelix/edirstat/issues](https://github.com/xangelix/edirstat/issues)
