import type { VmContext } from '@mirascript/mirascript';
import { globals as g } from './globals.js';
import { ConsoleManager } from './console-manager.js';

export let monaco: typeof import('@private/monaco-editor');
export let mirascript: typeof import('@mirascript/mirascript');
export let mirascriptSubtle: typeof import('@mirascript/mirascript/subtle');
export let globals: VmContext;
export let mirascriptBc: import('@mirascript/bindings/wasm').BcModuleType;
export const consoleManager = new ConsoleManager(document.querySelector<HTMLDivElement>('#console-output')!);

/** 加载 */
async function load() {
    mirascript = await import('@mirascript/mirascript');
    mirascriptSubtle = await import('@mirascript/mirascript/subtle');
    monaco = await import('@private/monaco-editor');
    const { loadModule } = await import('@mirascript/bindings/wasm');
    mirascriptBc = await loadModule();

    mirascript.configCheckpoint(500);
    globals = g(consoleManager);
    const { registerMiraScript } = await import('@mirascript/monaco');
    registerMiraScript(monaco, () => globals);

    // 暴露到全局以便调试
    Object.defineProperty(globalThis, 'mirascript', {
        value: Object.freeze({
            __proto__: null,
            ...mirascript,
            subtle: Object.freeze({
                __proto__: null,
                ...mirascriptSubtle,
            }),
        }),
    });
}

export const ready = load();
