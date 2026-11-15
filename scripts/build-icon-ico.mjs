import fs from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import pngToIco from 'png-to-ico'
import { PNG } from 'pngjs'

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..')
const src = PNG.sync.read(fs.readFileSync(path.join(ROOT, 'src/assets/brand/shield-1024.png')))
const sizes = [16, 24, 32, 48, 64, 128, 256]
const tmp = []

for (const size of sizes) {
  const out = new PNG({ width: size, height: size })
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      const sx = Math.min(src.width - 1, Math.floor(((x + 0.5) * src.width) / size))
      const sy = Math.min(src.height - 1, Math.floor(((y + 0.5) * src.height) / size))
      const si = (sy * src.width + sx) << 2
      const oi = (y * size + x) << 2
      out.data[oi] = src.data[si]
      out.data[oi + 1] = src.data[si + 1]
      out.data[oi + 2] = src.data[si + 2]
      out.data[oi + 3] = src.data[si + 3]
    }
  }
  const file = path.join(ROOT, 'src-tauri/icons', `_ico-${size}.png`)
  fs.writeFileSync(file, PNG.sync.write(out))
  tmp.push(file)
  console.log(
    'wrote',
    path.basename(file),
    'TL',
    [out.data[0], out.data[1], out.data[2], out.data[3]].join(','),
  )
}

const buf = await pngToIco(tmp)
const icoPath = path.join(ROOT, 'src-tauri/icons/icon.ico')
fs.writeFileSync(icoPath, buf)
console.log('ico bytes', buf.length)
for (const file of tmp) fs.unlinkSync(file)
