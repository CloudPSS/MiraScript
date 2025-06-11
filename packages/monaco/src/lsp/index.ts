import type { Monaco, IDisposable } from '../index.js';
import '../basic/index.js';

import './code-action-provider.js';
import './color-provider.js';
import './completion-item-provider.js';
import './definition-reference-provider.js';
import './diagnostics.js';
import './document-highlight-provider.js';
import './document-symbol-provider.js';
import './formatter.js';
import './hover-provider.js';
import './inlay-hints-provider.js';
import './range-provider.js';
import './rename-provider.js';
import './semantic-tokens-provider.js';
import './signature-help-provider.js';

/** 注册 LSP 相关的 Monaco 编辑器功能 */
export function registerLSP(monaco: Monaco): IDisposable[] {
    return [];
}
