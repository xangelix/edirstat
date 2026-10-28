//! Docker disk-attribution engine.
//!
//! Disk-only inventory of a Docker `overlay2` data root — no daemon socket.
//! Metadata (`layerdb`, `imagedb`, `repositories.json`, `config.v2.json`) is
//! parsed directly; sizes come from `layerdb` `size` files where available
//! and from targeted, cancel-aware walks for unreferenced layers, volumes,
//! and the build cache. Roots using the containerd image store yield a
//! partial inventory (containers, logs, volumes, build cache), since image
//! metadata there lives in bolt databases. On macOS/Windows, Docker Desktop
//! keeps its data inside a VM disk image, so detection reports opaque
//! [`VmDisk`] entries instead.

use std::{
    collections::{HashMap, HashSet},
    ffi::OsString,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::Arc,
    sync::atomic::{AtomicBool, Ordering},
};

#[cfg(any(target_os = "linux", windows))]
use edirstat_core::docker::{DataRoot, RootScope};
#[cfg(any(target_os = "macos", target_os = "windows"))]
use edirstat_core::docker::{VmBackend, VmDisk};
use edirstat_core::{
    EdirstatError,
    arena::FileArenaSnapshot,
    docker::{
        ContainerInfo, DockerCollector, DockerEnvironment, DockerInventory, ImageDeletion,
        ImageInfo, InventorySource, InventoryTotals, InventoryWarning, LayerInfo, StorageKind,
        VolumeInfo,
    },
    extensions::{EXT_DOCKER_INVENTORY, EXT_DOCKER_INVENTORY_VERSION},
    state::SharedState,
};
use serde_json::Value;

/// Container json logs larger than this earn a [`InventoryWarning::LargeContainerLog`].
const LARGE_LOG_THRESHOLD_BYTES: u64 = 50 * 1024 * 1024;

/// Native [`DockerCollector`] implementation backed by filesystem metadata.
pub struct NativeDockerCollector {
    /// Variables read in place of the process environment (empty outside
    /// tests), so tests never mutate it: unsound while other test threads run
    /// under `cargo test`.
    env_overrides: Vec<(&'static str, OsString)>,
    /// Probe the system-wide roots (`daemon.json`'s data-root,
    /// `/var/lib/docker`, `/var/lib/containerd`). Tests turn this off so only
    /// their fixtures are discovered, whatever Docker setup the host has.
    #[cfg(target_os = "linux")]
    system_roots: bool,
}

impl Default for NativeDockerCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl NativeDockerCollector {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            env_overrides: Vec::new(),
            #[cfg(target_os = "linux")]
            system_roots: true,
        }
    }

    /// Environment lookup for daemon-socket and data-root discovery.
    fn env_var(&self, key: &str) -> Option<OsString> {
        self.env_overrides
            .iter()
            .find(|(k, _)| *k == key)
            .map_or_else(|| std::env::var_os(key), |(_, value)| Some(value.clone()))
    }

    /// Build the full inventory from one data root. Separate from
    /// [`DockerCollector::collect_inventory`] so tests can target fixture
    /// trees that are not at a well-known path.
    ///
    /// # Errors
    /// Returns `Err` if the root does not exist, is not an overlay2 layout,
    /// or `cancel` is set before/during collection. An existing-but-unreadable
    /// root is not an error: it yields an empty inventory carrying a
    /// [`InventoryWarning::RootUnreadable`] warning.
    fn collect_from_root(
        root: &Path,
        cancel: &AtomicBool,
    ) -> Result<DockerInventory, EdirstatError> {
        check_cancel(cancel)?;

        // An existing but unlistable root (e.g. `/var/lib/docker` at mode
        // 710 without root rights) is reported, not fatal.
        if let Err(e) = fs::read_dir(root) {
            return if e.kind() == ErrorKind::NotFound {
                Err(EdirstatError::Io(e))
            } else {
                Ok(unreadable_inventory(root))
            };
        }

        let image_dir = root.join("image/overlay2");
        if !image_dir.is_dir() {
            return Err(EdirstatError::Io(std::io::Error::new(
                ErrorKind::InvalidData,
                format!("{} is not an overlay2 Docker data root", root.display()),
            )));
        }
        if fs::read_dir(&image_dir).is_err() {
            return Ok(unreadable_inventory(&image_dir));
        }

        let mut warnings: Vec<InventoryWarning> = Vec::new();

        let layers = parse_layerdb(&image_dir.join("layerdb"), cancel, &mut warnings)?;
        let mounts = parse_layerdb_mounts(&image_dir.join("layerdb"), cancel, &mut warnings)?;
        let mount_cache_ids: HashSet<&str> = mounts
            .values()
            .flat_map(|m| [&m.mount_id, &m.init_id])
            .map(String::as_str)
            .collect();

        let raw_images = parse_images(&image_dir.join("imagedb/content"), cancel, &mut warnings)?;
        let tags = parse_repositories(&image_dir.join("repositories.json"), &mut warnings);

        // Assemble images, resolving diff_ids to cache IDs and attributing
        // shared layers (referenced by more than one image).
        let mut cache_ref_counts: HashMap<&str, u32> = HashMap::new();
        let mut image_layers: Vec<(String, Vec<(String, u64)>)> =
            Vec::with_capacity(raw_images.len());
        for raw in &raw_images {
            let mut layers_of_image: Vec<(String, u64)> = Vec::with_capacity(raw.diff_ids.len());
            for diff_id in &raw.diff_ids {
                match layers.by_diff_id.get(diff_id) {
                    Some(&idx) => {
                        let entry = &layers.entries[idx];
                        *cache_ref_counts.entry(entry.cache_id.as_str()).or_insert(0) += 1;
                        layers_of_image.push((entry.cache_id.clone(), entry.size));
                    }
                    None => warnings.push(InventoryWarning::MetadataParse {
                        detail: format!(
                            "image {} references layer {diff_id} missing from layerdb",
                            raw.id
                        ),
                    }),
                }
            }
            image_layers.push((raw.id.clone(), layers_of_image));
        }

        let mut images: Vec<ImageInfo> = Vec::with_capacity(raw_images.len());
        let mut image_ids_of_cache: HashMap<String, Vec<String>> = HashMap::new();
        for (raw, layers_of_image) in raw_images.iter().zip(image_layers) {
            let (id, layer_pairs) = layers_of_image;
            let mut size_bytes = 0u64;
            let mut shared_bytes = 0u64;
            let mut layer_cache_ids = Vec::with_capacity(layer_pairs.len());
            for (cache_id, size) in &layer_pairs {
                size_bytes = size_bytes.saturating_add(*size);
                if cache_ref_counts
                    .get(cache_id.as_str())
                    .is_some_and(|c| *c > 1)
                {
                    shared_bytes = shared_bytes.saturating_add(*size);
                }
                image_ids_of_cache
                    .entry(cache_id.clone())
                    .or_default()
                    .push(id.clone());
                layer_cache_ids.push(cache_id.clone());
            }
            let mut image_tags = tags.get(&id).cloned().unwrap_or_default();
            image_tags.sort();
            images.push(ImageInfo {
                id,
                tags: image_tags,
                size_bytes,
                shared_bytes,
                layer_count: u32::try_from(layer_cache_ids.len()).unwrap_or(u32::MAX),
                ref_count: 0, // patched from container configs below
                layer_cache_ids,
                created: raw.created,
            });
        }
        images.sort_by(|a, b| a.id.cmp(&b.id));

        // One LayerInfo per distinct cache ID: layerdb entries first, then
        // mount/init IDs with no layerdb entry, then on-disk orphans.
        let mut layer_infos: Vec<LayerInfo> = Vec::new();
        let mut known_cache_ids: HashSet<String> = HashSet::new();
        for entry in &layers.entries {
            if !known_cache_ids.insert(entry.cache_id.clone()) {
                continue;
            }
            let mut image_ids = image_ids_of_cache
                .get(&entry.cache_id)
                .cloned()
                .unwrap_or_default();
            image_ids.sort();
            layer_infos.push(LayerInfo {
                cache_id: entry.cache_id.clone(),
                size_bytes: entry.size,
                path: root.join("overlay2").join(&entry.cache_id).join("diff"),
                image_ids,
                container_mount: mount_cache_ids.contains(entry.cache_id.as_str()),
                referenced: true,
            });
        }
        for mount in mounts.values() {
            for cache_id in [&mount.mount_id, &mount.init_id] {
                if !known_cache_ids.insert(cache_id.clone()) {
                    continue;
                }
                layer_infos.push(LayerInfo {
                    cache_id: cache_id.clone(),
                    size_bytes: 0,
                    path: root.join("overlay2").join(cache_id).join("diff"),
                    image_ids: Vec::new(),
                    container_mount: true,
                    referenced: true,
                });
            }
        }
        collect_unreferenced_layers(
            root,
            &known_cache_ids,
            cancel,
            &mut layer_infos,
            &mut warnings,
        )?;
        layer_infos.sort_by(|a, b| a.cache_id.cmp(&b.cache_id));

        // Reference counts come from the container configs: `Image`
        // references an image id (prefix already stripped by
        // `collect_containers`), `Mounts` reference volumes — anonymous hex
        // names included, bind mounts excluded by the `Type` filter.
        let mut volume_mounts: HashMap<String, u32> = HashMap::new();
        let containers = collect_containers(
            root,
            &mounts,
            &layers,
            cancel,
            &mut warnings,
            &mut volume_mounts,
        )?;
        let mut image_ref_counts: HashMap<&str, u32> = HashMap::new();
        for container in &containers {
            if let Some(image_id) = container.image_id.as_deref() {
                *image_ref_counts.entry(image_id).or_insert(0) += 1;
            }
        }
        for image in &mut images {
            image.ref_count = image_ref_counts
                .get(image.id.as_str())
                .copied()
                .unwrap_or(0);
        }
        let mut volumes = collect_volumes(root, cancel)?;
        for volume in &mut volumes {
            volume.ref_count = volume_mounts.get(&volume.name).copied().unwrap_or(0);
        }

        let buildkit_dir = root.join("buildkit");
        let build_cache_bytes = if buildkit_dir.is_dir() {
            dir_size(&buildkit_dir, cancel)?
        } else {
            0
        };

        let mut inventory = DockerInventory {
            root: Some(root.to_path_buf()),
            images,
            layers: layer_infos,
            containers,
            volumes,
            build_cache_bytes,
            collected_at: now_epoch(),
            source: InventorySource::DiskOverlay2,
            totals: InventoryTotals::default(),
            warnings,
        };
        inventory.compute_totals();
        Ok(inventory)
    }

    /// Partial inventory for a root using the containerd image store. Image
    /// and layer attribution lives in containerd's bolt metadata (not
    /// disk-parseable), but container configs/logs, volumes, and the build
    /// cache keep the classic layout and are collected as usual.
    fn collect_containerd_partial(
        root: &Path,
        cancel: &AtomicBool,
    ) -> Result<DockerInventory, EdirstatError> {
        check_cancel(cancel)?;
        if let Err(e) = fs::read_dir(root) {
            return if e.kind() == ErrorKind::NotFound {
                Err(EdirstatError::Io(e))
            } else {
                Ok(unreadable_inventory(root))
            };
        }

        let mut warnings = vec![InventoryWarning::ContainerdStorePartial];
        let no_layers = ParsedLayers {
            entries: Vec::new(),
            by_diff_id: HashMap::new(),
            by_cache_id: HashMap::new(),
        };
        let mut volume_mounts: HashMap<String, u32> = HashMap::new();
        let containers = collect_containers(
            root,
            &HashMap::new(),
            &no_layers,
            cancel,
            &mut warnings,
            &mut volume_mounts,
        )?;
        let mut volumes = collect_volumes(root, cancel)?;
        for volume in &mut volumes {
            volume.ref_count = volume_mounts.get(&volume.name).copied().unwrap_or(0);
        }

        let buildkit_dir = root.join("buildkit");
        let build_cache_bytes = if buildkit_dir.is_dir() {
            dir_size(&buildkit_dir, cancel)?
        } else {
            0
        };

        let mut inventory = DockerInventory {
            root: Some(root.to_path_buf()),
            containers,
            volumes,
            build_cache_bytes,
            collected_at: now_epoch(),
            source: InventorySource::DiskContainerdPartial,
            warnings,
            ..DockerInventory::default()
        };
        inventory.compute_totals();
        Ok(inventory)
    }

    /// Preferred collection strategy: ask the daemon API first when a socket
    /// is discoverable (full image attribution, works with containerd image
    /// stores), enriching the result from disk; on any API failure fall back
    /// to the pure disk parse. Every path stamps `collected_at`/`source`.
    ///
    /// # Errors
    /// Returns `Err` only when the disk fallback also fails (see
    /// [`Self::collect_disk`]).
    pub fn collect_preferred(&self, cancel: &AtomicBool) -> Result<DockerInventory, EdirstatError> {
        #[cfg(unix)]
        {
            if let Some(socket) = super::docker_api::discover_socket(|key| self.env_var(key)) {
                match super::docker_api::collect_system_df(&socket, cancel) {
                    Ok(mut inventory) => {
                        debug_log("docker inventory collected via daemon API");
                        // The API omits json log sizes and sometimes volume
                        // sizes; an accessible data root fills both in.
                        let environment = self.detect_environment();
                        let primary = environment.primary_root();
                        if let Some(root) = primary.filter(|root| root.accessible) {
                            super::docker_api::enrich_from_disk(&mut inventory, &root.path, cancel);
                        }
                        inventory.root = primary.map(|root| root.path.clone());
                        return Ok(inventory);
                    }
                    Err(e) => debug_log(&format!(
                        "daemon API collection failed ({e}); falling back to disk parse"
                    )),
                }
            }
        }
        self.collect_disk(cancel)
    }

    /// Disk-only collection: parse the primary data root's metadata without
    /// contacting the daemon (full inventory for `overlay2`, partial for the
    /// containerd image store). This is the fallback for
    /// [`Self::collect_preferred`] and the direct entry point for tests.
    ///
    /// # Errors
    /// Returns `Err` when no data root exists or its layout is unsupported.
    pub fn collect_disk(&self, cancel: &AtomicBool) -> Result<DockerInventory, EdirstatError> {
        let environment = self.detect_environment();
        let Some(root) = environment.primary_root() else {
            return Err(EdirstatError::Io(std::io::Error::new(
                ErrorKind::NotFound,
                "no Docker data root found on this system",
            )));
        };
        if !root.accessible {
            return Ok(unreadable_inventory(&root.path));
        }
        match root.kind {
            StorageKind::Overlay2 => Self::collect_from_root(&root.path, cancel),
            StorageKind::Containerd => Self::collect_containerd_partial(&root.path, cancel),
            other => Err(EdirstatError::Io(std::io::Error::new(
                ErrorKind::InvalidData,
                format!(
                    "Docker data root {} uses unsupported storage {other:?} (overlay2 only)",
                    root.path.display(),
                ),
            ))),
        }
    }
}

