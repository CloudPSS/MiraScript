import type { InitInput } from '../../lib/wasm.js';

export const module: Promise<InitInput> = /* @__PURE__ */ (async () => {
    try {
        if (import.meta.url) {
            return await fetch(new URL('../../lib/wasm_bg.wasm?url', import.meta.url));
        }
        const fallbackUrl =
            (document?.currentScript instanceof HTMLScriptElement
                ? document.currentScript.src
                : (document.currentScript?.href?.baseVal ?? '')) || document.location.href;
        return await fetch(new URL('../../lib/wasm_bg.wasm?url', fallbackUrl));
    } catch {
        // @ts-expect-error load as module
        const mod = (await import('../../lib/wasm_bg.wasm?url')) as { default: unknown };
        if (typeof mod.default == 'string' || mod.default instanceof WebAssembly.Module) return mod.default;
        return mod;
    }
})();
