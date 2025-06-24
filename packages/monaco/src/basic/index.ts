import type { IDisposable } from '../monaco-api.js';
import { setLanguageConfiguration, configuration } from './language-configuration.js';
import { registerMiraScriptTokensProvider } from './tokens-provider.js';

export { configuration };
/** 注册 */
export function registerBasic(): IDisposable[] {
    return [...setLanguageConfiguration(), ...registerMiraScriptTokensProvider()];
}