impl DockerCollector for NativeDockerCollector {
    fn detect_environment(&self) -> DockerEnvironment {
        let mut env = DockerEnvironment::default();
        #[cfg(target_os = "linux")]
        detect_linux_roots(&mut env, |key| self.env_var(key), self.system_roots);
        #[cfg(target_os = "macos")]
        detect_macos_vm_disks(&mut env, |key| self.env_var(key));
        #[cfg(windows)]
        detect_windows(&mut env, |key| self.env_var(key));
        // A reachable daemon counts as Docker presence even when no data
        // root is visible in this filesystem namespace (e.g. a socket
        // bind-mounted into a container).
        #[cfg(unix)]
        {
            env.daemon_socket = super::docker_api::discover_socket(|key| self.env_var(key));
            if let Some(socket) = &env.daemon_socket {
                debug_log(&format!("daemon socket discovered: {}", socket.display()));
            }
        }
        env
    }

    fn collect_inventory(&self, cancel: &AtomicBool) -> Result<DockerInventory, EdirstatError> {
        self.collect_preferred(cancel)
    }

    fn delete_image(&self, image_id: &str, force: bool) -> Result<ImageDeletion, EdirstatError> {
        #[cfg(unix)]
        {
            // Deletion is daemon-only: same socket discovery as the API
            // collection path, and no disk fallback — removing daemon state
            // from under a live daemon is never attempted.
            let Some(socket) = super::docker_api::discover_socket(|key| self.env_var(key)) else {
                return Err(EdirstatError::Io(std::io::Error::new(
                    ErrorKind::NotFound,
                    "no Docker daemon socket found; image deletion requires the daemon",
                )));
            };
            // The trait carries no cancel flag; the request is one quick
            // round trip bounded by the API layer's socket timeout, which
            // checks its flag exactly like the GET path does.
            let cancel = AtomicBool::new(false);
            let deletion = super::docker_api::delete_image(&socket, image_id, force, &cancel)?;
            Ok(deletion)
        }
        #[cfg(not(unix))]
        {
            let _ = (image_id, force);
            Err(EdirstatError::Io(std::io::Error::new(
                ErrorKind::Unsupported,
                "image deletion requires a local Docker daemon socket (unix only)",
            )))
        }
    }

    fn delete_container(&self, container_id: &str, force: bool) -> Result<(), EdirstatError> {
        #[cfg(unix)]
        {
            // Daemon-only, same as delete_image: socket discovery identical
            // to the API collection path, never a disk fallback.
            let Some(socket) = super::docker_api::discover_socket(|key| self.env_var(key)) else {
                return Err(EdirstatError::Io(std::io::Error::new(
                    ErrorKind::NotFound,
                    "no Docker daemon socket found; container deletion requires the daemon",
                )));
            };
            let cancel = AtomicBool::new(false);
            super::docker_api::delete_container(&socket, container_id, force, &cancel)?;
            Ok(())
        }
        #[cfg(not(unix))]
        {
            let _ = (container_id, force);
            Err(EdirstatError::Io(std::io::Error::new(
                ErrorKind::Unsupported,
                "container deletion requires a local Docker daemon socket (unix only)",
            )))
        }
    }

    fn delete_volume(&self, name: &str, force: bool) -> Result<(), EdirstatError> {
        #[cfg(unix)]
        {
            // Daemon-only, same as delete_image: socket discovery identical
            // to the API collection path, never a disk fallback.
            let Some(socket) = super::docker_api::discover_socket(|key| self.env_var(key)) else {
                return Err(EdirstatError::Io(std::io::Error::new(
                    ErrorKind::NotFound,
                    "no Docker daemon socket found; volume deletion requires the daemon",
                )));
            };
            let cancel = AtomicBool::new(false);
            super::docker_api::delete_volume(&socket, name, force, &cancel)?;
            Ok(())
        }
        #[cfg(not(unix))]
        {
            let _ = (name, force);
            Err(EdirstatError::Io(std::io::Error::new(
                ErrorKind::Unsupported,
                "volume deletion requires a local Docker daemon socket (unix only)",
            )))
        }
    }
}

