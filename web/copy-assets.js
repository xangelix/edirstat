const fs = require('fs');
const path = require('path');

const projectRoot = __dirname;
const repoRoot = path.resolve(projectRoot, '..');

const targets = [
  {
    src: path.join(repoRoot, 'assets/img/icon.ico'),
    dest: path.join(projectRoot, 'static/favicon.ico')
  },
  {
    src: path.join(repoRoot, 'assets/img/logo-nosubtext-transparent.svg'),
    dest: path.join(projectRoot, 'static/assets/logo-nosubtext-transparent.svg')
  },
  {
    src: path.join(repoRoot, 'assets/img/og-image.png'),
    dest: path.join(projectRoot, 'static/assets/og-image.png')
  },
  {
    src: path.join(projectRoot, 'robots.txt'),
    dest: path.join(projectRoot, 'static/robots.txt')
  },
  {
    src: path.join(repoRoot, 'assets/MGS3.edst.zst'),
    dest: path.join(projectRoot, 'static/MGS3.edst.zst')
  }
];

for (const target of targets) {
  const destDir = path.dirname(target.dest);
  if (!fs.existsSync(destDir)) {
    fs.mkdirSync(destDir, { recursive: true });
  }
  if (fs.existsSync(target.src)) {
    fs.copyFileSync(target.src, target.dest);
    console.log(`Copied ${path.relative(repoRoot, target.src)} -> ${path.relative(projectRoot, target.dest)}`);
  }
}

function copyFolderSync(from, to) {
  if (!fs.existsSync(from)) return;
  fs.mkdirSync(to, { recursive: true });
  fs.readdirSync(from).forEach(element => {
    const stat = fs.lstatSync(path.join(from, element));
    if (stat.isFile()) {
      fs.copyFileSync(path.join(from, element), path.join(to, element));
    } else if (stat.isDirectory()) {
      copyFolderSync(path.join(from, element), path.join(to, element));
    }
  });
}

// Copy the Web Viewer build output
const distPath = path.join(repoRoot, 'crates/edirstat-gui/dist');
const destAppPath = path.join(projectRoot, 'static/app');
if (fs.existsSync(distPath)) {
  copyFolderSync(distPath, destAppPath);
  console.log("Copied crates/edirstat-gui/dist -> static/app");
}

// Clean up old viewer directory if it exists
const oldViewerPath = path.join(projectRoot, 'static/viewer');
if (fs.existsSync(oldViewerPath)) {
  fs.rmSync(oldViewerPath, { recursive: true, force: true });
  console.log("Cleaned up old static/viewer directory");
}
