import type { VmFunctionInfo } from '../../vm/types/function';

/** 生成函数签名 */
export function signature(id: string | undefined, info: VmFunctionInfo): string {
    const prefix = id ? `\0fn ${id}` : 'fn';
    let params;
    if (!info.params) {
        params = '(..)';
    } else {
        const paramItems = Object.keys(info.params).map((key) => {
            const type = info.paramsType?.[key];
            const typeStr = type ? `: ${type}` : '';
            return `${key}${typeStr}`;
        });
        const len = paramItems.reduce((acc, item) => acc + item.length, 0);
        if (len <= 60) {
            params = `(${paramItems.join(', ')})`;
        } else {
            params = `(\n${paramItems.map((item) => `  ${item},`).join('\n')}\n)`;
        }
    }
    const returns = info.returnsType ? ` -> ${info.returnsType}` : '';
    return `${prefix}${params}${returns}`;
}

/** 生成函数文档 */
export function document(id: string | undefined, info: VmFunctionInfo): string {
    const signatureStr = signature(id, info);
    const doc = [
        codeblock(signatureStr),
        info.summary || '',
        info.params
            ? Object.entries(info.params)
                  .map(([key, value]) => `- \`${key}\`: ${value}`)
                  .join('\n')
            : '',
        info.returns ? `**返回值**: ${info.returns}` : '',
    ];
    return doc.join('\n\n');
}

const CODEBLOCK_FENCE = '`'.repeat(16);
/** 获取代码块格式化字符串 */
export function codeblock(value: string): string {
    return `\n${CODEBLOCK_FENCE}mirascript\n${value}\n${CODEBLOCK_FENCE}\n`;
}
