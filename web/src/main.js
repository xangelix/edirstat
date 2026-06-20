import './style.css';
import { Chart, registerables } from 'chart.js';
import { 
  createIcons, Folder, File, Zap, Cpu, Shield, Layers, 
  Image, HardDrive, Download, ChevronRight, ChevronDown, 
  Trash2, BarChart2, Eye, Copy, ExternalLink, Database 
} from 'lucide';

// Register all Chart.js components
Chart.register(...registerables);

// --- HELPER: Human Readable Bytes ---
function formatBytes(bytes) {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

// --- MOCK FILESYSTEM DATA ---
const mockData = {
  name: "Root",
  type: "directory",
  sizeBytes: 76163000000,
  path: "/",
  children: [
    {
      name: "Games",
      type: "directory",
      sizeBytes: 30410000000,
      path: "/Games",
      children: [
        {
          name: "Cyberpunk2077",
          type: "directory",
          sizeBytes: 19415000000,
          path: "/Games/Cyberpunk2077",
          children: [
            { name: "archive.archive", type: "file", ext: "archive", sizeBytes: 17500000000, path: "/Games/Cyberpunk2077/archive.archive" },
            { name: "cyberpunk.exe", type: "file", ext: "code", sizeBytes: 580000000, path: "/Games/Cyberpunk2077/cyberpunk.exe" },
            { name: "readme.txt", type: "file", ext: "other", sizeBytes: 120000000, path: "/Games/Cyberpunk2077/readme.txt" },
            {
              name: "mods",
              type: "directory",
              sizeBytes: 1215000000,
              path: "/Games/Cyberpunk2077/mods",
              children: [
                { name: "cyber_hud.lua", type: "file", ext: "code", sizeBytes: 15000000, path: "/Games/Cyberpunk2077/mods/cyber_hud.lua" },
                { name: "hd_textures.archive", type: "file", ext: "archive", sizeBytes: 1200000000, path: "/Games/Cyberpunk2077/mods/hd_textures.archive" }
              ]
            }
          ]
        },
        {
          name: "SteamLibrary",
          type: "directory",
          sizeBytes: 10600000000,
          path: "/Games/SteamLibrary",
          children: [
            { name: "common_data.bin", type: "file", ext: "archive", sizeBytes: 6100000000, path: "/Games/SteamLibrary/common_data.bin" },
            { name: "steam.ico", type: "file", ext: "image", sizeBytes: 200000000, path: "/Games/SteamLibrary/steam.ico" },
            { name: "depot_cache_01.dat", type: "file", ext: "archive", sizeBytes: 2500000000, path: "/Games/SteamLibrary/depot_cache_01.dat" },
            { name: "depot_cache_02.dat", type: "file", ext: "archive", sizeBytes: 1800000000, path: "/Games/SteamLibrary/depot_cache_02.dat" }
          ]
        },
        {
          name: "Minecraft",
          type: "directory",
          sizeBytes: 395000000,
          path: "/Games/Minecraft",
          children: [
            {
              name: "bin",
              type: "directory",
              sizeBytes: 350000000,
              path: "/Games/Minecraft/bin",
              children: [
                { name: "minecraft.jar", type: "file", ext: "archive", sizeBytes: 350000000, path: "/Games/Minecraft/bin/minecraft.jar" }
              ]
            },
            {
              name: "saves",
              type: "directory",
              sizeBytes: 45000000,
              path: "/Games/Minecraft/saves",
              children: [
                {
                  name: "world_1",
                  type: "directory",
                  sizeBytes: 45000000,
                  path: "/Games/Minecraft/saves/world_1",
                  children: [
                    { name: "level.dat", type: "file", ext: "other", sizeBytes: 45000000, path: "/Games/Minecraft/saves/world_1/level.dat" }
                  ]
                }
              ]
            }
          ]
        }
      ]
    },
    {
      name: "Videos",
      type: "directory",
      sizeBytes: 17370000000,
      path: "/Videos",
      children: [
        { name: "render_final.mp4", type: "file", ext: "video", sizeBytes: 8400000000, path: "/Videos/render_final.mp4" },
        { name: "stream_record.mp4", type: "file", ext: "video", sizeBytes: 3900000000, path: "/Videos/stream_record.mp4" },
        { name: "intro_logo.mov", type: "file", ext: "video", sizeBytes: 100000000, path: "/Videos/intro_logo.mov" },
        { name: "audio_track_stereo.wav", type: "file", ext: "audio", sizeBytes: 120000000, path: "/Videos/audio_track_stereo.wav" },
        {
          name: "Captures",
          type: "directory",
          sizeBytes: 3600000000,
          path: "/Videos/Captures",
          children: [
            { name: "gameplay_01.mp4", type: "file", ext: "video", sizeBytes: 1500000000, path: "/Videos/Captures/gameplay_01.mp4" },
            { name: "gameplay_02.mp4", type: "file", ext: "video", sizeBytes: 2100000000, path: "/Videos/Captures/gameplay_02.mp4" }
          ]
        },
        {
          name: "VFX_Assets",
          type: "directory",
          sizeBytes: 1250000000,
          path: "/Videos/VFX_Assets",
          children: [
            { name: "smoke_simulation.mov", type: "file", ext: "video", sizeBytes: 800000000, path: "/Videos/VFX_Assets/smoke_simulation.mov" },
            { name: "explosion_green.mp4", type: "file", ext: "video", sizeBytes: 450000000, path: "/Videos/VFX_Assets/explosion_green.mp4" }
          ]
        }
      ]
    },
    {
      name: "Projects",
      type: "directory",
      sizeBytes: 12750000000,
      path: "/Projects",
      children: [
        {
          name: "rust-compiler",
          type: "directory",
          sizeBytes: 3200000000,
          path: "/Projects/rust-compiler",
          children: [
            { name: "libstd.so", type: "file", ext: "code", sizeBytes: 3000000000, path: "/Projects/rust-compiler/libstd.so" },
            { name: "main.rs", type: "file", ext: "code", sizeBytes: 200000000, path: "/Projects/rust-compiler/main.rs" }
          ]
        },
        {
          name: "edirstat",
          type: "directory",
          sizeBytes: 1690000000,
          path: "/Projects/edirstat",
          children: [
            { name: "edirstat-binary", type: "file", ext: "code", sizeBytes: 1400000000, path: "/Projects/edirstat/edirstat-binary" },
            { name: "arena.rs", type: "file", ext: "code", sizeBytes: 200000000, path: "/Projects/edirstat/arena.rs" },
            { name: "Cargo.toml", type: "file", ext: "other", sizeBytes: 5000000, path: "/Projects/edirstat/Cargo.toml" },
            {
              name: "src",
              type: "directory",
              sizeBytes: 85000000,
              path: "/Projects/edirstat/src",
              children: [
                { name: "walker.rs", type: "file", ext: "code", sizeBytes: 85000000, path: "/Projects/edirstat/src/walker.rs" }
              ]
            }
          ]
        },
        {
          name: "machine-learning",
          type: "directory",
          sizeBytes: 7200000000,
          path: "/Projects/machine-learning",
          children: [
            { name: "dataset_v4.bin", type: "file", ext: "archive", sizeBytes: 4200000000, path: "/Projects/machine-learning/dataset_v4.bin" },
            { name: "model_weights.bin", type: "file", ext: "archive", sizeBytes: 2800000000, path: "/Projects/machine-learning/model_weights.bin" },
            { name: "train.py", type: "file", ext: "code", sizeBytes: 120000000, path: "/Projects/machine-learning/train.py" },
            { name: "loss_plot.png", type: "file", ext: "image", sizeBytes: 80000000, path: "/Projects/machine-learning/loss_plot.png" }
          ]
        },
        {
          name: "web-frontend",
          type: "directory",
          sizeBytes: 660000000,
          path: "/Projects/web-frontend",
          children: [
            {
              name: "node_modules",
              type: "directory",
              sizeBytes: 200000000,
              path: "/Projects/web-frontend/node_modules",
              children: [
                { name: "vite.js", type: "file", ext: "code", sizeBytes: 90000000, path: "/Projects/web-frontend/node_modules/vite.js" },
                { name: "react.js", type: "file", ext: "code", sizeBytes: 110000000, path: "/Projects/web-frontend/node_modules/react.js" }
              ]
            },
            {
              name: "dist",
              type: "directory",
              sizeBytes: 320000000,
              path: "/Projects/web-frontend/dist",
              children: [
                { name: "bundle.js", type: "file", ext: "code", sizeBytes: 320000000, path: "/Projects/web-frontend/dist/bundle.js" }
              ]
            },
            {
              name: "public",
              type: "directory",
              sizeBytes: 140000000,
              path: "/Projects/web-frontend/public",
              children: [
                { name: "background.png", type: "file", ext: "image", sizeBytes: 140000000, path: "/Projects/web-frontend/public/background.png" }
              ]
            }
          ]
        }
      ]
    },
    {
      name: "Downloads",
      type: "directory",
      sizeBytes: 4035000000,
      path: "/Downloads",
      children: [
        { name: "ubuntu-26.04-desktop.iso", type: "file", ext: "archive", sizeBytes: 2800000000, path: "/Downloads/ubuntu-26.04-desktop.iso" },
        { name: "wallpaper.png", type: "file", ext: "image", sizeBytes: 450000000, path: "/Downloads/wallpaper.png" },
        { name: "music_album.zip", type: "file", ext: "archive", sizeBytes: 250000000, path: "/Downloads/music_album.zip" },
        { name: "blender-4.1.msi", type: "file", ext: "archive", sizeBytes: 410000000, path: "/Downloads/blender-4.1.msi" },
        { name: "album_lossless.flac", type: "file", ext: "audio", sizeBytes: 95000000, path: "/Downloads/album_lossless.flac" },
        { name: "invoice_receipt.pdf", type: "file", ext: "other", sizeBytes: 12000000, path: "/Downloads/invoice_receipt.pdf" },
        { name: "setup_guide.epub", type: "file", ext: "other", sizeBytes: 18000000, path: "/Downloads/setup_guide.epub" }
      ]
    },
    {
      name: "System_Backup",
      type: "directory",
      sizeBytes: 11598000000,
      path: "/System_Backup",
      children: [
        { name: "database_backup.sql", type: "file", ext: "other", sizeBytes: 3500000000, path: "/System_Backup/database_backup.sql" },
        {
          name: "docker_volumes",
          type: "directory",
          sizeBytes: 4800000000,
          path: "/System_Backup/docker_volumes",
          children: [
            {
              name: "postgres",
              type: "directory",
              sizeBytes: 4800000000,
              path: "/System_Backup/docker_volumes/postgres",
              children: [
                { name: "data.bin", type: "file", ext: "archive", sizeBytes: 4800000000, path: "/System_Backup/docker_volumes/postgres/data.bin" }
              ]
            }
          ]
        },
        { name: "registry_dump.reg", type: "file", ext: "other", sizeBytes: 250000000, path: "/System_Backup/registry_dump.reg" },
        { name: "config_recovery.json", type: "file", ext: "other", sizeBytes: 18000000, path: "/System_Backup/config_recovery.json" },
        {
          name: "Photos_2025",
          type: "directory",
          sizeBytes: 3030000000,
          path: "/System_Backup/Photos_2025",
          children: [
            { name: "family_photo.jpg", type: "file", ext: "image", sizeBytes: 450000000, path: "/System_Backup/Photos_2025/family_photo.jpg" },
            { name: "vacation_group.jpg", type: "file", ext: "image", sizeBytes: 380000000, path: "/System_Backup/Photos_2025/vacation_group.jpg" },
            { name: "trip_recap.mp4", type: "file", ext: "video", sizeBytes: 2200000000, path: "/System_Backup/Photos_2025/trip_recap.mp4" }
          ]
        }
      ]
    }
  ]
};

// --- SIMULATOR: Left Panel Tree Explorer ---
function createTreeDOM(node, depth = 0) {
  const nodeDiv = document.createElement('div');
  nodeDiv.className = 'tree-node';
  nodeDiv.setAttribute('data-path', node.path);
  
  const rowDiv = document.createElement('div');
  rowDiv.className = 'tree-node-row';
  rowDiv.style.paddingLeft = `${(depth * 14) + 8}px`;
  
  const leftSide = document.createElement('div');
  leftSide.className = 'tree-node-left';
  
  // Icon
  const icon = document.createElement('i');
  icon.className = 'tree-icon';
  if (node.type === 'directory') {
    icon.setAttribute('data-lucide', 'folder');
    icon.classList.add('tree-folder-icon');
  } else {
    icon.setAttribute('data-lucide', 'file');
    icon.classList.add('tree-file-icon');
  }
  leftSide.appendChild(icon);
  
  // Name
  const nameSpan = document.createElement('span');
  nameSpan.className = 'tree-node-name';
  nameSpan.textContent = node.name;
  leftSide.appendChild(nameSpan);
  
  rowDiv.appendChild(leftSide);
  
  // Size
  const sizeSpan = document.createElement('span');
  sizeSpan.className = 'tree-node-size';
  sizeSpan.textContent = formatBytes(node.sizeBytes);
  rowDiv.appendChild(sizeSpan);
  
  nodeDiv.appendChild(rowDiv);
  
  // Children
  if (node.type === 'directory' && node.children) {
    const childrenContainer = document.createElement('div');
    childrenContainer.className = 'tree-node-children';
    node.children.forEach(child => {
      childrenContainer.appendChild(createTreeDOM(child, depth + 1));
    });
    nodeDiv.appendChild(childrenContainer);
  }
  
  // Event listeners for tree row selection
  rowDiv.addEventListener('click', (e) => {
    e.stopPropagation();
    selectNode(node.path);
  });
  
  rowDiv.addEventListener('mouseenter', () => {
    highlightBlock(node.path);
    updateFooter(node.path, node.sizeBytes);
  });
  
  rowDiv.addEventListener('mouseleave', () => {
    removeBlockHighlight(node.path);
    resetFooterToSelected();
  });
  
  return nodeDiv;
}

// --- SIMULATOR: Right Panel Treemap Generator (Squarified Treemap) ---
// Aspect ratio helper
function worst(rowAreas, L) {
  if (rowAreas.length === 0) return Infinity;
  const sum = rowAreas.reduce((s, val) => s + val, 0);
  if (sum === 0) return Infinity;
  let maxRatio = 0;
  for (let val of rowAreas) {
    const ratio = Math.max((val * L * L) / (sum * sum), (sum * sum) / (val * L * L));
    if (ratio > maxRatio) maxRatio = ratio;
  }
  return maxRatio;
}

function layoutRow(row, L, x, y, w, h, vertical, canvas) {
  const sum = row.reduce((s, n) => s + n.area, 0);
  if (sum === 0) return;
  
  if (vertical) {
    const rowW = sum / h;
    let currentY = y;
    row.forEach(node => {
      const nodeH = node.area / rowW;
      renderNodeOrRecurse(node.node, canvas, x, currentY, rowW, nodeH);
      currentY += nodeH;
    });
  } else {
    const rowH = sum / w;
    let currentX = x;
    row.forEach(node => {
      const nodeW = node.area / rowH;
      renderNodeOrRecurse(node.node, canvas, currentX, y, nodeW, rowH);
      currentX += nodeW;
    });
  }
}

function squarify(nodes, x, y, w, h, canvas) {
  if (nodes.length === 0) return;
  
  const totalSize = nodes.reduce((sum, n) => sum + n.sizeBytes, 0);
  if (totalSize === 0) return;
  
  const scale = (w * h) / totalSize;
  const sortedNodes = nodes
    .map(node => ({ node, area: node.sizeBytes * scale }))
    .sort((a, b) => b.area - a.area);
    
  let currentX = x;
  let currentY = y;
  let currentW = w;
  let currentH = h;
  
  let row = [];
  let index = 0;
  
  while (index < sortedNodes.length) {
    const nextNode = sortedNodes[index];
    if (currentW <= 0.001 || currentH <= 0.001) {
      row.push(nextNode);
      index++;
      continue;
    }
    
    const L = Math.min(currentW, currentH);
    const rowAreas = row.map(r => r.area);
    const currentWorst = worst(rowAreas, L);
    const nextWorst = worst([...rowAreas, nextNode.area], L);
    
    if (nextWorst <= currentWorst) {
      row.push(nextNode);
      index++;
    } else {
      const rowAreaSum = row.reduce((s, r) => s + r.area, 0);
      const isVertical = currentH <= currentW;
      
      layoutRow(row, L, currentX, currentY, currentW, currentH, isVertical, canvas);
      
      if (isVertical) {
        const rowW = rowAreaSum / currentH;
        currentX += rowW;
        currentW -= rowW;
      } else {
        const rowH = rowAreaSum / currentW;
        currentY += rowH;
        currentH -= rowH;
      }
      
      row = [];
    }
  }
  
  if (row.length > 0) {
    const rowAreaSum = row.reduce((s, r) => s + r.area, 0);
    const isVertical = currentH <= currentW;
    layoutRow(row, Math.min(currentW, currentH), currentX, currentY, currentW, currentH, isVertical, canvas);
  }
}

function renderNodeOrRecurse(node, canvas, x, y, w, h) {
  w = Math.max(w, 0.01);
  h = Math.max(h, 0.01);

  if (node.type === 'file') {
    const block = document.createElement('div');
    block.className = `tm-node ext-${node.ext || 'other'}`;
    block.style.left = `${x}%`;
    block.style.top = `${y}%`;
    block.style.width = `${w}%`;
    block.style.height = `${h}%`;
    block.setAttribute('data-path', node.path);
    block.setAttribute('data-size', node.sizeBytes);
    block.setAttribute('data-name', node.name);
    
    block.addEventListener('mouseenter', () => {
      highlightTreeNode(node.path);
      updateFooter(node.path, node.sizeBytes);
    });
    block.addEventListener('mouseleave', () => {
      removeTreeHighlight(node.path);
      resetFooterToSelected();
    });
    block.addEventListener('click', (e) => {
      e.stopPropagation();
      selectNode(node.path);
    });
    
    canvas.appendChild(block);
  } else if (node.type === 'directory' && node.children) {
    squarify(node.children, x, y, w, h, canvas);
  }
}

function renderTreemap(node, canvas, x = 0, y = 0, w = 100, h = 100, isHorizontal = true) {
  renderNodeOrRecurse(node, canvas, x, y, w, h);
}

// --- SELECTING & HIGHLIGHTING COORDINATOR ---
let selectedPath = null;

function selectNode(path) {
  if (selectedPath === path) {
    // Clicked on already selected path -> deselect
    selectedPath = null;
    document.querySelectorAll('.tree-node').forEach(el => el.classList.remove('selected'));
    document.querySelectorAll('.tm-node').forEach(el => el.classList.remove('selected-block'));
    resetFooterToSelected();
    return;
  }

  selectedPath = path;
  
  // Reset previous selections
  document.querySelectorAll('.tree-node').forEach(el => el.classList.remove('selected'));
  document.querySelectorAll('.tm-node').forEach(el => el.classList.remove('selected-block'));
  
  // Select in tree
  const treeEl = document.querySelector(`.tree-node[data-path="${path}"]`);
  if (treeEl) {
    treeEl.classList.add('selected');
    // Scroll into view if needed inside explorer
    treeEl.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
  }
  
  // Select in treemap (leaf or parent group)
  const mapEl = document.querySelector(`.tm-node[data-path="${path}"]`);
  if (mapEl) {
    mapEl.classList.add('selected-block');
  } else {
    // If it's a directory, highlight all its children blocks
    document.querySelectorAll(`.tm-node[data-path^="${path === '/' ? '/' : path + '/'}"]`).forEach(el => {
      el.classList.add('selected-block');
    });
  }
  
  // Update footer permanently
  const node = findNodeByPath(mockData, path);
  if (node) {
    updateFooter(node.path, node.sizeBytes, true);
  }
}

function highlightBlock(path) {
  const mapEl = document.querySelector(`.tm-node[data-path="${path}"]`);
  if (mapEl) {
    mapEl.classList.add('highlighted');
  } else {
    // Directory - highlight descendant blocks
    document.querySelectorAll(`.tm-node[data-path^="${path === '/' ? '/' : path + '/'}"]`).forEach(el => {
      el.classList.add('highlighted');
    });
  }
}

function removeBlockHighlight(path) {
  document.querySelectorAll('.tm-node').forEach(el => el.classList.remove('highlighted'));
}

function highlightTreeNode(path) {
  const treeEl = document.querySelector(`.tree-node[data-path="${path}"]`);
  if (treeEl) {
    const row = treeEl.querySelector('.tree-node-row');
    if (row) row.classList.add('hovered');
  }
}

function removeTreeHighlight(path) {
  document.querySelectorAll('.tree-node-row').forEach(el => el.classList.remove('hovered'));
}

function updateFooter(path, size, isPermanent = false) {
  const pathEl = document.getElementById('sim-footer-path');
  const sizeEl = document.getElementById('sim-footer-size');
  const iconEl = document.getElementById('sim-footer-icon');
  
  if (!pathEl || !sizeEl || !iconEl) return;

  pathEl.textContent = path;
  pathEl.classList.add('active');
  sizeEl.textContent = formatBytes(size);
  
  // Dynamically change icon based on file type
  const node = findNodeByPath(mockData, path);
  if (node && node.type === 'directory') {
    iconEl.setAttribute('data-lucide', 'folder');
  } else {
    iconEl.setAttribute('data-lucide', 'file');
  }
  createIcons({ icons: { Folder, File } });
}

function resetFooterToSelected() {
  if (selectedPath) {
    const node = findNodeByPath(mockData, selectedPath);
    if (node) {
      updateFooter(node.path, node.sizeBytes);
      return;
    }
  }
  
  const pathEl = document.getElementById('sim-footer-path');
  const sizeEl = document.getElementById('sim-footer-size');
  if (pathEl && sizeEl) {
    pathEl.textContent = "Hover or click a block above to inspect...";
    pathEl.classList.remove('active');
    sizeEl.textContent = "";
  }
}

function findNodeByPath(node, path) {
  if (node.path === path) return node;
  if (node.children) {
    for (let child of node.children) {
      const result = findNodeByPath(child, path);
      if (result) return result;
    }
  }
  return null;
}

// --- BENCHMARKS: CHART.JS SETUP ---
const benchmarkData = {
  nvme: {
    title: "Samsung 990 Pro NVMe PCIe Gen 4 SSD",
    desc: "Scanning dense repositories containing millions of files and nested directories. (Warm Cache)",
    labels: ['eDirStat (Rust, Parallel)', 'QDirStat (Perl Backend)', 'WinDirStat (Legacy C++)', 'WizTree (Windows MFT)'],
    dnfTexts: [null, null, "Incompatible (Not supported on Linux/btrfs)", "Incompatible (Not supported on Linux/btrfs)"],
    datasets: [{
      label: 'Median Scan Duration (Seconds)',
      data: [0.86, 6.91, null, null],
      backgroundColor: [
        'rgba(124, 58, 237, 0.85)', // primary violet glow
        'rgba(6, 182, 212, 0.6)',  // cyan
        'rgba(148, 163, 184, 0.4)', // slate
        'rgba(236, 72, 153, 0.4)'  // pink
      ],
      borderColor: [
        '#c084fc',
        '#22d3ee',
        '#cbd5e1',
        '#f472b6'
      ],
      borderWidth: 1.5,
      borderRadius: 6,
      barThickness: 40
    }]
  },
  sata: {
    title: "Samsung 870 QVO SATA SSD (8TB)",
    desc: "Scanning game installations containing a mix of large zip archives and small asset files.",
    labels: ['eDirStat (Rust, Parallel)', 'QDirStat (Perl Backend)', 'WinDirStat (Legacy C++)', 'WizTree (Windows MFT)'],
    dnfTexts: [null, null, "Incompatible (Not supported on Linux/btrfs)", "Incompatible (Not supported on Linux/btrfs)"],
    datasets: [{
      label: 'Median Scan Duration (Seconds)',
      data: [0.47, 4.54, null, null],
      backgroundColor: [
        'rgba(124, 58, 237, 0.85)',
        'rgba(6, 182, 212, 0.6)',
        'rgba(148, 163, 184, 0.4)',
        'rgba(236, 72, 153, 0.4)'
      ],
      borderColor: [
        '#c084fc',
        '#22d3ee',
        '#cbd5e1',
        '#f472b6'
      ],
      borderWidth: 1.5,
      borderRadius: 6,
      barThickness: 40
    }]
  },
  hdd: {
    title: "Toshiba MG09SACA Mechanical HDD (16TB)",
    desc: "Traversing massive deeply nested directory structures on traditional spinning disks.",
    labels: ['eDirStat (Rust, Parallel)', 'QDirStat (Perl Backend)', 'WinDirStat (Legacy C++)', 'WizTree (Windows MFT)'],
    dnfTexts: [null, null, "Incompatible (Not supported on Linux/btrfs)", "Incompatible (Not supported on Linux/btrfs)"],
    datasets: [{
      label: 'Median Scan Duration (Seconds)',
      data: [0.53, 3.54, null, null],
      backgroundColor: [
        'rgba(124, 58, 237, 0.85)',
        'rgba(6, 182, 212, 0.6)',
        'rgba(148, 163, 184, 0.4)',
        'rgba(236, 72, 153, 0.4)'
      ],
      borderColor: [
        '#c084fc',
        '#22d3ee',
        '#cbd5e1',
        '#f472b6'
      ],
      borderWidth: 1.5,
      borderRadius: 6,
      barThickness: 40
    }]
  },
  mzvlb: {
    title: "Samsung MZVLB512HBJQ PCIe Gen 3 SSD",
    desc: "Scanning Windows system directories containing deep system libraries and DLLs.",
    labels: ['eDirStat (Rust, Parallel)', 'WizTree (Windows MFT)', 'WinDirStat (Legacy C++)', 'QDirStat (Perl Backend)'],
    dnfTexts: [null, null, null, "Incompatible (Not supported on Windows)"],
    datasets: [{
      label: 'Median Scan Duration (Seconds)',
      data: [1.72, 4.38, 92.38, null],
      backgroundColor: [
        'rgba(124, 58, 237, 0.85)',
        'rgba(236, 72, 153, 0.6)',
        'rgba(148, 163, 184, 0.4)',
        'rgba(6, 182, 212, 0.4)'
      ],
      borderColor: [
        '#c084fc',
        '#f472b6',
        '#cbd5e1',
        '#22d3ee'
      ],
      borderWidth: 1.5,
      borderRadius: 6,
      barThickness: 40
    }]
  }
};

let benchmarkChart = null;

function initChart() {
  const chartEl = document.getElementById('benchmarkChart');
  if (!chartEl) return;
  const ctx = chartEl.getContext('2d');
  
  benchmarkChart = new Chart(ctx, {
    type: 'bar',
    data: JSON.parse(JSON.stringify(benchmarkData.mzvlb)), // Clone
    plugins: [{
      id: 'dnfPlugin',
      afterDatasetsDraw(chart) {
        const { ctx, chartArea: { left }, scales: { y } } = chart;
        ctx.save();
        chart.data.datasets.forEach((dataset, datasetIndex) => {
          const meta = chart.getDatasetMeta(datasetIndex);
          
          // Find the slowest scan time that still finishes (maximum numeric value)
          const validValues = dataset.data.filter(val => val !== null && val !== undefined && !isNaN(val));
          const maxVal = validValues.length > 0 ? Math.max(...validValues) : 0;
          
          meta.data.forEach((bar, index) => {
            const val = dataset.data[index];
            if (val === null || val === undefined || isNaN(val)) {
              const text = chart.data.dnfTexts?.[index] || 'Incompatible';
              ctx.font = 'bold 12px "JetBrains Mono", monospace';
              ctx.fillStyle = '#ef4444'; // Red
              ctx.textAlign = 'left';
              ctx.textBaseline = 'middle';
              const yPos = bar ? bar.y : y.getPixelForValue(index);
              ctx.fillText(text, left + 15, yPos);
            } else {
              // Calculate relative performance speedup factor
              const multiplier = (maxVal > 0 && val > 0) ? (maxVal / val) : 1.0;
              const text = multiplier.toFixed(1) + 'x';
              
              ctx.font = 'bold 14px "JetBrains Mono", monospace';
              ctx.fillStyle = (Array.isArray(dataset.borderColor) ? dataset.borderColor[index] : dataset.borderColor) || '#f8fafc';
              ctx.textAlign = 'left';
              ctx.textBaseline = 'middle';
              const xPos = (bar ? bar.x : left) + 10;
              const yPos = bar ? bar.y : y.getPixelForValue(index);
              ctx.fillText(text, xPos, yPos);
            }
          });
        });
        ctx.restore();
      }
    }],
    options: {
      layout: {
        padding: {
          right: 60
        }
      },
      indexAxis: 'y', // Horizontal bars
      responsive: true,
      maintainAspectRatio: false,
      scales: {
        x: {
          grid: {
            color: 'rgba(255, 255, 255, 0.05)',
            drawBorder: false
          },
          ticks: {
            color: '#94a3b8',
            font: {
              family: 'JetBrains Mono',
              size: 11
            }
          },
          title: {
            display: true,
            text: 'Seconds (Lower is Better)',
            color: '#64748b',
            font: {
              family: 'Outfit',
              weight: 'bold'
            }
          }
        },
        y: {
          grid: {
            display: false
          },
          ticks: {
            color: '#f8fafc',
            font: {
              family: 'Outfit',
              size: 13,
              weight: '600'
            }
          }
        }
      },
      plugins: {
        legend: {
          display: false
        },
        tooltip: {
          backgroundColor: '#0f111a',
          titleColor: '#f8fafc',
          bodyColor: '#cbd5e1',
          bodyFont: {
            family: 'JetBrains Mono'
          },
          titleFont: {
            family: 'Outfit',
            weight: 'bold'
          },
          borderColor: 'rgba(124, 58, 237, 0.3)',
          borderWidth: 1,
          padding: 12,
          displayColors: false,
          callbacks: {
            label: function(context) {
              return `Time: ${context.parsed.x} seconds`;
            }
          }
        }
      }
    }
  });
}

function updateChart(driveKey) {
  if (!benchmarkChart) return;
  const currentData = benchmarkData[driveKey];
  
  // Update header text
  document.getElementById('benchmark-title').textContent = currentData.title;
  document.getElementById('benchmark-desc').textContent = currentData.desc;
  
  // Animate chart transition and update properties dynamically
  benchmarkChart.data.labels = currentData.labels;
  benchmarkChart.data.dnfTexts = currentData.dnfTexts;
  benchmarkChart.data.datasets[0].data = currentData.datasets[0].data;
  benchmarkChart.data.datasets[0].backgroundColor = currentData.datasets[0].backgroundColor;
  benchmarkChart.data.datasets[0].borderColor = currentData.datasets[0].borderColor;
  
  benchmarkChart.update();
}

// --- DEDUPLICATOR PIPELINE SIMULATOR DATA ---
const mockFilePool = [
  { id: 1, name: "core_engine.rs", size: 1048576, ext: "code", duplicateGroup: "group_a", prefix: "b3a7", mid: "cd90", suf: "8f12" },
  { id: 2, name: "core_engine_backup.rs", size: 1048576, ext: "code", duplicateGroup: "group_a", prefix: "b3a7", mid: "cd90", suf: "8f12" },
  { id: 3, name: "shader_renderer.cpp", size: 5242880, ext: "code", duplicateGroup: "group_b", prefix: "e91a", mid: "21bf", suf: "aa55" },
  { id: 4, name: "shader_copy.cpp", size: 5242880, ext: "code", duplicateGroup: "group_b", prefix: "e91a", mid: "21bf", suf: "aa55" },
  { id: 5, name: "test_shader_backup.cpp", size: 5242880, ext: "code", duplicateGroup: "group_b", prefix: "e91a", mid: "21bf", suf: "aa55" },
  { id: 6, name: "background_audio.wav", size: 24117248, ext: "audio", duplicateGroup: null, prefix: "4a2b", mid: "1c2f", suf: "ff00" },
  { id: 7, name: "intro_sequence.mp4", size: 141557760, ext: "video", duplicateGroup: "group_c", prefix: "9a2f", mid: "33ff", suf: "cda1" },
  { id: 8, name: "intro_copy_raw.mp4", size: 141557760, ext: "video", duplicateGroup: "group_c", prefix: "9a2f", mid: "33ff", suf: "cda1" },
  { id: 9, name: "manifest_config.toml", size: 4096, ext: "config", duplicateGroup: null, prefix: "f1c1", mid: "00a2", suf: "1234" },
  { id: 10, name: "profile_avatar.png", size: 5242880, ext: "image", duplicateGroup: null, prefix: "e91a", mid: "0000", suf: "99aa" }
];

let pipelineState = "idle";
let selectedDuplicates = new Set();
let foundGroups = [];
let activeTimeouts = [];

function writeConsole(text, type = "info") {
  const consoleEl = document.getElementById('virtual-console');
  if (!consoleEl) return;
  
  const line = document.createElement('div');
  line.className = `console-line text-${type}`;
  line.textContent = `[${new Date().toLocaleTimeString()}] ${text}`;
  consoleEl.appendChild(line);
  consoleEl.scrollTop = consoleEl.scrollHeight;
}

function updateStageUi(stepNum, statusText, className) {
  const stepEl = document.getElementById(`step-${stepNum}`);
  const statusEl = document.getElementById(`status-${stepNum}`);
  if (!stepEl || !statusEl) return;
  
  stepEl.className = 'stage-step';
  if (className) {
    stepEl.classList.add(className);
  }
  statusEl.textContent = statusText;
}

// Programmatic Width-Aware Grid Spacing Engine
function arrangeFilesGrid(files, alignMode = "grid") {
  const canvas = document.getElementById('dedup-canvas');
  if (!canvas) return;
  
  const width = canvas.clientWidth;
  const cardW = 130;
  const cardH = 46;

  // Clear any previously existing grouping boxes
  document.querySelectorAll('.sim-group-box').forEach(el => el.remove());

  if (alignMode === "grid") {
    const colGap = 16;
    const rowGap = 12;
    const colWidth = cardW + colGap;
    
    // Dynamically calculate columns based on actual client resolution width
    const cols = Math.max(2, Math.floor((width - 20) / colWidth));
    
    const totalGridWidth = cols * colWidth - colGap;
    const startX = (width - totalGridWidth) / 2;
    const startY = 16;

    files.forEach((file, idx) => {
      const el = document.getElementById(`sim-file-${file.id}`);
      if (!el) return;
      
      const col = idx % cols;
      const row = Math.floor(idx / cols);
      const x = startX + col * colWidth;
      const y = startY + row * (cardH + rowGap);
      
      el.style.transition = 'all 0.6s cubic-bezier(0.4, 0, 0.2, 1)';
      el.style.left = `${x}px`;
      el.style.top = `${y}px`;
      el.classList.remove('grouped-active');
    });
  } else if (alignMode === "groups") {
    // Isolate survivor groups
    const groups = {};
    files.forEach(f => {
      if (f.duplicateGroup) {
        if (!groups[f.duplicateGroup]) groups[f.duplicateGroup] = [];
        groups[f.duplicateGroup].push(f);
      }
    });

    const groupKeys = Object.keys(groups);
    const numGroups = groupKeys.length;
    const sectionW = width / Math.max(1, numGroups);

    groupKeys.forEach((groupKey, gIdx) => {
      const groupFiles = groups[groupKey];
      const sectionX = gIdx * sectionW;
      
      // Draw enclosing box
      const borderEl = document.createElement('div');
      borderEl.id = `sim-group-box-${groupKey}`;
      borderEl.className = 'sim-group-box';
      borderEl.style.left = `${sectionX + 10}px`;
      borderEl.style.width = `${sectionW - 20}px`;
      borderEl.style.top = '15px';
      borderEl.style.height = '175px';
      
      const label = document.createElement('span');
      label.className = 'sim-group-box-label';
      label.textContent = `GROUP: ${groupKey.replace('_', ' ').toUpperCase()}`;
      borderEl.appendChild(label);
      canvas.appendChild(borderEl);

      // Stack duplicate file nodes vertically inside the box
      groupFiles.forEach((file, fIdx) => {
        const el = document.getElementById(`sim-file-${file.id}`);
        if (!el) return;
        
        const x = sectionX + (sectionW - cardW) / 2;
        const y = 30 + fIdx * 48;
        
        el.style.transition = 'all 0.6s cubic-bezier(0.4, 0, 0.2, 1)';
        el.style.left = `${x}px`;
        el.style.top = `${y}px`;
        el.classList.add('grouped-active');
      });
    });
  }
}

function renderFileBlock(file, container, x, y) {
  const card = document.createElement('div');
  card.className = `sim-streaming-file ext-${file.ext}`;
  card.id = `sim-file-${file.id}`;
  card.style.left = `${x}px`;
  card.style.top = `${y}px`;
  
  const topRow = document.createElement('div');
  topRow.className = 'sim-file-top';
  
  const title = document.createElement('span');
  title.className = 'sim-file-title';
  title.textContent = file.name;
  topRow.appendChild(title);
  
  card.appendChild(topRow);
  
  const subRow = document.createElement('div');
  subRow.className = 'sim-file-top';
  
  const hashLabel = document.createElement('span');
  hashLabel.className = 'sim-file-hash';
  hashLabel.id = `sim-hash-${file.id}`;
  hashLabel.textContent = "---";
  subRow.appendChild(hashLabel);
  
  const sizeLabel = document.createElement('span');
  sizeLabel.className = 'sim-file-size';
  sizeLabel.textContent = formatBytes(file.size);
  subRow.appendChild(sizeLabel);
  
  card.appendChild(subRow);
  container.appendChild(card);
  return card;
}

function clearAnimationCanvas() {
  const canvas = document.getElementById('dedup-canvas');
  if (!canvas) return;
  
  const instruction = document.getElementById('canvas-instructions');
  if (instruction) instruction.style.display = 'none';
  
  document.querySelectorAll('.sim-streaming-file').forEach(el => el.remove());
  document.querySelectorAll('.sim-group-box').forEach(el => el.remove());
}

// Managed sleep wrapper targeting an interrupt-friendly registry
const sleep = (ms) => {
  return new Promise((resolve, reject) => {
    const handle = setTimeout(resolve, ms);
    activeTimeouts.push({ handle, reject });
  });
};

function clearRunningTimeouts() {
  activeTimeouts.forEach(t => {
    clearTimeout(t.handle);
    t.reject(new Error("ResetInterrupt"));
  });
  activeTimeouts = [];
}

async function runDeduplicationPipeline() {
  if (pipelineState === "scanning") return;
  pipelineState = "scanning";
  
  const minSizeSelect = document.getElementById('dedup-sim-size');
  const minSizeLimit = parseInt(minSizeSelect ? minSizeSelect.value : 1024);
  
  const startBtn = document.getElementById('btn-start-dedup');
  if (startBtn) startBtn.disabled = true;
  
  for (let s = 1; s <= 7; s++) {
    updateStageUi(s, "Idle", "");
  }
  
  const tableView = document.getElementById('dedup-table-view');
  if (tableView) tableView.classList.add('collapsed');
  
  const tbody = document.getElementById('sim-dedup-rows');
  if (tbody) tbody.innerHTML = "";
  
  selectedDuplicates.clear();
  foundGroups = [];
  
  clearAnimationCanvas();
  const canvas = document.getElementById('dedup-canvas');
  
  writeConsole("[ENGINE] Initializing sector-aligned I/O and preparing file buffers...");
  
  try {
    await sleep(600);

    // Initial render using the programmatic grid spacing layout
    let activeCandidates = [...mockFilePool];
    activeCandidates.forEach((file) => {
      renderFileBlock(file, canvas, 0, 0);
    });
    arrangeFilesGrid(activeCandidates, "grid");
    
    await sleep(1000);

    // --- STAGE 1: Size Partitioning ---
    updateStageUi(1, "Active", "active");
    writeConsole("[P-1] Scanning directory layout for matching byte footprints...");
    await sleep(400);
    
    const tooSmallFiles = activeCandidates.filter(f => f.size < minSizeLimit);
    tooSmallFiles.forEach(file => {
      const el = document.getElementById(`sim-file-${file.id}`);
      if (el) {
        el.style.animation = 'file-mismatch-out 0.8s ease forwards';
        writeConsole(`[P-1] Discarded below minimum size threshold: ${file.name}`, "warning");
      }
    });
    activeCandidates = activeCandidates.filter(f => f.size >= minSizeLimit);
    await sleep(800);
    document.querySelectorAll('.sim-streaming-file[style*="animation"]').forEach(el => el.remove());

    const sizeMap = {};
    activeCandidates.forEach(f => {
      sizeMap[f.size] = (sizeMap[f.size] || 0) + 1;
    });
    
    const singletons = activeCandidates.filter(f => sizeMap[f.size] === 1);
    singletons.forEach(file => {
      const el = document.getElementById(`sim-file-${file.id}`);
      if (el) {
        el.style.animation = 'file-mismatch-out 0.8s ease forwards';
        writeConsole(`[P-1] Discarded non-duplicate (unique size): ${file.name}`);
      }
    });
    activeCandidates = activeCandidates.filter(f => sizeMap[f.size] > 1);
    await sleep(800);
    document.querySelectorAll('.sim-streaming-file[style*="animation"]').forEach(el => el.remove());
    
    arrangeFilesGrid(activeCandidates, "grid");
    updateStageUi(1, "Complete", "complete");

    if (activeCandidates.length === 0) {
      writeConsole("[ENGINE] Scanning complete. No duplicate candidates cleared Phase 1.", "success");
      pipelineState = "idle";
      if (startBtn) startBtn.disabled = false;
      return;
    }

    // --- STAGE 2: Prefix Hashing ---
    await sleep(800);
    updateStageUi(2, "Active", "active");
    writeConsole("[P-2] Computing 4KB cryptographic header checksums...");
    
    activeCandidates.forEach(file => {
      const hashEl = document.getElementById(`sim-hash-${file.id}`);
      if (hashEl) {
        hashEl.textContent = `Pre: ${file.prefix}`;
        hashEl.style.color = '#a855f7';
      }
    });
    await sleep(1000);

    const prefixMap = {};
    activeCandidates.forEach(f => {
      prefixMap[f.prefix] = (prefixMap[f.prefix] || 0) + 1;
    });
    
    const prefixMismatches = activeCandidates.filter(f => prefixMap[f.prefix] === 1);
    prefixMismatches.forEach(file => {
      const el = document.getElementById(`sim-file-${file.id}`);
      if (el) {
        el.style.animation = 'file-mismatch-out 0.8s ease forwards';
        writeConsole(`[P-2] Discarded mismatching prefix: ${file.name}`, "warning");
      }
    });
    activeCandidates = activeCandidates.filter(f => prefixMap[f.prefix] > 1);
    await sleep(800);
    document.querySelectorAll('.sim-streaming-file[style*="animation"]').forEach(el => el.remove());
    
    arrangeFilesGrid(activeCandidates, "grid");
    updateStageUi(2, "Complete", "complete");

    // --- STAGE 3: Midpoint Hashing ---
    await sleep(800);
    updateStageUi(3, "Active", "active");
    writeConsole("[P-3] Querying central file clusters (Midpoint hashing)...");
    
    activeCandidates.forEach(file => {
      const hashEl = document.getElementById(`sim-hash-${file.id}`);
      if (hashEl) {
        hashEl.textContent = `Mid: ${file.mid}`;
        hashEl.style.color = '#3b82f6';
      }
    });
    await sleep(1000);

    const midMap = {};
    activeCandidates.forEach(f => {
      midMap[f.mid] = (midMap[f.mid] || 0) + 1;
    });
    
    const midMismatches = activeCandidates.filter(f => midMap[f.mid] === 1);
    midMismatches.forEach(file => {
      const el = document.getElementById(`sim-file-${file.id}`);
      if (el) {
        el.style.animation = 'file-mismatch-out 0.8s ease forwards';
        writeConsole(`[P-3] Discarded mismatching midpoint: ${file.name}`, "warning");
      }
    });
    activeCandidates = activeCandidates.filter(f => midMap[f.mid] > 1);
    await sleep(800);
    document.querySelectorAll('.sim-streaming-file[style*="animation"]').forEach(el => el.remove());
    
    arrangeFilesGrid(activeCandidates, "grid");
    updateStageUi(3, "Complete", "complete");

    // --- STAGE 4: Suffix Hashing ---
    await sleep(800);
    updateStageUi(4, "Active", "active");
    writeConsole("[P-4] Examining final boundary sectors (Suffix Hashing)...");
    
    activeCandidates.forEach(file => {
      const hashEl = document.getElementById(`sim-hash-${file.id}`);
      if (hashEl) {
        hashEl.textContent = `Suf: ${file.suf}`;
        hashEl.style.color = '#10b981';
      }
    });
    await sleep(1000);
    updateStageUi(4, "Complete", "complete");

    // --- STAGE 5: Multi-Range Hashing ---
    await sleep(800);
    updateStageUi(5, "Active", "active");
    writeConsole("[P-5] Initiating multi-range block sampling on files exceeding 100MB...");
    
    activeCandidates.forEach(file => {
      const el = document.getElementById(`sim-file-${file.id}`);
      if (el && file.size >= 104857600) {
        el.style.borderColor = '#eab308';
        writeConsole(`[P-5] Periodic checking applied across 100MB boundaries: ${file.name}`);
      }
    });
    await sleep(1200);
    updateStageUi(5, "Complete", "complete");

    // --- STAGE 6: Full BLAKE3 Hashing ---
    await sleep(800);
    updateStageUi(6, "Active", "active");
    writeConsole("[P-6] Hashing candidates via full BLAKE3 256-bit cryptographic loops...");
    
    for (let file of activeCandidates) {
      const hashEl = document.getElementById(`sim-hash-${file.id}`);
      if (hashEl) {
        const mockHash = Math.random().toString(16).substring(2, 10).toUpperCase();
        hashEl.textContent = `B3: ${mockHash}`;
        hashEl.style.color = '#ef4444';
        writeConsole(`[P-6] Full hash computed for ${file.name}: ${mockHash}`);
        await sleep(200);
      }
    }
    
    // Format results groups before rearranging
    foundGroups = [
      { size: 1048576, nodes: [1, 2], name: "group_a" },
      { size: 5242880, nodes: [3, 4, 5], name: "group_b" },
      { size: 141557760, nodes: [7, 8], name: "group_c" }
    ];
    foundGroups = foundGroups.filter(g => g.size >= minSizeLimit);

    // Rearrange survivors into duplicate groups side-by-side
    arrangeFilesGrid(activeCandidates, "groups");
    
    await sleep(1000);
    updateStageUi(6, "Complete", "complete");

    // --- STAGE 7: Timestamp Validation ---
    updateStageUi(7, "Active", "active");
    writeConsole("[P-7] Interrogating filesystem journal registers to validate timestamps...");
    
    activeCandidates.forEach(file => {
      const el = document.getElementById(`sim-file-${file.id}`);
      if (el) {
        el.style.borderColor = '#10b981';
        el.style.boxShadow = '0 0 15px rgba(16, 185, 129, 0.3)';
      }
    });
    await sleep(800);
    updateStageUi(7, "Complete", "complete");

    writeConsole("[ENGINE] Deduplication scan finalized successfully! Building result table...", "success");
    
    buildDeduplicatorTable();
    
    pipelineState = "idle";
    if (startBtn) startBtn.disabled = false;
    
  } catch (err) {
    if (err.message === "ResetInterrupt") {
      console.log("[SYSTEM] Active scan interrupted via Reset command.");
    } else {
      console.error(err);
    }
  }
}

function buildDeduplicatorTable() {
  const tbody = document.getElementById('sim-dedup-rows');
  const tableView = document.getElementById('dedup-table-view');
  if (!tbody || !tableView) return;
  
  tbody.innerHTML = "";
  
  foundGroups.forEach(group => {
    group.nodes.forEach((nodeId, idx) => {
      const file = mockFilePool.find(f => f.id === nodeId);
      if (!file) return;
      
      const isOriginal = idx === 0;
      const tr = document.createElement('tr');
      tr.className = isOriginal ? 'original-row' : 'duplicate-row';
      tr.id = `row-file-${file.id}`;
      
      // Checkbox cell
      const tdCheck = document.createElement('td');
      if (!isOriginal) {
        const checkbox = document.createElement('input');
        checkbox.type = 'checkbox';
        checkbox.setAttribute('data-id', file.id);
        checkbox.setAttribute('data-size', file.size);
        checkbox.addEventListener('change', (e) => {
          toggleDuplicateSelection(file.id, e.target.checked);
        });
        tdCheck.appendChild(checkbox);
      } else {
        tdCheck.innerHTML = `<span style="color: #eab308; font-size: 0.8rem;">★</span>`;
      }
      tr.appendChild(tdCheck);
      
      // Filename
      const tdName = document.createElement('td');
      tdName.textContent = isOriginal ? `⭐ ${file.name}` : `     >> ${file.name}`;
      tdName.style.whiteSpace = 'pre';
      tr.appendChild(tdName);
      
      // Folder path
      const tdFolder = document.createElement('td');
      const slashIdx = file.path ? file.path.lastIndexOf('/') : 1;
      const folderVal = file.path ? file.path.substring(0, slashIdx || 1) : "/Root/MockPath";
      tdFolder.textContent = folderVal;
      tr.appendChild(tdFolder);
      
      // Size
      const tdSize = document.createElement('td');
      tdSize.textContent = formatBytes(file.size);
      tr.appendChild(tdSize);
      
      // Reclaimable
      const tdReclaim = document.createElement('td');
      tdReclaim.textContent = isOriginal ? "Original Header" : formatBytes(file.size);
      tdReclaim.className = isOriginal ? 'text-dim' : 'text-success';
      tr.appendChild(tdReclaim);
      
      tbody.appendChild(tr);
    });
  });
  
  tableView.classList.remove('collapsed');
}

function toggleDuplicateSelection(fileId, isChecked) {
  if (isChecked) {
    selectedDuplicates.add(fileId);
  } else {
    selectedDuplicates.delete(fileId);
  }
  
  calculateReclaimableStorage();
}

function calculateReclaimableStorage() {
  let totalBytes = 0;
  selectedDuplicates.forEach(id => {
    const file = mockFilePool.find(f => f.id === id);
    if (file) {
      totalBytes += file.size;
    }
  });
  
  const reclaimEl = document.getElementById('reclaim-amount');
  if (reclaimEl) {
    reclaimEl.textContent = formatBytes(totalBytes);
  }
}

function resetDeduplicatorSimulator() {
  // 1. Terminate all running intervals and async timeout streams
  clearRunningTimeouts();
  pipelineState = "idle";
  
  // 2. Re-enable start button
  const startBtn = document.getElementById('btn-start-dedup');
  if (startBtn) startBtn.disabled = false;
  
  // 3. Reset 7 pipeline stage indicators to Idle state
  for (let s = 1; s <= 7; s++) {
    updateStageUi(s, "Idle", "");
  }
  
  // 4. Restore original Instructions display to canvas
  const canvas = document.getElementById('dedup-canvas');
  if (canvas) {
    document.querySelectorAll('.sim-streaming-file').forEach(el => el.remove());
    document.querySelectorAll('.sim-group-box').forEach(el => el.remove());
    const instruction = document.getElementById('canvas-instructions');
    if (instruction) instruction.style.display = 'block';
  }
  
  // 5. Restore Virtual Console to startup baseline message
  const consoleEl = document.getElementById('virtual-console');
  if (consoleEl) {
    consoleEl.innerHTML = '<div class="console-line text-dim">[SYSTEM] Engine online. Awaiting control signal.</div>';
  }
  
  // 6. Reset duplicate rows table back to its initial collapsed state
  const tableView = document.getElementById('dedup-table-view');
  if (tableView) {
    tableView.classList.add('collapsed');
  }
  
  const tbody = document.getElementById('sim-dedup-rows');
  if (tbody) {
    tbody.innerHTML = "";
  }
  
  selectedDuplicates.clear();
  const reclaimAmtEl = document.getElementById('reclaim-amount');
  if (reclaimAmtEl) reclaimAmtEl.textContent = "0 Bytes";
  writeConsole("[SYSTEM] Pipeline simulator reset to clean state.");
}

function triggerReclaimAnimation(actionType) {
  if (selectedDuplicates.size === 0) return;
  
  const selectedCount = selectedDuplicates.size;
  
  toast_success(
    actionType === 'hardlink'
      ? `Successfully replaced ${selectedCount} duplicate(s) with hardlinks!`
      : `Permanently deleted ${selectedCount} duplicate file(s) from system buffers.`
  );

  const tbody = document.getElementById('sim-dedup-rows');
  if (tbody) {
    tbody.classList.add('reclaim-flash');
    setTimeout(() => {
      tbody.classList.remove('reclaim-flash');
    }, 800);
  }

  // Remove matching elements from DOM with a sliding transition
  selectedDuplicates.forEach(id => {
    const row = document.getElementById(`row-file-${id}`);
    if (row) {
      row.style.transition = 'all 0.4s ease';
      row.style.opacity = '0';
      row.style.transform = 'translateX(-20px)';
      setTimeout(() => {
        row.remove();
      }, 400);
    }
    
    // Clear matches on canvas
    const canvasEl = document.getElementById(`sim-file-${id}`);
    if (canvasEl) {
      canvasEl.style.transition = 'all 0.4s ease';
      canvasEl.style.opacity = '0';
      canvasEl.style.transform = 'scale(0.8)';
      setTimeout(() => {
        canvasEl.remove();
      }, 400);
    }
  });

  // Reset counters
  selectedDuplicates.clear();
  const reclaimAmtEl = document.getElementById('reclaim-amount');
  if (reclaimAmtEl) reclaimAmtEl.textContent = "0 Bytes";
}

// Hook actions into simulator controls on load
document.addEventListener('DOMContentLoaded', () => {
  // Initialize Lucide Icons
  createIcons({
    icons: {
      Folder, File, Zap, Cpu, Shield, Layers, Image, HardDrive, Download, 
      ChevronRight, ChevronDown, Trash2, BarChart2, Eye, Copy, ExternalLink,
      Database
    }
  });
  
  // Render Simulator components
  const treeContainer = document.getElementById('sim-tree-root');
  if (treeContainer) {
    treeContainer.appendChild(createTreeDOM(mockData));
  }
  
  const treemapCanvas = document.getElementById('sim-treemap-canvas');
  if (treemapCanvas) {
    renderTreemap(mockData, treemapCanvas, 0, 0, 100, 100, true);
  }
  
  // Re-run icon parser for generated elements
  createIcons({ icons: { Folder, File } });
  
  // Setup Benchmark Tabs
  const benchmarkTabs = document.querySelectorAll('.benchmark-tab');
  benchmarkTabs.forEach(tab => {
    tab.addEventListener('click', () => {
      benchmarkTabs.forEach(t => t.classList.remove('active'));
      tab.classList.add('active');
      
      const target = tab.getAttribute('data-target');
      updateChart(target);
    });
  });
  
  // Init Chart.js
  if (document.getElementById('benchmarkChart')) {
    initChart();
  }
  
  // Setup Guide Tabs
  const guideTabs = document.querySelectorAll('.guide-tab');
  const guidePanes = document.querySelectorAll('.guide-pane');
  guideTabs.forEach(tab => {
    tab.addEventListener('click', () => {
      guideTabs.forEach(t => t.classList.remove('active'));
      guidePanes.forEach(p => p.classList.remove('active'));
      
      tab.classList.add('active');
      const target = tab.getAttribute('data-target');
      const targetPane = document.getElementById(target);
      if (targetPane) targetPane.classList.add('active');
    });
  });

  const startBtn = document.getElementById('btn-start-dedup');
  if (startBtn) {
    startBtn.addEventListener('click', runDeduplicationPipeline);
  }
  
  const resetBtn = document.getElementById('btn-reset-dedup');
  if (resetBtn) {
    resetBtn.addEventListener('click', resetDeduplicatorSimulator);
  }
  
  const linkBtn = document.getElementById('btn-reclaim-hardlink');
  if (linkBtn) {
    linkBtn.addEventListener('click', () => triggerReclaimAnimation('hardlink'));
  }
  
  const deleteBtn = document.getElementById('btn-reclaim-delete');
  if (deleteBtn) {
    deleteBtn.addEventListener('click', () => triggerReclaimAnimation('delete'));
  }

  // --- WINDOW RESIZE ENGINE COORDINATOR ---
  let resizeTimeout;
  window.addEventListener('resize', () => {
    clearTimeout(resizeTimeout);
    resizeTimeout = setTimeout(() => {
      // Re-arrange elements matching current layout grid state
      const canvas = document.getElementById('dedup-canvas');
      if (!canvas) return;

      const streamingFiles = Array.from(canvas.querySelectorAll('.sim-streaming-file'));
      if (streamingFiles.length === 0) return;

      const activeIds = streamingFiles.map(el => parseInt(el.id.replace('sim-file-', ''))).filter(Boolean);
      const files = mockFilePool.filter(f => activeIds.includes(f.id));

      if (foundGroups.length > 0) {
        arrangeFilesGrid(files, "groups");
      } else {
        arrangeFilesGrid(files, "grid");
      }
    }, 250);
  });
});

// Polyfill Toast interface natively to ensure sandbox actions display alerts correctly
function toast_success(message) {
  // Create temporary toast element
  const container = document.body;
  const toast = document.createElement('div');
  toast.className = 'glass-card';
  toast.style.position = 'fixed';
  toast.style.bottom = '20px';
  toast.style.right = '20px';
  toast.style.zIndex = '9999';
  toast.style.padding = '12px 24px';
  toast.style.borderLeft = '4px solid #22c55e';
  toast.style.backgroundColor = '#0d0f18';
  toast.style.boxShadow = '0 10px 25px rgba(0,0,0,0.5)';
  toast.style.fontFamily = 'Outfit, sans-serif';
  toast.style.fontSize = '0.9rem';
  toast.style.color = '#f8fafc';
  toast.innerHTML = `✨ ${message}`;
  
  container.appendChild(toast);
  setTimeout(() => {
    toast.style.transition = 'all 0.5s ease';
    toast.style.opacity = '0';
    toast.style.transform = 'translateY(10px)';
    setTimeout(() => toast.remove(), 500);
  }, 3500);
}
