import type { InitInput } from '../../lib/wasm.js';

/** 回退加载 */
async function loadFallback() {
    const fallbackUrl =
        (document?.currentScript instanceof HTMLScriptElement
            ? document.currentScript.src
            : (document.currentScript?.href?.baseVal ?? '')) || document.location.href;
    return await fetch(new URL('../../lib/wasm_bg.wasm', fallbackUrl));
}

export const module: Promise<InitInput> = /* @__PURE__ */ (async () => {
    if (!import.meta.url) {
        return await loadFallback();
    }
    return await fetch(new URL('../../lib/wasm_bg.wasm?url', import.meta.url));
})();
