import fs from 'node:fs/promises';
import { fileURLToPath } from 'node:url';

/** 加载 wasm 模块 */
async function loadModule(): Promise<BufferSource | Response> {
    const url = new URL('../../lib/wasm_bg.wasm', import.meta.url);
    if (url.protocol !== 'file:') {
        const { module } = await import('./web.js');
        return await module;
    }
    const file = fileURLToPath(url);
    return await fs.readFile(file);
}

export const module: Promise<BufferSource | Response> = /* @__PURE__ */ loadModule();