/// Linux data-root candidates: an explicit daemon `data-root` first (users
/// relocate it precisely because Docker fills the system disk), then the
/// system default path, then the rootless one.
#[cfg(target_os = "linux")]
fn detect_linux_roots(
    env: &mut DockerEnvironment,
    var: impl Fn(&str) -> Option<OsString>,
    system_roots: bool,
) {
    let mut candidates: Vec<(PathBuf, RootScope)> = Vec::new();
    if system_roots {
        if let Some(data_root) = daemon_data_root() {
            debug_log(&format!("daemon.json data-root: {}", data_root.display()));
            candidates.push((data_root, RootScope::System));
        }
        candidates.push((PathBuf::from("/var/lib/docker"), RootScope::System));
        // With the containerd image store, image bytes live in containerd's own
        // root (often shared with other containerd consumers such as k8s).
        candidates.push((PathBuf::from("/var/lib/containerd"), RootScope::System));
    }

    if let Some(base) = var("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())
        .or_else(|| var("HOME").map(|home| PathBuf::from(home).join(".local/share")))
    {
        candidates.push((base.join("docker"), RootScope::Rootless));
        candidates.push((base.join("containerd"), RootScope::Rootless));
    }

    let mut seen = HashSet::new();
    for (path, scope) in candidates {
        if !seen.insert(path.clone()) || !path.is_dir() {
            continue;
        }
        env.data_roots.push(classify_root(&path, scope));
    }
}

/// The daemon's configured `data-root` (or legacy `graph`) from
/// `/etc/docker/daemon.json`, if present and parseable.
#[cfg(target_os = "linux")]
fn daemon_data_root() -> Option<PathBuf> {
    let content = fs::read_to_string("/etc/docker/daemon.json").ok()?;
    let root = parse_daemon_data_root(&content);
    if root.is_none() {
        debug_log("daemon.json present but has no data-root/graph key");
    }
    root
}

/// Extract `data-root` (preferred) or the legacy `graph` key from daemon.json.
#[cfg(target_os = "linux")]
fn parse_daemon_data_root(json: &str) -> Option<PathBuf> {
    let value: Value = serde_json::from_str(json).ok()?;
    ["data-root", "graph"].iter().find_map(|key| {
        value
            .get(*key)
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .map(PathBuf::from)
    })
}

/// macOS Docker Desktop keeps all state in a VM disk image per VM.
#[cfg(target_os = "macos")]
fn detect_macos_vm_disks(env: &mut DockerEnvironment, var: impl Fn(&str) -> Option<OsString>) {
    use std::os::unix::fs::MetadataExt as _;

    let Some(home) = var("HOME") else {
        return;
    };
    let vms_dir = PathBuf::from(home).join("Library/Containers/com.docker.docker/Data/vms");
    let Ok(entries) = fs::read_dir(&vms_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let disk = entry.path().join("data/Docker.raw");
        if !disk.is_file() {
            continue;
        }
        let Ok(meta) = fs::metadata(&disk) else {
            continue;
        };
        env.vm_disks.push(VmDisk {
            path: disk,
            apparent_bytes: meta.len(),
            allocated_bytes: meta.blocks().saturating_mul(512),
            backend: VmBackend::DockerDesktopMac,
        });
    }
    env.vm_disks.sort_by(|a, b| a.path.cmp(&b.path));
}

/// Windows Docker Desktop (WSL2 backend): `*.vhdx` VM disks plus the
/// daemon's (bolt-based, unparsable) data directory.
#[cfg(windows)]
fn detect_windows(env: &mut DockerEnvironment, var: impl Fn(&str) -> Option<OsString>) {
    let program_data = Path::new(r"C:\ProgramData\Docker");
    if program_data.is_dir() {
        // classify_root recognizes the `windowsfilter/` driver dir.
        env.data_roots
            .push(classify_root(program_data, RootScope::System));
    }

    let Some(local_app_data) = var("LOCALAPPDATA") else {
        return;
    };
    let wsl_dir = PathBuf::from(local_app_data).join("Docker").join("wsl");
    let mut vhdx_files = Vec::new();
    collect_vhdx(&wsl_dir, 0, &mut vhdx_files);
    vhdx_files.sort();
    for path in vhdx_files {
        let (apparent_bytes, allocated_bytes) = vhdx_sizes(&path);
        env.vm_disks.push(VmDisk {
            path,
            apparent_bytes,
            allocated_bytes,
            backend: VmBackend::DockerDesktopWsl2,
        });
    }
}

/// Collect `*.vhdx` files one or two directory levels below `dir`.
#[cfg(windows)]
fn collect_vhdx(dir: &Path, depth: usize, out: &mut Vec<PathBuf>) {
    if depth >= 2 {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if file_type.is_dir() {
            collect_vhdx(&entry.path(), depth + 1, out);
        } else if entry
            .path()
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("vhdx"))
        {
            out.push(entry.path());
        }
    }
}

/// Apparent and host-allocated bytes of a (possibly sparse) `*.vhdx`.
#[cfg(windows)]
#[allow(unsafe_code)]
fn vhdx_sizes(path: &Path) -> (u64, u64) {
    use std::os::windows::ffi::OsStrExt as _;

    use windows_sys::Win32::Storage::FileSystem::GetCompressedFileSizeW;

    let apparent = fs::metadata(path).map_or(0, |meta| meta.len());

    let mut wide: Vec<u16> = path.as_os_str().encode_wide().collect();
    wide.push(0);
    let mut high = 0u32;
    // SAFETY: `wide` is a NUL-terminated UTF-16 buffer valid for the call's
    // duration; `high` is a valid out-pointer to stack storage.
    let low = unsafe { GetCompressedFileSizeW(wide.as_ptr(), &raw mut high) };
    // `u32::MAX` is also a valid low half: the call failed only if it set an
    // error (`last_os_error` reads `GetLastError`; 0 is `ERROR_SUCCESS`).
    if low == u32::MAX && std::io::Error::last_os_error().raw_os_error() != Some(0) {
        return (apparent, apparent);
    }
    (apparent, (u64::from(high) << 32) | u64::from(low))
}

/// Storage-driver marker dirs: a root is classified by the first driver whose
/// content dir (`<driver>/`) or metadata dir (`image/<driver>/`) exists.
/// Only Linux and Windows have classifiable on-disk roots; on macOS Docker
/// Desktop keeps everything inside the VM disk image.
#[cfg(any(target_os = "linux", windows))]
const DRIVER_MARKERS: [(&str, StorageKind); 7] = [
    ("overlay2", StorageKind::Overlay2),
    ("overlay", StorageKind::Overlay),
    ("aufs", StorageKind::Aufs),
    ("btrfs", StorageKind::Btrfs),
    ("zfs", StorageKind::Zfs),
    ("devicemapper", StorageKind::Devicemapper),
    ("vfs", StorageKind::Vfs),
];

/// Classify a data root by its on-disk layout and whether we can list it.
#[cfg(any(target_os = "linux", windows))]
fn classify_root(path: &Path, scope: RootScope) -> DataRoot {
    let kind = DRIVER_MARKERS
        .iter()
        .find(|(driver, _)| path.join(driver).is_dir() || path.join("image").join(driver).is_dir())
        .map_or_else(
            || {
                if path.join("windowsfilter").is_dir() {
                    StorageKind::WindowsFilter
                } else if has_containerd_store(path) {
                    StorageKind::Containerd
                } else {
                    StorageKind::Unknown
                }
            },
            |(_, kind)| *kind,
        );

    let overlay2_meta = path.join("image/overlay2");
    let accessible = fs::read_dir(path).is_ok()
        && (kind != StorageKind::Overlay2
            || !overlay2_meta.is_dir()
            || fs::read_dir(&overlay2_meta).is_ok());

    debug_log(&format!(
        "classify {} -> {kind:?}, accessible={accessible}, entries: {}",
        path.display(),
        top_level_names(path),
    ));

    DataRoot {
        path: path.to_path_buf(),
        kind,
        scope,
        accessible,
    }
}

/// Verbose detection tracing, enabled with `EDIRSTAT_DOCKER_DEBUG=1`.
fn debug_log(msg: &str) {
    if std::env::var_os("EDIRSTAT_DOCKER_DEBUG").is_some() {
        eprintln!("[edirstat docker] {msg}");
    }
}

/// Names of a directory's immediate children (plus `image/`'s), for debug logs.
#[cfg(any(target_os = "linux", windows))]
fn top_level_names(path: &Path) -> String {
    let mut names: Vec<String> = fs::read_dir(path)
        .map(|entries| {
            entries
                .flatten()
                .take(64)
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    if let Ok(entries) = fs::read_dir(path.join("image")) {
        names.extend(
            entries
                .flatten()
                .take(16)
                .map(|e| format!("image/{}", e.file_name().to_string_lossy())),
        );
    }
    names.join(", ")
}

/// True when the root holds a containerd image store: an embedded
/// `containerd/` dir, raw `io.containerd.*` dirs (a containerd root itself),
/// or the containerd-snapshotter integration signature (`rootfs/` mount
/// views plus `image/identity-cache.db`, with no classic driver dirs).
#[cfg(any(target_os = "linux", windows))]
fn has_containerd_store(path: &Path) -> bool {
    if path.join("rootfs").is_dir() && path.join("image/identity-cache.db").is_file() {
        return true;
    }
    let Ok(entries) = fs::read_dir(path) else {
        return false;
    };
    entries.flatten().any(|entry| {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        entry.file_type().is_ok_and(|ft| ft.is_dir())
            && (name == "containerd" || name.starts_with("io.containerd"))
    })
}

/// Empty inventory flagging a root whose metadata cannot be read (likely
/// needs elevated privileges).
fn unreadable_inventory(path: &Path) -> DockerInventory {
    let mut inventory = DockerInventory {
        root: Some(path.to_path_buf()),
        collected_at: now_epoch(),
        ..DockerInventory::default()
    };
    inventory.warnings.push(InventoryWarning::RootUnreadable {
        path: path.to_path_buf(),
    });
    inventory.compute_totals();
    inventory
}

fn check_cancel(cancel: &AtomicBool) -> Result<(), EdirstatError> {
    if cancel.load(Ordering::Relaxed) {
        return Err(EdirstatError::Io(std::io::Error::from(
            ErrorKind::Interrupted,
        )));
    }
    Ok(())
}

/// Epoch seconds now: the `collected_at` stamp for a freshly built inventory.
pub(crate) fn now_epoch() -> u64 {
    u64::from(edirstat_core::time_utils::system_time_to_unix_timestamp(
        std::time::SystemTime::now(),
    ))
}

/// Attach `inventory` to the snapshot currently published in `shared_state`
/// as the `EXT_DOCKER_INVENTORY` extension.
///
/// The snapshot is immutable-`Arc`: clone the struct (cheap — its fields are
/// all `Arc`), preserving any extensions it already carries, swap in the
/// docker payload, and republish.
///
/// # Errors
/// Returns `Err` if the inventory fails to serialize (not expected for this
/// plain-data shape); the published snapshot is left untouched in that case.
pub fn attach_docker_inventory(
    shared_state: &Arc<SharedState>,
    inventory: &DockerInventory,
) -> Result<(), EdirstatError> {
    let payload = inventory.to_extension_payload()?;
    let current = shared_state.current_snapshot.load();
    let mut extensions = current.extensions.clone();
    extensions.insert(
        EXT_DOCKER_INVENTORY,
        EXT_DOCKER_INVENTORY_VERSION,
        Arc::from(payload.as_slice()),
    );
    let updated = FileArenaSnapshot {
        nodes: Arc::clone(&current.nodes),
        string_pool: Arc::clone(&current.string_pool),
        dir_counts: Arc::clone(&current.dir_counts),
        extensions,
    };
    shared_state.store_snapshot(updated);
    Ok(())
}

struct LayerDbEntry {
    /// Contents of the `cache-id` file (the `overlay2/` directory name).
    cache_id: String,
    /// Parsed contents of the `size` file.
    size: u64,
}

struct ParsedLayers {
    entries: Vec<LayerDbEntry>,
    /// diff ID → index into `entries`.
    by_diff_id: HashMap<String, usize>,
    /// cache ID → index into `entries`.
    by_cache_id: HashMap<String, usize>,
}

/// Parse `layerdb/sha256/<chain-id>/` entries. Chain IDs key the database;
/// the `diff` file inside each entry holds the diff ID that image configs
/// reference, so matching happens on file contents, not directory names.
fn parse_layerdb(
    layerdb: &Path,
    cancel: &AtomicBool,
    warnings: &mut Vec<InventoryWarning>,
) -> Result<ParsedLayers, EdirstatError> {
    let mut parsed = ParsedLayers {
        entries: Vec::new(),
        by_diff_id: HashMap::new(),
        by_cache_id: HashMap::new(),
    };
    let sha_dir = layerdb.join("sha256");
    let Ok(entries) = fs::read_dir(&sha_dir) else {
        return Ok(parsed);
    };
    for (i, entry) in entries.flatten().enumerate() {
        if i.is_multiple_of(16) {
            check_cancel(cancel)?;
        }
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }
        let dir = entry.path();
        let read_field = |name: &str| {
            fs::read(dir.join(name))
                .ok()
                .map(|bytes| String::from_utf8_lossy(&bytes).trim().to_owned())
                .filter(|s| !s.is_empty())
        };
        let (Some(diff_id), Some(cache_id), Some(size_text)) = (
            read_field("diff"),
            read_field("cache-id"),
            read_field("size"),
        ) else {
            warnings.push(InventoryWarning::MetadataParse {
                detail: format!(
                    "layerdb entry {} is missing diff/cache-id/size",
                    entry.file_name().display()
                ),
            });
            continue;
        };
        let Ok(size) = size_text.parse::<u64>() else {
            warnings.push(InventoryWarning::MetadataParse {
                detail: format!(
                    "layerdb entry {} has invalid size {size_text:?}",
                    entry.file_name().display()
                ),
            });
            continue;
        };
        let idx = parsed.entries.len();
        parsed.by_diff_id.insert(diff_id.clone(), idx);
        parsed.by_cache_id.insert(cache_id.clone(), idx);
        parsed.entries.push(LayerDbEntry { cache_id, size });
    }
    Ok(parsed)
}

struct MountEntry {
    /// `overlay2/` directory backing the container's read-write layer.
    mount_id: String,
    /// `overlay2/` directory backing the container's init layer.
    init_id: String,
}

/// Parse `layerdb/mounts/<container-id>/` → mount/init overlay2 IDs.
fn parse_layerdb_mounts(
    layerdb: &Path,
    cancel: &AtomicBool,
    warnings: &mut Vec<InventoryWarning>,
) -> Result<HashMap<String, MountEntry>, EdirstatError> {
    let mut mounts = HashMap::new();
    let mounts_dir = layerdb.join("mounts");
    let Ok(entries) = fs::read_dir(&mounts_dir) else {
        return Ok(mounts);
    };
    for (i, entry) in entries.flatten().enumerate() {
        if i.is_multiple_of(16) {
            check_cancel(cancel)?;
        }
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }
        let dir = entry.path();
        let read_field = |name: &str| {
            fs::read(dir.join(name))
                .ok()
                .map(|bytes| String::from_utf8_lossy(&bytes).trim().to_owned())
                .filter(|s| !s.is_empty())
        };
        let (Some(mount_id), Some(init_id)) = (read_field("mount-id"), read_field("init-id"))
        else {
            warnings.push(InventoryWarning::MetadataParse {
                detail: format!(
                    "layerdb mounts entry {} is missing mount-id/init-id",
                    entry.file_name().display()
                ),
            });
            continue;
        };
        let container_id = entry.file_name().to_string_lossy().into_owned();
        mounts.insert(container_id, MountEntry { mount_id, init_id });
    }
    Ok(mounts)
}

