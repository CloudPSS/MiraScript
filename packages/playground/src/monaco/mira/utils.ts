import type { VmFunctionInfo } from '../../vm/types/function';

/** 生成函数签名 */
export function signature(id: string | undefined, info: VmFunctionInfo): string {
    const prefix = id ? `fn ${id}` : 'fn';
    const params = info.params ? `(${Object.keys(info.params).join(', ')})` : '(..)';
    return `${prefix}${params}`;
}

const CODEBLOCK_FENCE = '`'.repeat(16);
/** 获取代码块格式化字符串 */
export function codeblock(value: string): string {
    return `\n${CODEBLOCK_FENCE}mirascript\n${value}\n${CODEBLOCK_FENCE}\n`;
}
