import type { InitInput } from '../../lib/wasm.js';

/** 回退加载 */
async function loadFallback() {
    const fallbackUrl =
        (document?.currentScript instanceof HTMLScriptElement
            ? document.currentScript.src
            : (document.currentScript?.href?.baseVal ?? '')) || document.location.href;
    return await fetch(new URL('../../lib/wasm_bg.wasm', fallbackUrl));
}

/** 加载模块 */
async function loadMod(mod: unknown): Promise<InitInput> {
    if (mod && typeof mod == 'object' && 'default' in mod) {
        return loadMod(mod.default);
    }
    if (typeof mod == 'string' && mod.startsWith('data:')) {
        return fetch(mod);
    }
    if (
        mod instanceof Response ||
        ArrayBuffer.isView(mod) ||
        mod instanceof ArrayBuffer ||
        mod instanceof WebAssembly.Module
    ) {
        return mod;
    }
    throw new Error('Failed to load wasm module');
}

export const module: Promise<InitInput> = /* @__PURE__ */ (async () => {
    try {
        // use ?url to force vite to load as bytes
        // https://github.com/vitejs/vite/issues/12366
        return await loadMod(await import('../../lib/wasm_bg.wasm?url', { with: { type: 'bytes' } }));
    } catch {
        if (!import.meta.url) {
            return await loadFallback();
        } else {
            return await fetch(new URL('../../lib/wasm_bg.wasm?url', import.meta.url));
        }
    }
})();