struct RawImage {
    id: String,
    diff_ids: Vec<String>,
    created: u64,
}

/// Parse `imagedb/content/sha256/<image-id>` configs: `rootfs.diff_ids` and
/// the `created` timestamp (RFC 3339 → epoch seconds).
fn parse_images(
    content_dir: &Path,
    cancel: &AtomicBool,
    warnings: &mut Vec<InventoryWarning>,
) -> Result<Vec<RawImage>, EdirstatError> {
    let mut images = Vec::new();
    let sha_dir = content_dir.join("sha256");
    let Ok(entries) = fs::read_dir(&sha_dir) else {
        return Ok(images);
    };
    for (i, entry) in entries.flatten().enumerate() {
        if i.is_multiple_of(16) {
            check_cancel(cancel)?;
        }
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_file() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().into_owned();
        let Ok(bytes) = fs::read(entry.path()) else {
            warnings.push(InventoryWarning::MetadataParse {
                detail: format!("image {id}: cannot read config"),
            });
            continue;
        };
        let Ok(json) = serde_json::from_slice::<Value>(&bytes) else {
            warnings.push(InventoryWarning::MetadataParse {
                detail: format!("image {id}: config is not valid JSON"),
            });
            continue;
        };
        let Some(diff_ids) = json
            .pointer("/rootfs/diff_ids")
            .and_then(Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(Value::as_str)
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
        else {
            warnings.push(InventoryWarning::MetadataParse {
                detail: format!("image {id}: config has no rootfs.diff_ids"),
            });
            continue;
        };
        if diff_ids.is_empty() {
            warnings.push(InventoryWarning::MetadataParse {
                detail: format!("image {id}: config has an empty rootfs.diff_ids"),
            });
            continue;
        }
        let created = json
            .get("created")
            .and_then(Value::as_str)
            .and_then(parse_rfc3339_epoch)
            .unwrap_or(0);
        images.push(RawImage {
            id,
            diff_ids,
            created,
        });
    }
    images.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(images)
}

/// Parse `repositories.json` into image ID → `repo:tag` list.
fn parse_repositories(
    path: &Path,
    warnings: &mut Vec<InventoryWarning>,
) -> HashMap<String, Vec<String>> {
    let mut tags: HashMap<String, Vec<String>> = HashMap::new();
    let Ok(bytes) = fs::read(path) else {
        // Absent on fresh daemons; not noteworthy.
        return tags;
    };
    let Ok(json) = serde_json::from_slice::<Value>(&bytes) else {
        warnings.push(InventoryWarning::MetadataParse {
            detail: "repositories.json is not valid JSON".to_owned(),
        });
        return tags;
    };
    let Some(repos) = json.get("Repositories").and_then(Value::as_object) else {
        warnings.push(InventoryWarning::MetadataParse {
            detail: "repositories.json has no Repositories object".to_owned(),
        });
        return tags;
    };
    for (repo, tag_map) in repos {
        let Some(tag_map) = tag_map.as_object() else {
            continue;
        };
        for (tag, image) in tag_map {
            let Some(image) = image.as_str() else {
                continue;
            };
            let id = image.strip_prefix("sha256:").unwrap_or(image);
            tags.entry(id.to_owned())
                .or_default()
                .push(format!("{repo}:{tag}"));
        }
    }
    for tag_list in tags.values_mut() {
        tag_list.sort();
    }
    tags
}

/// Find `overlay2/` directories that neither layerdb nor any container
/// mount references; these are what `docker system prune` would reclaim.
/// Sizes require a targeted walk since layerdb knows nothing about them.
fn collect_unreferenced_layers(
    root: &Path,
    known_cache_ids: &HashSet<String>,
    cancel: &AtomicBool,
    layer_infos: &mut Vec<LayerInfo>,
    warnings: &mut Vec<InventoryWarning>,
) -> Result<(), EdirstatError> {
    let overlay2 = root.join("overlay2");
    let Ok(entries) = fs::read_dir(&overlay2) else {
        return Ok(());
    };
    for (i, entry) in entries.flatten().enumerate() {
        if i.is_multiple_of(8) {
            check_cancel(cancel)?;
        }
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        // Skips the `l` symlink dir and stray files.
        if !file_type.is_dir() {
            continue;
        }
        let cache_id = entry.file_name().to_string_lossy().into_owned();
        if known_cache_ids.contains(&cache_id) {
            continue;
        }
        let diff = entry.path().join("diff");
        if !diff.is_dir() {
            continue;
        }
        let size = dir_size(&diff, cancel)?;
        warnings.push(InventoryWarning::UnreferencedLayer {
            cache_id: cache_id.clone(),
            bytes: size,
        });
        layer_infos.push(LayerInfo {
            cache_id,
            size_bytes: size,
            path: diff,
            image_ids: Vec::new(),
            container_mount: false,
            referenced: false,
        });
    }
    Ok(())
}

/// Parse `containers/<id>/`: name/image/created from `config.v2.json`, the
/// json log size from disk, and the rw-layer size via the layerdb mounts
/// mapping. Volume references (`"Mounts"` entries of `Type` `"volume"`) are
/// tallied into `volume_mounts` by name — anonymous hex names count, bind
/// mounts do not.
fn collect_containers(
    root: &Path,
    mounts: &HashMap<String, MountEntry>,
    layers: &ParsedLayers,
    cancel: &AtomicBool,
    warnings: &mut Vec<InventoryWarning>,
    volume_mounts: &mut HashMap<String, u32>,
) -> Result<Vec<ContainerInfo>, EdirstatError> {
    let mut containers = Vec::new();
    let containers_dir = root.join("containers");
    let Ok(entries) = fs::read_dir(&containers_dir) else {
        return Ok(containers);
    };
    for (i, entry) in entries.flatten().enumerate() {
        if i.is_multiple_of(8) {
            check_cancel(cancel)?;
        }
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().into_owned();
        let dir = entry.path();

        let config_bytes = match fs::read(dir.join("config.v2.json")) {
            Ok(bytes) => bytes,
            Err(e) => {
                warnings.push(InventoryWarning::MetadataParse {
                    detail: format!("container {id}: cannot read config.v2.json ({e})"),
                });
                continue;
            }
        };
        let Ok(json) = serde_json::from_slice::<Value>(&config_bytes) else {
            warnings.push(InventoryWarning::MetadataParse {
                detail: format!("container {id}: config.v2.json is not valid JSON"),
            });
            continue;
        };
        let name = json
            .get("Name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim_start_matches('/')
            .to_owned();
        let name = if name.is_empty() { id.clone() } else { name };
        let image_id = json
            .get("Image")
            .and_then(Value::as_str)
            .and_then(|s| s.strip_prefix("sha256:"))
            .map(str::to_owned);
        let created = json
            .get("Created")
            .and_then(Value::as_str)
            .and_then(parse_rfc3339_epoch)
            .unwrap_or(0);

        // Volume references for the caller's per-volume ref counts.
        if let Some(mounts) = json.get("Mounts").and_then(Value::as_array) {
            for mount in mounts {
                if mount.get("Type").and_then(Value::as_str) == Some("volume")
                    && let Some(name) = mount.get("Name").and_then(Value::as_str)
                {
                    *volume_mounts.entry(name.to_owned()).or_insert(0) += 1;
                }
            }
        }

        let rw_size_bytes = mounts
            .get(&id)
            .and_then(|m| layers.by_cache_id.get(&m.mount_id))
            .map_or(0, |&idx| layers.entries[idx].size);

        let log_bytes = container_log_bytes(root, &id);
        if log_bytes > LARGE_LOG_THRESHOLD_BYTES {
            warnings.push(InventoryWarning::LargeContainerLog {
                container: name.clone(),
                bytes: log_bytes,
            });
        }

        containers.push(ContainerInfo {
            id,
            name,
            image_id,
            rw_size_bytes,
            log_bytes,
            created,
        });
    }
    containers.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(containers)
}

/// Size of `<root>/containers/<id>/<id>-json.log` (0 when absent): the
/// classic disk hog the daemon API never reports.
pub(crate) fn container_log_bytes(root: &Path, container_id: &str) -> u64 {
    let log_path = root
        .join("containers")
        .join(container_id)
        .join(format!("{container_id}-json.log"));
    fs::metadata(&log_path).map_or(0, |meta| meta.len())
}

/// Disk size of one volume's `_data` tree; `None` when the volume is absent
/// or the walk fails. Fills API inventories whose `UsageData` was missing;
/// `collect_volumes` keeps its own error-propagating walk.
/// Used only by the (unix-gated) API enrichment path.
#[cfg(unix)]
pub(crate) fn volume_size_from_disk(root: &Path, name: &str, cancel: &AtomicBool) -> Option<u64> {
    let data = root.join("volumes").join(name).join("_data");
    if data.is_dir() {
        dir_size(&data, cancel).ok()
    } else {
        None
    }
}

/// Size each `volumes/<name>/_data` tree; permission errors mid-walk simply
/// count what is readable.
fn collect_volumes(root: &Path, cancel: &AtomicBool) -> Result<Vec<VolumeInfo>, EdirstatError> {
    let mut volumes = Vec::new();
    let volumes_dir = root.join("volumes");
    let Ok(entries) = fs::read_dir(&volumes_dir) else {
        return Ok(volumes);
    };
    for (i, entry) in entries.flatten().enumerate() {
        if i.is_multiple_of(4) {
            check_cancel(cancel)?;
        }
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let size_bytes = if entry.path().join("_data").is_dir() {
            dir_size(&entry.path().join("_data"), cancel)?
        } else {
            0
        };
        // Best-effort creation time; filesystems without birth-time support
        // yield 0.
        let created = entry
            .metadata()
            .ok()
            .and_then(|meta| meta.created().ok())
            .map_or(0, |time| {
                u64::from(edirstat_core::time_utils::system_time_to_unix_timestamp(
                    time,
                ))
            });
        volumes.push(VolumeInfo {
            name,
            size_bytes,
            ref_count: 0, // patched from container Mounts by the caller
            created,
        });
    }
    volumes.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(volumes)
}

/// Sum of regular-file bytes under `root`, without following symlinks.
/// Unreadable directories are skipped (callers count what is readable);
/// `cancel` aborts with an error between directory reads.
pub(crate) fn dir_size(root: &Path, cancel: &AtomicBool) -> Result<u64, EdirstatError> {
    let mut total = 0u64;
    let mut stack: Vec<PathBuf> = vec![root.to_path_buf()];
    let mut visited = 0u32;
    while let Some(dir) = stack.pop() {
        visited = visited.wrapping_add(1);
        if visited.is_multiple_of(32) {
            check_cancel(cancel)?;
        }
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                stack.push(entry.path());
            } else if file_type.is_file()
                && let Ok(meta) = entry.metadata()
            {
                total = total.saturating_add(meta.len());
            }
        }
    }
    Ok(total)
}

