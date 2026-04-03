import fs from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { loadModule as loadWeb } from './web.js';

/** 加载 wasm 模块 */
export async function loadModule(): Promise<BufferSource | Response> {
    const url = new URL('../../lib/wasm_bg.wasm', import.meta.url);
    if (url.protocol !== 'file:') {
        return await loadWeb();
    }
    const file = fileURLToPath(url);
    return await fs.readFile(file);
}
