/** 从 Document 推断 URL */
async function load4() {
    const fallbackUrl =
        (document?.currentScript instanceof HTMLScriptElement
            ? document.currentScript.src
            : (document.currentScript?.href?.baseVal ?? '')) || document.location.href;
    return await body(fetch(new URL('../../lib/wasm_bg.wasm', fallbackUrl)));
}

/** 从 import.meta.url 推断 URL */
async function load2() {
    return await body(fetch(new URL('../../lib/wasm_bg.wasm', import.meta.url)));
}

/** 从 import.meta.resolve 推断 URL */
async function load3() {
    return await body(fetch(new URL(import.meta.resolve('../../lib/wasm_bg.wasm'))));
}

/** 由 esm 加载模块 */
async function load1() {
    /** 加载模块 */
    async function loadMod(mod: unknown): Promise<BufferSource> {
        if (mod && typeof mod == 'object' && 'default' in mod) {
            return loadMod(mod.default);
        }
        if (
            mod instanceof URL ||
            (typeof mod == 'string' &&
                (mod.startsWith('data:') ||
                    mod.startsWith('http:') ||
                    mod.startsWith('https:') ||
                    mod.startsWith('//')))
        ) {
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

    // use ?url to force vite to load as bytes
    // https://github.com/vitejs/vite/issues/12366
    const mod: unknown = await import('../../lib/wasm_bg.wasm?url', { with: { type: 'bytes' } });
    return await loadMod(mod);
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

export const module: Promise<BufferSource> = /* @__PURE__ */ (async () => {
    return load1()
        .catch(async () => load2())
        .catch(async () => load3())
        .catch(async () => load4());
})();