/// Parse an RFC 3339 timestamp (`YYYY-MM-DDTHH:MM:SS[.frac](Z|±hh:mm)`)
/// into epoch seconds, without pulling in a date-time crate.
pub(crate) fn parse_rfc3339_epoch(s: &str) -> Option<u64> {
    fn num(b: &[u8], start: usize, len: usize) -> Option<i64> {
        let mut value = 0i64;
        for i in start..start + len {
            let c = *b.get(i)?;
            if !c.is_ascii_digit() {
                return None;
            }
            value = value * 10 + i64::from(c - b'0');
        }
        Some(value)
    }
    fn two(b: &[u8], start: usize) -> Option<i64> {
        num(b, start, 2)
    }

    let b = s.as_bytes();
    if *b.get(4)? != b'-'
        || *b.get(7)? != b'-'
        || (*b.get(10)? != b'T' && *b.get(10)? != b't')
        || *b.get(13)? != b':'
        || *b.get(16)? != b':'
    {
        return None;
    }
    let year = num(b, 0, 4)?;
    let month = two(b, 5)?;
    let day = two(b, 8)?;
    let hour = two(b, 11)?;
    let minute = two(b, 14)?;
    let second = two(b, 17)?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) || hour > 23 || minute > 59 {
        return None;
    }

    // Optional fractional seconds.
    let mut i = 19usize;
    if b.get(i) == Some(&b'.') {
        i += 1;
        while b.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
    }

    // Time zone offset: `Z` or `±hh:mm`, applied by subtracting.
    let tz_offset = match b.get(i) {
        Some(b'Z' | b'z') => 0,
        Some(sign @ (b'+' | b'-')) => {
            let sign = if *sign == b'-' { -1 } else { 1 };
            sign * (two(b, i + 1)? * 3600 + two(b, i + 4)? * 60)
        }
        _ => return None,
    };

    let days = days_from_civil(year, month, day);
    let epoch = days * 86_400 + hour * 3600 + minute * 60 + second - tz_offset;
    u64::try_from(epoch).ok()
}

