import fs from 'node:fs/promises';
import type { InitInput } from '../../lib/wasm.js';

const file = /* @__PURE__ */ new URL('../../lib/wasm_bg.wasm', import.meta.url);
export const module: Promise<InitInput> = /* @__PURE__ */ fs.readFile(file);
