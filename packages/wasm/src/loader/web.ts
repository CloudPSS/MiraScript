import type { InitInput } from '../../lib/wasm.js';

export const module: Promise<InitInput> = fetch(new URL('../../lib/wasm_bg.wasm', import.meta.url));
