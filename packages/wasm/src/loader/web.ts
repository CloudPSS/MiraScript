/** 回退加载 */
async function loadFallback() {
    const fallbackUrl =
        (document?.currentScript instanceof HTMLScriptElement
            ? document.currentScript.src
            : (document.currentScript?.href?.baseVal ?? '')) || document.location.href;
    return await body(fetch(new URL('../../lib/wasm_bg.wasm', fallbackUrl)));
}

/** 获取模块的响应体 */
async function body(response: Response | Promise<Response>): Promise<BufferSource> {
    const resp = await response;
    if (resp.ok) {
        return await resp.arrayBuffer();
    } else {
        throw new Error(`Failed to fetch wasm module: ${resp.status} ${resp.statusText}`);
    }
}

/** 加载模块 */
async function loadMod(mod: unknown): Promise<BufferSource> {
    if (mod && typeof mod == 'object' && 'default' in mod) {
        return loadMod(mod.default);
    }
    if (typeof mod == 'string' && mod.startsWith('data:')) {
        return await body(fetch(mod));
    }
    if (mod instanceof Response) {
        return await body(mod);
    }
    if (ArrayBuffer.isView(mod) || mod instanceof ArrayBuffer) {
        return mod as ArrayBuffer;
    }
    if (mod instanceof WebAssembly.Module) {
        return mod as unknown as BufferSource;
    }
    throw new Error('Failed to load wasm module');
}

export const module: Promise<BufferSource> = /* @__PURE__ */ (async () => {
    try {
        // use ?url to force vite to load as bytes
        // https://github.com/vitejs/vite/issues/12366
        return await loadMod(await import('../../lib/wasm_bg.wasm?url', { with: { type: 'bytes' } }));
    } catch {
        if (!import.meta.url) {
            return await loadFallback();
        } else {
            return await body(fetch(new URL('../../lib/wasm_bg.wasm?url', import.meta.url)));
        }
    }
})();
