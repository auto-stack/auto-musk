#!/usr/bin/env node
// PLAN-093 T-01 probe fixture: deterministic 240x160 PNG with a known marker
// rectangle. Hand-rolled encoder (zlib deflate of raw scanlines) — no deps.
// Output: tests/ui-parity/probes/fixture-frame.png
// Layout: dark background; blue marker square at x=150..189, y=90..113;
//         amber corner tick 8x8 at (0,0) for orientation sanity.

import fs from 'node:fs';
import path from 'node:path';
import zlib from 'node:zlib';
import { fileURLToPath } from 'node:url';

const W = 240, H = 160;
const MARKER = { x: 150, y: 90, w: 40, h: 24 };

const raw = Buffer.alloc(H * (1 + W * 3));
for (let y = 0; y < H; y++) {
  const row = y * (1 + W * 3);
  raw[row] = 0; // filter: none
  for (let x = 0; x < W; x++) {
    const i = row + 1 + x * 3;
    let r = 32, g = 36, b = 44; // background #20242c
    if (x < 8 && y < 8) { r = 245; g = 158; b = 11; } // amber orientation tick
    if (x >= MARKER.x && x < MARKER.x + MARKER.w && y >= MARKER.y && y < MARKER.y + MARKER.h) {
      r = 59; g = 130; b = 246; // blue marker (tailwind blue-500)
    }
    raw[i] = r; raw[i + 1] = g; raw[i + 2] = b;
  }
}

function chunk(type, data) {
  const len = Buffer.alloc(4); len.writeUInt32BE(data.length);
  const typeBuf = Buffer.from(type, 'ascii');
  const crcBuf = Buffer.concat([typeBuf, data]);
  const crc = Buffer.alloc(4); crc.writeUInt32BE(zlib.crc32 ? zlib.crc32(crcBuf) : crc32(crcBuf));
  return Buffer.concat([len, typeBuf, data, crc]);
}

// zlib.crc32 exists on node >=20.15; fallback for safety.
function crc32(buf) {
  let c, table = [];
  for (let n = 0; n < 256; n++) {
    c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    table[n] = c >>> 0;
  }
  let crc = 0xffffffff;
  for (const b of buf) crc = table[(crc ^ b) & 0xff] ^ (crc >>> 8);
  return (crc ^ 0xffffffff) >>> 0;
}

const ihdr = Buffer.alloc(13);
ihdr.writeUInt32BE(W, 0); ihdr.writeUInt32BE(H, 4);
ihdr[8] = 8;  // bit depth
ihdr[9] = 2;  // color type: truecolor RGB
const png = Buffer.concat([
  Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
  chunk('IHDR', ihdr),
  chunk('IDAT', zlib.deflateSync(raw, { level: 9 })),
  chunk('IEND', Buffer.alloc(0)),
]);

const out = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', '..', 'tests', 'ui-parity', 'probes', 'fixture-frame.png');
fs.mkdirSync(path.dirname(out), { recursive: true });
fs.writeFileSync(out, png);
console.log(`wrote ${out} (${png.length} bytes, ${W}x${H}, marker=${JSON.stringify(MARKER)})`);
