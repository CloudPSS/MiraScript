/** 从 Document 推断 URL */
async function loadDocument() {
    const fallbackUrl =
        (document?.currentScript instanceof HTMLScriptElement
            ? document.currentScript.src
            : (document.currentScript?.href?.baseVal ?? '')) || document.location.href;
    return await body(fetch(new URL('../../lib/wasm_bg.wasm', fallbackUrl)));
}

/** 从 import.meta.url 推断 URL */
async function loadUrl() {
    return await body(fetch(new URL('../../lib/wasm_bg.wasm', import.meta.url)));
}

/** 由 esm 加载模块 */
async function loadEsm() {
    // use ?url to force vite to load as bytes
    // https://github.com/vitejs/vite/issues/12366
    const m: unknown = await import('../../lib/wasm_bg.wasm?url', { with: { type: 'bytes' } });
    return await mod(m);
}

/** 由 esm.sh 加载模块 */
async function loadEsmSh() {
    const url = new URL(import.meta.url);
    const { pathname } = url;
    if (!pathname.includes('/@mirascript/wasm')) {
        throw new Error('Not loaded from esm.sh');
    }
    const segments = pathname.split('/');
    let newUrl = url.origin;
    for (let i = 0; i < segments.length; i++) {
        const segment = segments[i]!;
        newUrl += segment + '/';
        if (i >= 1 && segments[i - 1] === '@mirascript' && (segment === 'wasm' || segment?.startsWith('wasm@'))) {
            break;
        }
    }
    newUrl += 'lib/wasm_bg.wasm';
    return await body(fetch(newUrl));
}

/** 加载模块 */
async function mod(m: unknown): Promise<BufferSource> {
    if (m && typeof m == 'object' && 'default' in m) {
        return mod(m.default);
    }
    if (
        m instanceof URL ||
        (typeof m == 'string' &&
            (m.startsWith('data:') || m.startsWith('http:') || m.startsWith('https:') || m.startsWith('//')))
    ) {
        return await body(fetch(m));
    }
    if (m instanceof Response) {
        return await body(m);
    }
    if (ArrayBuffer.isView(m) || m instanceof ArrayBuffer) {
        return m as ArrayBuffer;
    }
    if (m instanceof WebAssembly.Module) {
        return m as unknown as BufferSource;
    }
    throw new Error('Failed to load wasm module');
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
    const candidates = [loadEsmSh, loadEsm, loadUrl, loadDocument];
    for (const candidate of candidates) {
        try {
            const mod = await candidate();
            if (mod == null) continue;
            return mod;
        } catch {
            // ignore
        }
    }
    throw new Error('Failed to load wasm module');
})();
