import fs from 'fs'
import path from 'path'
import { PNG } from 'pngjs'

function outsideRoundedRect(x, y, w, h, r) {
  const cx = Math.min(Math.max(x + 0.5, r), w - r)
  const cy = Math.min(Math.max(y + 0.5, r), h - r)
  // if inside the axis-aligned inset rect, definitely inside
  if (x + 0.5 >= r && x + 0.5 < w - r && y + 0.5 >= r && y + 0.5 < h - r) {
    return -1
  }
  // corner distance from nearest corner center
  let qx = 0
  let qy = 0
  if (x + 0.5 < r) qx = r - (x + 0.5)
  else if (x + 0.5 > w - r) qx = x + 0.5 - (w - r)
  if (y + 0.5 < r) qy = r - (y + 0.5)
  else if (y + 0.5 > h - r) qy = y + 0.5 - (h - r)

  if (qx === 0 || qy === 0) return -1
  return Math.sqrt(qx * qx + qy * qy) - r
}

function fixFile(file) {
  const png = PNG.sync.read(fs.readFileSync(file))
  const { width: w, height: h, data } = png
  const r = w * (112 / 512)

  for (let y = 0; y < h; y++) {
    for (let x = 0; x < w; x++) {
      const i = (w * y + x) << 2
      const d = outsideRoundedRect(x, y, w, h, r)
      if (d > 0.35) {
        data[i] = 0
        data[i + 1] = 0
        data[i + 2] = 0
        data[i + 3] = 0
        continue
      }
      if (d > -0.6) {
        // soft AA on the squircle edge — force near-black, fade alpha
        const t = Math.max(0, Math.min(1, 0.5 - d))
        data[i] = 10
        data[i + 1] = 10
        data[i + 2] = 10
        data[i + 3] = Math.round(255 * t)
        continue
      }

      // kill leftover bright fringe in corner zones
      const inCornerBand =
        (x < r + 1 && y < r + 1) ||
        (x > w - r - 2 && y < r + 1) ||
        (x < r + 1 && y > h - r - 2) ||
        (x > w - r - 2 && y > h - r - 2)
      if (inCornerBand) {
        const lum = (data[i] + data[i + 1] + data[i + 2]) / 3
        if (lum > 180 && data[i + 3] > 0) {
          data[i + 3] = 0
        }
      }
    }
  }

  fs.writeFileSync(file, PNG.sync.write(png))
  console.log('fixed', file)
}

const dirs = ['brand', 'src/assets/brand', 'public/brand']
for (const dir of dirs) {
  for (const f of fs.readdirSync(dir)) {
    if (/^shield-\d+\.png$/.test(f)) fixFile(path.join(dir, f))
  }
}
