import type { IDisposable, Monaco } from '../index.js';
import { setLanguageConfiguration } from './language-configuration.js';
import { registerMiraScriptTokensProvider } from './tokens-provider.js';

/** 注册 */
export function registerBasic(monaco: Monaco): IDisposable[] {
    return [...setLanguageConfiguration(monaco), ...registerMiraScriptTokensProvider(monaco)];
}
