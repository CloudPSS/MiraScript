import { ready } from '@mirascript/wasm';
import type { IDisposable } from '../monaco-api.js';
import { setLanguageConfiguration, configuration } from './language-configuration.js';
import { registerMiraScriptTokensProvider } from './tokens-provider.js';

export { configuration };
/** 注册 */
export async function registerBasic(): Promise<IDisposable[]> {
    await ready;
    return [...setLanguageConfiguration(), ...registerMiraScriptTokensProvider()];
}
