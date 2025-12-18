/** 回退加载 */
async function loadFallback() {
    const fallbackUrl =
        (document?.currentScript instanceof HTMLScriptElement
            ? document.currentScript.src
            : (document.currentScript?.href?.baseVal ?? '')) || document.location.href;
    const url = new URL('../../lib/wasm_bg.wasm', fallbackUrl);
    if (await isValidUrl(url)) {
        return url;
    }
    throw new Error('Failed to load wasm module');
}

/** 检查 URL 是否有效 */
async function isValidUrl(url: URL): Promise<boolean> {
    const resp = await fetch(url, { method: 'HEAD' });
    return (resp.ok && resp.headers.get('Content-Type')?.startsWith('application/wasm')) ?? false;
}

/** 加载模块 */
async function loadMod(mod: unknown): Promise<BufferSource | URL> {
    if (mod && typeof mod == 'object' && 'default' in mod) {
        return loadMod(mod.default);
    }
    if (typeof mod == 'string') {
        if (mod.startsWith('data:')) {
            return new URL(mod);
        }
        if (mod.startsWith('http://') || mod.startsWith('https://') || mod.startsWith('/')) {
            const url = new URL(mod, document?.baseURI);
            if (await isValidUrl(url)) {
                return url;
            }
        }
        throw new Error('Failed to load wasm module');
    }
    if (typeof URL == 'function' && mod instanceof URL) {
        if (await isValidUrl(mod)) {
            return mod;
        }
        throw new Error('Failed to load wasm module');
    }
    if (ArrayBuffer.isView(mod) || mod instanceof ArrayBuffer) {
        return mod as ArrayBuffer;
    }
    if (mod instanceof WebAssembly.Module) {
        return mod as unknown as BufferSource;
    }
    throw new Error('Failed to load wasm module');
}

export const module: Promise<BufferSource | URL> = /* @__PURE__ */ (async () => {
    try {
        // use ?url to force vite to load as bytes
        // https://github.com/vitejs/vite/issues/12366
        return await loadMod(await import('../../lib/wasm_bg.wasm?url', { with: { type: 'bytes' } }));
    } catch {
        if (!import.meta.url) {
            return await loadFallback();
        } else {
            return await loadMod(new URL('../../lib/wasm_bg.wasm?url', import.meta.url));
        }
    }
})();
