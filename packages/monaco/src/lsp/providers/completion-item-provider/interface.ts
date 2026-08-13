import type { VmValue, VmExtern, VmModule } from '@mirascript/mirascript';
import type { languages } from '../../../monaco-api.js';
import type { MonacoContext } from '../base.js';

/** 扩展完成项 */
export interface CustomCompletionItem extends languages.CompletionItem {
    /** 是否为字段 */
    isField: boolean;
    /** 对应的父值 */
    vmParent?: MonacoContext | VmExtern | VmModule;
    /** 对应的变量值 */
    vmValue?: VmValue;
}

export const DESC_GLOBAL = '(global)';
export const DESC_LOCAL = '(local)';
export const DESC_FIELD = '(field)';
