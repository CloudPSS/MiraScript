import type { InitInput } from '../../lib/wasm.js';

import.meta.url ||=
    document.currentScript instanceof HTMLScriptElement
        ? document.currentScript.src
        : (document.currentScript?.href.baseVal ?? '');
import.meta.url ||= document.location.href;

export const module: Promise<InitInput> = (async () => {
    try {
        return await fetch(new URL('../../lib/wasm_bg.wasm?url', import.meta.url));
    } catch {
        // @ts-expect-error load as module
        const mod = (await import('../../lib/wasm_bg.wasm?url')) as { default: unknown };
        if (typeof mod.default == 'string' || mod.default instanceof WebAssembly.Module) return mod.default;
        return mod;
    }
})();