/// Days from the Unix epoch to `year-month-day` (proleptic Gregorian),
/// Howard Hinnant's `days_from_civil`.
const fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let adjusted_year = if month <= 2 { year - 1 } else { year };
    let era = if adjusted_year >= 0 {
        adjusted_year
    } else {
        adjusted_year - 399
    } / 400;
    let year_of_era = adjusted_year - era * 400;
    let month_prime = (month + 9) % 12;
    let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `DOCKER_HOST` value pointing at a fake daemon socket.
    #[cfg(target_os = "linux")]
    fn unix_host(sock: &Path) -> OsString {
        OsString::from(format!("unix://{}", sock.display()))
    }

    // Only the Linux-gated discovery tests need an isolated collector.
    #[cfg(target_os = "linux")]
    impl NativeDockerCollector {
        /// A collector isolated from the host: it sees `vars` in place of the
        /// process environment and skips the system-wide data roots, so tests
        /// behave the same with or without Docker installed.
        fn isolated<const N: usize>(vars: [(&'static str, OsString); N]) -> Self {
            Self {
                env_overrides: vars.into(),
                system_roots: false,
            }
        }
    }

    const IMAGE_ONE: &str = "1111aaaa2222bbbb3333cccc4444dddd5555eeee6666ffff0000111122223333";
    const IMAGE_TWO: &str = "9999aaaa8888bbbb7777cccc6666dddd5555eeee4444ffff3333222211110000";
    const CONTAINER: &str = "c0ffee42abc";
    /// Anonymous-volume-style 64-hex name: must be counted exactly like a
    /// named volume when a container config mounts it.
    const ANON_VOL: &str = "a1b2c3d4e5f60718293a4b5c6d7e8f90123456789abcdef0123456789abcdef";

    fn write_str(path: &Path, content: &str) -> Result<(), EdirstatError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    fn write_zeros(path: &Path, len: usize) -> Result<(), EdirstatError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, vec![0u8; len])?;
        Ok(())
    }

    /// Fixture data root: two images sharing layer B, one container (rw +
    /// init mount layers, 600-byte json log), one volume, build cache, and an
    /// orphan overlay2 directory.
    fn build_fixture(root: &Path) -> Result<(), EdirstatError> {
        let layerdb = root.join("image/overlay2/layerdb");
        // Image layers: A (100 B), B (200 B, shared), C (50 B).
        for (chain, diff, cache, size, parent) in [
            ("chaina", "sha256:diffa", "cachea", "100", None),
            ("chainb", "sha256:diffb", "cacheb", "200", Some("chaina")),
            ("chainc", "sha256:diffc", "cachec", "50", Some("chainb")),
        ] {
            let dir = layerdb.join("sha256").join(chain);
            write_str(&dir.join("diff"), diff)?;
            write_str(&dir.join("cache-id"), cache)?;
            write_str(&dir.join("size"), size)?;
            if let Some(parent) = parent {
                write_str(&dir.join("parent"), parent)?;
            }
        }
        // Container rw layer registered in layerdb (cache-id = mount-id).
        let chain_m = layerdb.join("sha256/chainm");
        write_str(&chain_m.join("diff"), "sha256:diffm")?;
        write_str(&chain_m.join("cache-id"), "mountcache")?;
        write_str(&chain_m.join("size"), "30")?;
        write_str(&chain_m.join("parent"), "chainc")?;
        let mounts = layerdb.join("mounts").join(CONTAINER);
        write_str(&mounts.join("mount-id"), "mountcache")?;
        write_str(&mounts.join("init-id"), "initcache")?;

        let image_one = r#"{"architecture":"amd64","created":"2024-01-02T15:04:05Z","rootfs":{"type":"layers","diff_ids":["sha256:diffa","sha256:diffb"]}}"#;
        let image_two = r#"{"created":"2023-06-07T08:09:10.5Z","rootfs":{"type":"layers","diff_ids":["sha256:diffb","sha256:diffc"]}}"#;
        write_str(
            &root.join(format!("image/overlay2/imagedb/content/sha256/{IMAGE_ONE}")),
            image_one,
        )?;
        write_str(
            &root.join(format!("image/overlay2/imagedb/content/sha256/{IMAGE_TWO}")),
            image_two,
        )?;
        let repos = format!(
            r#"{{"Repositories":{{"repo/one":{{"latest":"sha256:{IMAGE_ONE}"}},"repo/two":{{"1.0":"sha256:{IMAGE_TWO}"}}}}}}"#
        );
        write_str(&root.join("image/overlay2/repositories.json"), &repos)?;

        // On-disk layer content (sizes here are irrelevant; layerdb governs).
        write_zeros(&root.join("overlay2/cachea/diff/file_a1"), 100)?;
        write_zeros(&root.join("overlay2/cacheb/diff/file_b1"), 200)?;
        write_zeros(&root.join("overlay2/cachec/diff/file_c1"), 50)?;
        write_zeros(&root.join("overlay2/mountcache/diff/rw.txt"), 30)?;
        write_zeros(&root.join("overlay2/initcache/diff/init.txt"), 5)?;
        fs::create_dir_all(root.join("overlay2/l"))?;
        // Orphan: on disk but absent from layerdb and mounts → reclaimable.
        write_zeros(&root.join("overlay2/orphan/diff/orphan.bin"), 77)?;

        let container_dir = root.join("containers").join(CONTAINER);
        // Mounts: vol1 (named) and ANON_VOL (anonymous-style 64-hex name)
        // are volumes and must be refcounted; the bind mount must not be.
        let config = format!(
            r#"{{"Name":"/test-container","Image":"sha256:{IMAGE_ONE}","Created":"2024-01-03T00:00:00Z","Mounts":[{{"Type":"volume","Name":"vol1"}},{{"Type":"volume","Name":"{ANON_VOL}"}},{{"Type":"bind","Source":"/host","Destination":"/x"}}]}}"#
        );
        write_str(&container_dir.join("config.v2.json"), &config)?;
        write_zeros(&container_dir.join(format!("{CONTAINER}-json.log")), 600)?;

        write_zeros(&root.join("volumes/vol1/_data/a.bin"), 300)?;
        write_zeros(&root.join("volumes/vol1/_data/b.bin"), 20)?;
        // vol0: no container references it (ref 0); ANON_VOL: one anonymous
        // mount (ref 1).
        write_zeros(&root.join("volumes/vol0/_data/c.bin"), 10)?;
        write_zeros(&root.join("volumes").join(ANON_VOL).join("_data/d.bin"), 5)?;

        write_zeros(&root.join("buildkit/some/dir/cache.blob"), 500)?;
        Ok(())
    }

    #[test]
    fn test_collect_inventory_fixture() -> Result<(), EdirstatError> {
        let root = std::env::current_dir()?
            .join("target")
            .join("test_docker_inventory");
        let _ = fs::remove_dir_all(&root);
        build_fixture(&root)?;

        let cancel = AtomicBool::new(false);
        let inventory = NativeDockerCollector::collect_from_root(&root, &cancel)?;

        assert_eq!(inventory.root.as_deref(), Some(root.as_path()));
        assert_eq!(inventory.source, InventorySource::DiskOverlay2);
        assert!(inventory.collected_at > 0);

        // Images: attribution incl. the layer shared between both images.
        assert_eq!(inventory.images.len(), 2);
        let Some(img_one) = inventory.images.iter().find(|i| i.id == IMAGE_ONE) else {
            return Err(missing(&format!("image {IMAGE_ONE}")));
        };
        assert_eq!(img_one.size_bytes, 300);
        assert_eq!(img_one.shared_bytes, 200);
        assert_eq!(img_one.exclusive_bytes(), 100);
        assert_eq!(img_one.tags, vec!["repo/one:latest".to_owned()]);
        assert_eq!(img_one.created, 1_704_207_845);
        assert_eq!(img_one.layer_cache_ids, vec!["cachea", "cacheb"]);
        assert_eq!(img_one.layer_count, 2, "one per layer_cache_ids entry");
        assert_eq!(img_one.ref_count, 1, "the fixture container's Image field");

        let Some(img_two) = inventory.images.iter().find(|i| i.id == IMAGE_TWO) else {
            return Err(missing(&format!("image {IMAGE_TWO}")));
        };
        assert_eq!(img_two.size_bytes, 250);
        assert_eq!(img_two.shared_bytes, 200);
        assert_eq!(img_two.exclusive_bytes(), 50);
        assert_eq!(img_two.tags, vec!["repo/two:1.0".to_owned()]);
        assert_eq!(img_two.created, 1_686_125_350);
        assert_eq!(img_two.layer_cache_ids, vec!["cacheb", "cachec"]);
        assert_eq!(img_two.layer_count, 2);
        assert_eq!(img_two.ref_count, 0, "no container uses image two");

        // Layers: 3 image layers + rw/init mount layers + 1 orphan.
        assert_eq!(inventory.layers.len(), 6);
        let layer = |cache: &str| {
            inventory
                .layers
                .iter()
                .find(|l| l.cache_id == cache)
                .ok_or_else(|| missing(&format!("layer {cache}")))
        };
        let layer_b = layer("cacheb")?;
        assert_eq!(layer_b.size_bytes, 200);
        assert!(!layer_b.container_mount);
        assert!(layer_b.referenced);
        assert_eq!(
            layer_b.image_ids,
            vec![IMAGE_ONE.to_owned(), IMAGE_TWO.to_owned()]
        );

        let rw_layer = layer("mountcache")?;
        assert!(rw_layer.container_mount);
        assert!(rw_layer.referenced);
        assert_eq!(rw_layer.size_bytes, 30);
        assert_eq!(rw_layer.image_ids, Vec::<String>::new());

        let init_layer = layer("initcache")?;
        assert!(init_layer.container_mount);
        assert!(init_layer.referenced);
        assert_eq!(init_layer.size_bytes, 0);

        let orphan = layer("orphan")?;
        assert!(!orphan.referenced);
        assert!(!orphan.container_mount);
        assert_eq!(orphan.size_bytes, 77);
        assert_eq!(orphan.image_ids, Vec::<String>::new());

        // Containers: config.v2.json + json log + rw layer via mounts mapping.
        assert_eq!(inventory.containers.len(), 1);
        let container = &inventory.containers[0];
        assert_eq!(container.id, CONTAINER);
        assert_eq!(container.name, "test-container");
        assert_eq!(container.image_id.as_deref(), Some(IMAGE_ONE));
        assert_eq!(container.rw_size_bytes, 30);
        assert_eq!(container.log_bytes, 600);
        assert_eq!(container.created, 1_704_240_000);

        // Volumes (sorted by name: ANON_VOL < vol0 < vol1) and build cache.
        assert_eq!(inventory.volumes.len(), 3);
        let volume = |name: &str| {
            inventory
                .volumes
                .iter()
                .find(|volume| volume.name == name)
                .ok_or_else(|| missing(&format!("volume {name}")))
        };
        let vol1 = volume("vol1")?;
        assert_eq!(vol1.size_bytes, 320);
        assert_eq!(vol1.ref_count, 1, "one Mounts entry references vol1");
        assert!(vol1.created > 0, "birth time from disk metadata");
        let vol0 = volume("vol0")?;
        assert_eq!(vol0.size_bytes, 10);
        assert_eq!(vol0.ref_count, 0, "no container mounts vol0");
        assert!(vol0.created > 0);
        let anon = volume(ANON_VOL)?;
        assert_eq!(anon.size_bytes, 5);
        assert_eq!(
            anon.ref_count, 1,
            "anonymous hex-named volumes count like named ones"
        );
        assert_eq!(inventory.build_cache_bytes, 500);

        // Totals.
        let totals = &inventory.totals;
        assert_eq!(totals.images_bytes, 550);
        assert_eq!(totals.shared_bytes, 400);
        assert_eq!(totals.unique_image_bytes, 427);
        assert_eq!(totals.container_rw_bytes, 30);
        assert_eq!(totals.log_bytes, 600);
        assert_eq!(totals.volumes_bytes, 335);
        assert_eq!(totals.build_cache_bytes, 500);
        assert_eq!(totals.reclaimable_bytes, 77);

        // Warnings: the orphan layer is flagged, nothing else.
        assert!(inventory
            .warnings
            .iter()
            .any(|w| matches!(w, InventoryWarning::UnreferencedLayer { cache_id, bytes } if cache_id == "orphan" && *bytes == 77)));
        assert!(
            !inventory
                .warnings
                .iter()
                .any(|w| matches!(w, InventoryWarning::LargeContainerLog { .. }))
        );

        let _ = fs::remove_dir_all(&root);
        Ok(())
    }

    #[test]
    fn test_collect_inventory_cancelled() -> Result<(), EdirstatError> {
        let root = std::env::current_dir()?
            .join("target")
            .join("test_docker_cancel");
        let _ = fs::remove_dir_all(&root);
        build_fixture(&root)?;

        let cancel = AtomicBool::new(true);
        let result = NativeDockerCollector::collect_from_root(&root, &cancel);
        assert!(
            matches!(result, Err(EdirstatError::Io(_))),
            "pre-set cancel must abort collection"
        );

        let _ = fs::remove_dir_all(&root);
        Ok(())
    }

    #[test]
    #[cfg(unix)]
    fn test_collect_inventory_root_unreadable() -> Result<(), EdirstatError> {
        use std::os::unix::fs::PermissionsExt as _;

        // If running as root, permission bits do not block reads.
        let is_root = std::process::Command::new("id")
            .arg("-u")
            .output()
            .ok()
            .and_then(|out| String::from_utf8(out.stdout).ok())
            .and_then(|s| s.trim().parse::<u32>().ok())
            .is_some_and(|uid| uid == 0);
        if is_root {
            return Ok(());
        }

        let root = std::env::current_dir()?
            .join("target")
            .join("test_docker_unreadable");
        let _ = fs::remove_dir_all(&root);
        build_fixture(&root)?;

        let mut perms = fs::metadata(&root)?.permissions();
        perms.set_mode(0o000);
        fs::set_permissions(&root, perms)?;

        let cancel = AtomicBool::new(false);
        let inventory = NativeDockerCollector::collect_from_root(&root, &cancel)?;

        let mut restore = fs::metadata(&root)?.permissions();
        restore.set_mode(0o755);
        let _ = fs::set_permissions(&root, restore);
        let _ = fs::remove_dir_all(&root);

        assert!(inventory.images.is_empty());
        assert!(inventory.containers.is_empty());
        assert_eq!(inventory.totals.images_bytes, 0);
        assert!(
            inventory
                .warnings
                .iter()
                .any(|w| matches!(w, InventoryWarning::RootUnreadable { path } if path == &root))
        );
        Ok(())
    }

    #[test]
    fn test_collect_inventory_empty_root() -> Result<(), EdirstatError> {
        let root = std::env::current_dir()?
            .join("target")
            .join("test_docker_empty_root");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("overlay2"))?;
        fs::create_dir_all(root.join("image/overlay2"))?;

        let cancel = AtomicBool::new(false);
        let inventory = NativeDockerCollector::collect_from_root(&root, &cancel)?;

        assert!(inventory.images.is_empty());
        assert!(inventory.layers.is_empty());
        assert!(inventory.containers.is_empty());
        assert!(inventory.volumes.is_empty());
        assert_eq!(inventory.build_cache_bytes, 0);
        assert_eq!(inventory.totals.reclaimable_bytes, 0);
        assert!(inventory.warnings.is_empty());

        let _ = fs::remove_dir_all(&root);
        Ok(())
    }

    #[cfg(any(target_os = "linux", windows))]
    #[test]
    fn test_classify_root_overlay2() -> Result<(), EdirstatError> {
        let root = std::env::current_dir()?
            .join("target")
            .join("test_docker_classify");
        let _ = fs::remove_dir_all(&root);
        build_fixture(&root)?;

        let root_info = classify_root(&root, RootScope::System);
        assert_eq!(root_info.kind, StorageKind::Overlay2);
        assert!(root_info.accessible);

        // The content dir alone is still a strong enough signal.
        fs::remove_dir_all(root.join("image"))?;
        let without_image = classify_root(&root, RootScope::System);
        assert_eq!(without_image.kind, StorageKind::Overlay2);

        // Nothing docker-shaped left at all.
        fs::remove_dir_all(root.join("overlay2"))?;
        let bare = classify_root(&root, RootScope::System);
        assert_eq!(bare.kind, StorageKind::Unknown);

        let _ = fs::remove_dir_all(&root);
        Ok(())
    }

    #[cfg(any(target_os = "linux", windows))]
    #[test]
    fn test_classify_root_storage_drivers() -> Result<(), EdirstatError> {
        let base = std::env::current_dir()?
            .join("target")
            .join("test_docker_classify_drivers");
        let _ = fs::remove_dir_all(&base);

        for (dirs, expected) in [
            (vec!["btrfs"], StorageKind::Btrfs),
            (vec!["image/zfs"], StorageKind::Zfs),
            (vec!["aufs"], StorageKind::Aufs),
            (vec!["devicemapper"], StorageKind::Devicemapper),
            (vec!["vfs"], StorageKind::Vfs),
            (vec!["overlay"], StorageKind::Overlay),
            (vec!["windowsfilter"], StorageKind::WindowsFilter),
            (vec!["containerd"], StorageKind::Containerd),
            (
                vec!["io.containerd.content.v1.content"],
                StorageKind::Containerd,
            ),
            (vec!["volumes", "containers"], StorageKind::Unknown),
        ] {
            let root = base.join(format!("{expected:?}").to_lowercase());
            for dir in &dirs {
                fs::create_dir_all(root.join(dir))?;
            }
            let info = classify_root(&root, RootScope::System);
            assert_eq!(
                info.kind, expected,
                "dirs {dirs:?} should classify as {expected:?}"
            );
        }

        let _ = fs::remove_dir_all(&base);
        Ok(())
    }

    #[cfg(any(target_os = "linux", windows))]
    #[test]
    fn test_classify_root_containerd_snapshotter_signature() -> Result<(), EdirstatError> {
        // Docker ≥29 with the containerd image store: no driver dirs, just
        // `rootfs/` mount views and `image/identity-cache.db`.
        let root = std::env::current_dir()?
            .join("target")
            .join("test_docker_classify_c8d");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("rootfs"))?;
        fs::create_dir_all(root.join("containers"))?;
        write_str(&root.join("image/identity-cache.db"), "bolt-ish")?;

        let info = classify_root(&root, RootScope::System);
        assert_eq!(info.kind, StorageKind::Containerd);
        assert!(info.accessible);

        let _ = fs::remove_dir_all(&root);
        Ok(())
    }

    #[test]
    fn test_collect_containerd_partial() -> Result<(), EdirstatError> {
        let root = std::env::current_dir()?
            .join("target")
            .join("test_docker_containerd_partial");
        let _ = fs::remove_dir_all(&root);
        // Containerd-store root: container configs/logs, volumes and build
        // cache keep the classic layout; there is no layerdb/imagedb.
        let container_dir = root.join("containers").join(CONTAINER);
        let config = r#"{"Name":"/c8d-container","Image":"docker.io/library/img:latest","Created":"2024-05-06T07:08:09Z","Mounts":[{"Type":"volume","Name":"vol1"}]}"#;
        write_str(&container_dir.join("config.v2.json"), config)?;
        write_zeros(&container_dir.join(format!("{CONTAINER}-json.log")), 600)?;
        write_zeros(&root.join("volumes/vol1/_data/blob.bin"), 321)?;
        write_zeros(&root.join("buildkit/cache/entry"), 432)?;
        write_str(&root.join("image/identity-cache.db"), "bolt-ish")?;

        let inventory =
            NativeDockerCollector::collect_containerd_partial(&root, &AtomicBool::new(false))?;

        assert!(inventory.images.is_empty());
        assert!(inventory.layers.is_empty());
        assert!(
            inventory
                .warnings
                .iter()
                .any(|w| matches!(w, InventoryWarning::ContainerdStorePartial))
        );
        assert_eq!(inventory.containers.len(), 1);
        let container = &inventory.containers[0];
        assert_eq!(container.name, "c8d-container");
        // rw layers are containerd active snapshots: not disk-attributable.
        assert_eq!(container.rw_size_bytes, 0);
        assert_eq!(container.log_bytes, 600);
        assert_eq!(inventory.volumes.len(), 1);
        assert_eq!(inventory.volumes[0].size_bytes, 321);
        assert_eq!(
            inventory.volumes[0].ref_count, 1,
            "Mounts refcount works on the containerd partial path too"
        );
        assert!(inventory.volumes[0].created > 0);
        assert_eq!(inventory.build_cache_bytes, 432);
        assert_eq!(inventory.totals.log_bytes, 600);
        assert_eq!(inventory.totals.volumes_bytes, 321);

        let _ = fs::remove_dir_all(&root);
        Ok(())
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_parse_daemon_data_root() {
        assert_eq!(
            parse_daemon_data_root(r#"{"data-root": "/mnt/bigdisk/docker"}"#),
            Some(PathBuf::from("/mnt/bigdisk/docker"))
        );
        // Legacy key, and data-root winning over it.
        assert_eq!(
            parse_daemon_data_root(r#"{"graph": "/legacy/docker"}"#),
            Some(PathBuf::from("/legacy/docker"))
        );
        assert_eq!(
            parse_daemon_data_root(r#"{"graph": "/legacy", "data-root": "/new"}"#),
            Some(PathBuf::from("/new"))
        );
        assert_eq!(
            parse_daemon_data_root(r#"{"storage-driver": "btrfs"}"#),
            None
        );
        assert_eq!(parse_daemon_data_root(r#"{"data-root": ""}"#), None);
        assert_eq!(parse_daemon_data_root("not json"), None);
    }

    #[test]
    fn test_detect_environment_runs() {
        // Probing must never error or panic, with or without Docker present.
        let env = NativeDockerCollector::new().detect_environment();
        for root in &env.data_roots {
            assert!(!root.path.as_os_str().is_empty());
        }
    }

    #[test]
    fn test_parse_rfc3339_epoch() {
        assert_eq!(
            parse_rfc3339_epoch("2024-01-02T15:04:05Z"),
            Some(1_704_207_845)
        );
        assert_eq!(
            parse_rfc3339_epoch("2023-06-07T08:09:10.5Z"),
            Some(1_686_125_350)
        );
        // Ahead of UTC: local time minus the offset.
        assert_eq!(
            parse_rfc3339_epoch("2024-01-02T15:04:05+02:30"),
            Some(1_704_198_845)
        );
        // Behind UTC: local time plus the offset.
        assert_eq!(
            parse_rfc3339_epoch("2024-01-02T15:04:05-05:00"),
            Some(1_704_225_845)
        );
        assert_eq!(
            parse_rfc3339_epoch("2024-01-02t15:04:05z"),
            Some(1_704_207_845)
        );

        for bad in [
            "",
            "not-a-date",
            "2024-01-02 15:04:05Z",
            "2024-13-02T15:04:05Z",
            "2024-00-02T15:04:05Z",
            "2024-01-32T15:04:05Z",
            "2024-01-02T25:04:05Z",
            "2024-01-02T15:04:05",
            "2024-01-02T15:04:05X",
        ] {
            assert_eq!(parse_rfc3339_epoch(bad), None, "should reject {bad:?}");
        }
    }

    /// Full public path exactly as the GUI drives it: env probe → primary
    /// root → disk collection, using an `XDG_DATA_HOME` override so the
    /// fixture is discovered as the rootless data root.
    #[cfg(target_os = "linux")]
    #[test]
    fn test_collect_inventory_via_env_detection() -> Result<(), EdirstatError> {
        let base = std::env::current_dir()?
            .join("target")
            .join("test_docker_env_detect");
        let _ = fs::remove_dir_all(&base);
        let xdg = base.join("xdg");
        build_fixture(&xdg.join("docker"))?;

        let collector = NativeDockerCollector::isolated([("XDG_DATA_HOME", OsString::from(&xdg))]);
        let env = collector.detect_environment();
        let root = env
            .data_roots
            .iter()
            .find(|r| r.scope == RootScope::Rootless)
            .ok_or_else(|| missing("rootless root"))?;
        assert_eq!(root.path, xdg.join("docker"));
        assert_eq!(root.kind, StorageKind::Overlay2);
        assert!(root.accessible);

        let inventory = collector.collect_disk(&AtomicBool::new(false))?;
        assert_eq!(
            inventory.root.as_deref(),
            Some(xdg.join("docker").as_path())
        );
        assert_eq!(inventory.images.len(), 2);
        assert_eq!(inventory.totals.reclaimable_bytes, 77);
        Ok(())
    }

    /// The attach helper must preserve extensions the writer does not
    /// understand while adding the docker payload to the live snapshot.
    #[test]
    fn test_attach_docker_inventory_preserves_existing_extensions() -> Result<(), EdirstatError> {
        let shared = Arc::new(SharedState::new());

        // Publish a snapshot carrying an unknown future extension.
        let initial = shared.current_snapshot.load();
        let mut extensions = edirstat_core::extensions::ExtensionStore::default();
        extensions.insert(u32::from_le_bytes(*b"UNKN"), 9, Arc::from(&b"opaque"[..]));
        shared.store_snapshot(FileArenaSnapshot {
            nodes: Arc::clone(&initial.nodes),
            string_pool: Arc::clone(&initial.string_pool),
            dir_counts: Arc::clone(&initial.dir_counts),
            extensions,
        });
        drop(initial);

        let mut inventory = DockerInventory {
            source: InventorySource::DockerApi,
            ..DockerInventory::default()
        };
        inventory.containers.push(ContainerInfo {
            id: "abc".into(),
            name: "c".into(),
            rw_size_bytes: 5,
            ..ContainerInfo::default()
        });
        inventory.compute_totals();
        attach_docker_inventory(&shared, &inventory)?;

        let snapshot = shared.current_snapshot.load();
        assert_eq!(snapshot.extensions.len(), 2);
        let unknown = snapshot.extensions.get(u32::from_le_bytes(*b"UNKN"));
        assert_eq!(unknown.map(|entry| &*entry.payload), Some(&b"opaque"[..]));
        let decoded = snapshot
            .docker_inventory()
            .ok_or_else(|| missing("docker extension"))?;
        assert_eq!(decoded.source, InventorySource::DockerApi);
        assert_eq!(decoded.containers.len(), 1);
        assert_eq!(decoded.containers[0].rw_size_bytes, 5);
        assert_eq!(decoded.totals.container_rw_bytes, 5);
        Ok(())
    }

    /// Canned `/system/df` for the preferred-collection tests: one image, one
    /// container (`CONTAINER`), and `vol1` with null `UsageData` so the disk
    /// fallback must size it.
    #[cfg(all(unix, target_os = "linux"))]
    fn df_body() -> String {
        r#"{"LayersSize": 100,
            "Images": [{"Id": "sha256:DFIMAGE", "RepoTags": ["app:latest"], "Size": 100, "SharedSize": 0, "Created": 1704207845}],
            "Containers": [{"Id": "CONTAINER", "Names": ["/c"], "ImageID": "sha256:DFIMAGE", "SizeRw": 7, "Created": 1704240000}],
            "Volumes": [{"Name": "vol1", "UsageData": null}],
            "BuildCache": []}"#
            .replace("CONTAINER", CONTAINER)
            .replace("DFIMAGE", "a1b2c3d4e5f60718293a4b5c6d7e8f90123456789abcdef0123456789abcdef")
    }

    #[cfg(all(unix, target_os = "linux"))]
    fn http_ok(body: &str) -> Vec<u8> {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .into_bytes()
    }

    /// One-shot Unix socket server for a canned daemon response.
    #[cfg(all(unix, target_os = "linux"))]
    fn serve_once(name: &str, response: Vec<u8>) -> Result<PathBuf, EdirstatError> {
        use std::io::{Read as _, Write as _};
        use std::os::unix::net::UnixListener;

        let path = super::super::docker_api::test_socket_path(name);
        let _ = fs::remove_file(&path);
        let listener = UnixListener::bind(&path)?;
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut req = Vec::new();
                let mut buf = [0u8; 512];
                while !req.windows(4).any(|w| w == b"\r\n\r\n") {
                    match stream.read(&mut buf) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => req.extend_from_slice(&buf[..n]),
                    }
                }
                let _ = stream.write_all(&response);
                let _ = stream.flush();
            }
        });
        Ok(path)
    }

    /// With a reachable socket, `collect_preferred` uses the API and enriches
    /// it from the disk root (json log size, missing volume size, root field).
    #[cfg(all(unix, target_os = "linux"))]
    #[test]
    fn test_collect_preferred_prefers_api_and_enriches_from_disk() -> Result<(), EdirstatError> {
        let base = std::env::current_dir()?
            .join("target")
            .join("test_docker_pref_api");
        let _ = fs::remove_dir_all(&base);
        let xdg = base.join("xdg");
        build_fixture(&xdg.join("docker"))?;
        let sock = serve_once("api", http_ok(&df_body()))?;

        let collector = NativeDockerCollector::isolated([
            ("DOCKER_HOST", unix_host(&sock)),
            ("XDG_DATA_HOME", OsString::from(&xdg)),
        ]);
        let inventory = collector.collect_preferred(&AtomicBool::new(false))?;
        let _ = fs::remove_file(&sock);

        assert_eq!(inventory.source, InventorySource::DockerApi);
        assert!(inventory.collected_at > 0);
        assert_eq!(
            inventory.root.as_deref(),
            Some(xdg.join("docker").as_path())
        );

        assert_eq!(inventory.images.len(), 1);
        assert_eq!(inventory.images[0].tags, vec!["app:latest".to_owned()]);

        assert_eq!(inventory.containers.len(), 1);
        let container = &inventory.containers[0];
        assert_eq!(container.id, CONTAINER);
        assert_eq!(container.name, "c");
        assert_eq!(container.rw_size_bytes, 7);
        assert_eq!(
            container.log_bytes, 600,
            "json log size must be enriched from the data root"
        );

        assert_eq!(inventory.volumes.len(), 1);
        assert_eq!(
            inventory.volumes[0].size_bytes, 320,
            "null API UsageData must fall back to the disk walk"
        );

        assert_eq!(inventory.totals.log_bytes, 600);
        assert_eq!(inventory.totals.volumes_bytes, 320);
        Ok(())
    }

    /// A socket that accepts but answers with garbage must send
    /// `collect_preferred` falling back to the disk parse.
    #[cfg(all(unix, target_os = "linux"))]
    #[test]
    fn test_collect_preferred_falls_back_to_disk_on_api_failure() -> Result<(), EdirstatError> {
        let base = std::env::current_dir()?
            .join("target")
            .join("test_docker_pref_fallback");
        let _ = fs::remove_dir_all(&base);
        let xdg = base.join("xdg");
        build_fixture(&xdg.join("docker"))?;

        // A listener that slams the connection shut: EOF before any bytes.
        let sock = super::super::docker_api::test_socket_path("pref_fallback");
        let _ = fs::remove_file(&sock);
        let listener = std::os::unix::net::UnixListener::bind(&sock)?;
        std::thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                drop(stream);
            }
        });

        let collector = NativeDockerCollector::isolated([
            ("DOCKER_HOST", unix_host(&sock)),
            ("XDG_DATA_HOME", OsString::from(&xdg)),
        ]);
        let inventory = collector.collect_preferred(&AtomicBool::new(false))?;
        let _ = fs::remove_file(&sock);

        assert_eq!(inventory.source, InventorySource::DiskOverlay2);
        assert!(inventory.collected_at > 0);
        assert_eq!(
            inventory.root.as_deref(),
            Some(xdg.join("docker").as_path())
        );
        assert_eq!(inventory.images.len(), 2);
        assert_eq!(inventory.totals.reclaimable_bytes, 77);
        let _ = fs::remove_dir_all(&base);
        Ok(())
    }

    /// Trait-level delete path against a fake daemon: an explicit
    /// `DOCKER_HOST` wins socket discovery, so this is deterministic even on
    /// hosts with a real Docker.
    #[cfg(all(unix, target_os = "linux"))]
    #[test]
    fn test_delete_image_via_collector_uses_daemon_api() -> Result<(), EdirstatError> {
        let sock = serve_once(
            "delete",
            http_ok(r#"[{"Untagged":"app:latest"},{"Deleted":"sha256:deadbeef0123456789"}]"#),
        )?;

        let deletion = NativeDockerCollector::isolated([("DOCKER_HOST", unix_host(&sock))])
            .delete_image("a1b2c3d4e5f6", false)?;
        let _ = fs::remove_file(&sock);

        assert_eq!(deletion.untagged, vec!["app:latest".to_owned()]);
        assert_eq!(
            deletion.deleted,
            vec!["sha256:deadbeef0123456789".to_owned()]
        );
        Ok(())
    }

    /// A 409 conflict must reach the caller with the daemon's message intact
    /// — the UI shows it verbatim ("image is being used by stopped
    /// container xyz").
    #[cfg(all(unix, target_os = "linux"))]
    #[test]
    fn test_delete_image_via_collector_surfaces_daemon_message() -> Result<(), EdirstatError> {
        let body = r#"{"message":"image is being used by stopped container xyz"}"#;
        let response = format!(
            "HTTP/1.1 409 Conflict\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .into_bytes();
        let sock = serve_once("delete_conflict", response)?;

        let result = NativeDockerCollector::isolated([("DOCKER_HOST", unix_host(&sock))])
            .delete_image("a1b2c3d4e5f6", false);
        let _ = fs::remove_file(&sock);
        let Err(error) = result else {
            return Err(missing("expected delete_image to fail with 409"));
        };
        assert!(
            error
                .to_string()
                .contains("image is being used by stopped container xyz"),
            "daemon message must surface verbatim, got: {error}"
        );
        Ok(())
    }

    /// A container 409 conflict must reach the caller with the daemon's
    /// message intact — the UI shows it verbatim ("container is running:
    /// stop the container before removing or force remove").
    #[cfg(all(unix, target_os = "linux"))]
    #[test]
    fn test_delete_container_via_collector_surfaces_daemon_message() -> Result<(), EdirstatError> {
        let body = r#"{"message":"cannot remove container abc123: container is running: stop the container before removing or force remove"}"#;
        let response = format!(
            "HTTP/1.1 409 Conflict\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .into_bytes();
        let sock = serve_once("ct_conflict", response)?;

        let result = NativeDockerCollector::isolated([("DOCKER_HOST", unix_host(&sock))])
            .delete_container("abc123", false);
        let _ = fs::remove_file(&sock);
        let Err(error) = result else {
            return Err(missing("expected delete_container to fail with 409"));
        };
        assert!(
            error
                .to_string()
                .contains("container is running: stop the container before removing"),
            "daemon message must surface verbatim, got: {error}"
        );
        Ok(())
    }

    /// A volume 409 conflict must reach the caller with the daemon's message
    /// intact ("volume is in use").
    #[cfg(all(unix, target_os = "linux"))]
    #[test]
    fn test_delete_volume_via_collector_surfaces_daemon_message() -> Result<(), EdirstatError> {
        let body =
            r#"{"message":"remove edirstat-test-vol: volume is in use - [\"c0ffee42abc\"]"}"#;
        let response = format!(
            "HTTP/1.1 409 Conflict\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )
        .into_bytes();
        let sock = serve_once("vol_conflict", response)?;

        let result = NativeDockerCollector::isolated([("DOCKER_HOST", unix_host(&sock))])
            .delete_volume("edirstat-test-vol", false);
        let _ = fs::remove_file(&sock);
        let Err(error) = result else {
            return Err(missing("expected delete_volume to fail with 409"));
        };
        assert!(
            error.to_string().contains("volume is in use"),
            "daemon message must surface verbatim, got: {error}"
        );
        Ok(())
    }

    fn missing(what: &str) -> EdirstatError {
        EdirstatError::Io(std::io::Error::new(
            ErrorKind::NotFound,
            format!("fixture missing {what}"),
        ))
    }
}
