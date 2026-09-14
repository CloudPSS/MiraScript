import type { VmAny } from '../../../../index.js';
import { expectString, VmLib } from '../../helpers.js';

/** 生成函数 */
function build(
    f: (str: string, search: string) => boolean,
    summary: string,
    examples: string[],
): VmLib<(str: VmAny, search: VmAny) => boolean> {
    return VmLib(
        (str, search) => {
            const s_str = expectString('str', str);
            const s_search = expectString('search', search);
            return f(s_str, s_search);
        },
        {
            summary,
            params: {
                str: { type: 'string', description: '要检查的字符串' },
                search: { type: 'string', description: '要匹配的子串' },
            },
            returns: { type: 'boolean' },
            examples,
        },
    );
}

export const starts_with = build(
    (str, search) => {
        return str.startsWith(search);
    },
    '检查字符串是否以指定子串开头',
    ['starts_with("mira", "mi") // true'],
);
export const ends_with = build(
    (str, search) => {
        return str.endsWith(search);
    },
    '检查字符串是否以指定子串结尾',
    ['ends_with("mira", "ra") // true'],
);

export const contains = build(
    (str, search) => {
        return str.includes(search);
    },
    '检查字符串是否包含指定子串',
    ['contains("hello", "ll") // true'],
);
