import type { IDisposable } from '../monaco-api.js';
import { setLanguageConfiguration, configuration } from './language-configuration.js';
import { registerMiraScriptTokensProvider } from './tokens-provider.js';

export { configuration };
export { registerMiraScriptTokensProvider };

/** 注册 */
export async function registerBasic(): Promise<IDisposable[]> {
    const { loadModule } = await import('@mirascript/bindings/wasm');
    await loadModule();
    return [...setLanguageConfiguration(), ...registerMiraScriptTokensProvider()];
}
