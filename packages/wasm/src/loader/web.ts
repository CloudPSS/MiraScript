import type { InitInput } from '../../lib/wasm.js';

import.meta.url ||=
    document.currentScript instanceof HTMLScriptElement
        ? document.currentScript.src
        : (document.currentScript?.href.baseVal ?? '');
import.meta.url ||= document.location.href;

export const module: Promise<InitInput> = fetch(new URL('../../lib/wasm_bg.wasm', import.meta.url));
