import { svg2png, initialize } from 'svg2png-wasm';
import { readFile, writeFile } from 'node:fs/promises';
import path from 'node:path';

await initialize(readFile('./node_modules/svg2png-wasm/svg2png_wasm_bg.wasm'));

const SVG_PATH = path.resolve(import.meta.dirname, '../../website/static/favicon.svg');
const OUTPUT_PATH = path.resolve(import.meta.dirname, '../dist/icon.png');

const svgData = await readFile(SVG_PATH, 'utf8');
const pngData = await svg2png(svgData, { width: 128, height: 128 });

await writeFile(OUTPUT_PATH, pngData);
