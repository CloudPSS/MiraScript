import type { IDisposable } from '../monaco-api.js';
import { setLanguageConfiguration } from './language-configuration.js';
import { registerMiraScriptTokensProvider } from './tokens-provider.js';

/** 注册 */
export function registerBasic(): IDisposable[] {
    return [...setLanguageConfiguration(), ...registerMiraScriptTokensProvider()];
}
