import fs from 'node:fs/promises';
import { fileURLToPath } from 'node:url';

const file = /* @__PURE__ */ fileURLToPath(new URL('../../lib/wasm_bg.wasm', import.meta.url));
export const module: Promise<BufferSource> = /* @__PURE__ */ fs.readFile(file);
