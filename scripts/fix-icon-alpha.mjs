/**
 * Make outside-squircle pixels transparent on brand / Tauri icon PNGs.
 * Does not redesign the mark — only removes white corner fill baked into alpha.
 */
import fs from 'node:fs'
import path from 'node:path'
import { PNG } from 'pngjs'
import { fileURLToPath } from 'node:url'

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
// Matches brand SVG clipPath: rx/ry = 112 on a 512 canvas.
const CORNER_RATIO = 112 / 512

function roundedRectCoverage(px, py, size, radius) {
  const x = Math.min(px, size - 1 - px)
  const y = Math.min(py, size - 1 - py)
  if (x >= radius || y >= radius) return 1

  const dx = radius - x - 0.5
  const dy = radius - y - 0.5
  const dist = Math.hypot(dx, dy) - radius
  // Soft AA band ~1px
  if (dist <= -0.5) return 1
  if (dist >= 0.5) return 0
  return 0.5 - dist
}

function unpremultiplyAgainstWhite(r, g, b, a, coverage) {
  // Corner fringe was composited over white. Recover dark edge toward black.
  const lum = (r + g + b) / (3 * 255)
  if (coverage < 1 && lum > 0.15 && a > 0) {
    const recoveredA = Math.max(0, Math.min(1, 1 - lum)) * coverage
    return [0, 0, 0, Math.round(recoveredA * 255)]
  }
  const outA = Math.round(a * coverage)
  return [r, g, b, outA]
}

function fixPng(filePath) {
  const input = fs.readFileSync(filePath)
  const png = PNG.sync.read(input)
  const { width, height } = png
  if (width !== height) {
    console.warn(`skip non-square ${filePath} (${width}x${height})`)
    return false
  }
  const radius = width * CORNER_RATIO
  let changed = 0
  for (let y = 0; y < height; y++) {
    for (let x = 0; x < width; x++) {
      const i = (width * y + x) << 2
      const coverage = roundedRectCoverage(x, y, width, radius)
      const [r, g, b, a] = [png.data[i], png.data[i + 1], png.data[i + 2], png.data[i + 3]]
      if (coverage >= 0.999 && a === 255 && r < 250) continue
      const [nr, ng, nb, na] = unpremultiplyAgainstWhite(r, g, b, a, coverage)
      if (na !== a || (na === 0 && a !== 0)) changed++
      png.data[i] = nr
      png.data[i + 1] = ng
      png.data[i + 2] = nb
      png.data[i + 3] = na
    }
  }
  fs.writeFileSync(filePath, PNG.sync.write(png))
  console.log(`fixed ${path.relative(ROOT, filePath)} (${changed} px)`)
  return true
}

const targets = [
  ...fs
    .readdirSync(path.join(ROOT, 'src/assets/brand'))
    .filter((f) => f.startsWith('shield-') && f.endsWith('.png'))
    .map((f) => path.join(ROOT, 'src/assets/brand', f)),
  path.join(ROOT, 'src-tauri/icons/icon.png'),
  path.join(ROOT, 'src-tauri/icons/32x32.png'),
  path.join(ROOT, 'src-tauri/icons/128x128.png'),
  path.join(ROOT, 'src-tauri/icons/128x128@2x.png'),
  ...fs
    .readdirSync(path.join(ROOT, 'src-tauri/icons'))
    .filter((f) => f.startsWith('Square') && f.endsWith('.png'))
    .map((f) => path.join(ROOT, 'src-tauri/icons', f)),
  path.join(ROOT, 'src-tauri/icons/StoreLogo.png'),
]

for (const file of targets) {
  if (fs.existsSync(file)) fixPng(file)
}

// Prefer the large brand mark as the canonical icon source for tauri icon regen.
const brand1024 = path.join(ROOT, 'src/assets/brand/shield-1024.png')
const iconPng = path.join(ROOT, 'src-tauri/icons/icon.png')
if (fs.existsSync(brand1024)) {
  fs.copyFileSync(brand1024, iconPng)
  console.log('synced brand 1024 → icons/icon.png')
}
