#!/usr/bin/env node
/**
 * Generates the synthetic test fixtures.
 *
 * The fixtures are generated instead of committed so that the
 * repository stays free of binary blobs and so that every fixture is
 * known to be synthetic and free of third party content
 * (`TESTING.md` section 3).
 *
 * Usage: node scripts/generate-fixtures.mjs
 */

import { deflateSync } from 'node:zlib';
import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');
const OUTPUT = join(ROOT, 'tests', 'fixtures', 'generated');

/** CRC-32 table, needed for the PNG chunk checksums. */
const CRC_TABLE = (() => {
  const table = new Uint32Array(256);
  for (let n = 0; n < 256; n += 1) {
    let c = n;
    for (let k = 0; k < 8; k += 1) {
      c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    }
    table[n] = c >>> 0;
  }
  return table;
})();

/** Computes the CRC-32 of a buffer. */
function crc32(buffer) {
  let crc = 0xffffffff;
  for (const byte of buffer) {
    crc = CRC_TABLE[(crc ^ byte) & 0xff] ^ (crc >>> 8);
  }
  return (crc ^ 0xffffffff) >>> 0;
}

/** Builds one PNG chunk: length, type, data and CRC. */
function chunk(type, data) {
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length, 0);

  const typeAndData = Buffer.concat([Buffer.from(type, 'ascii'), data]);

  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(typeAndData), 0);

  return Buffer.concat([length, typeAndData, crc]);
}

/**
 * Encodes an RGB image as a PNG.
 *
 * @param {number} width Image width in pixels.
 * @param {number} height Image height in pixels.
 * @param {(x: number, y: number) => [number, number, number]} pixel Colour of a pixel.
 */
function encodePng(width, height, pixel) {
  const header = Buffer.alloc(13);
  header.writeUInt32BE(width, 0);
  header.writeUInt32BE(height, 4);
  header.writeUInt8(8, 8); // bit depth
  header.writeUInt8(2, 9); // colour type: truecolour
  header.writeUInt8(0, 10); // compression
  header.writeUInt8(0, 11); // filter
  header.writeUInt8(0, 12); // interlace

  const stride = width * 3;
  const raw = Buffer.alloc((stride + 1) * height);
  for (let y = 0; y < height; y += 1) {
    const rowStart = y * (stride + 1);
    raw[rowStart] = 0; // filter type: none
    for (let x = 0; x < width; x += 1) {
      const [r, g, b] = pixel(x, y);
      const offset = rowStart + 1 + x * 3;
      raw[offset] = r;
      raw[offset + 1] = g;
      raw[offset + 2] = b;
    }
  }

  return Buffer.concat([
    Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]),
    chunk('IHDR', header),
    chunk('IDAT', deflateSync(raw)),
    chunk('IEND', Buffer.alloc(0)),
  ]);
}

/** Writes a fixture and reports it. */
function write(relativePath, contents) {
  const target = join(OUTPUT, relativePath);
  mkdirSync(dirname(target), { recursive: true });
  writeFileSync(target, contents);
  console.log(`  ${relativePath} (${contents.length} bytes)`);
}

mkdirSync(OUTPUT, { recursive: true });
console.log(`Generating fixtures in ${OUTPUT}`);

// A valid, minimal image: the smallest input an adapter can be tested
// with.
write(
  'valid/minimal.png',
  encodePng(1, 1, () => [255, 0, 0]),
);

// A valid image with a gradient, used for visual comparisons.
write(
  'valid/gradient.png',
  encodePng(64, 64, (x, y) => [(x * 4) % 256, (y * 4) % 256, 128]),
);

// A larger valid image, used for the file size bands in the resource
// tests.
write(
  'valid/large.png',
  encodePng(512, 512, (x, y) => [(x + y) % 256, (x * 2) % 256, (y * 2) % 256]),
);

// A valid image whose name contains spaces and a non-ASCII character.
write(
  'valid/nama file ünïcode ✓.png',
  encodePng(2, 2, () => [0, 128, 255]),
);

// A valid image whose name contains characters that are meaningful to a
// shell. The name must be treated as data, never as a command.
write(
  'valid/report; rm -rf $(whoami).png',
  encodePng(2, 2, () => [10, 20, 30]),
);

// A zero byte file: the engine must reject it, not crash on it.
write('invalid/empty.png', Buffer.alloc(0));

// Random bytes with a PNG name: the signature check has to catch it.
write('invalid/corrupt.png', Buffer.from('not a png at all', 'utf8'));

// A text file with a PNG name: the extension and the content disagree.
write('invalid/mismatch.png', Buffer.from('This is plain text.\n', 'utf8'));

// A valid PNG that was cut in half: the engine must fail without
// producing an output file.
const truncated = encodePng(32, 32, () => [200, 100, 50]).subarray(0, 40);
write('invalid/truncated.png', truncated);

console.log('Done.');
