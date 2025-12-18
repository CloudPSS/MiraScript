import fs from 'node:fs/promises';

const file = /* @__PURE__ */ new URL('../../lib/wasm_bg.wasm', import.meta.url);
export const module: Promise<BufferSource> = /* @__PURE__ */ fs.readFile(file);
