import { expectString, VmLib } from '../../helpers.js';

export const starts_with = VmLib(
    (str, search) => {
        return expectString('str', str).startsWith(expectString('search', search));
    },
    {
        summary: '检查字符串是否以指定子串开头',
        params: {
            str: { type: 'string', description: '要检查的字符串' },
            search: { type: 'string', description: '要匹配的子串' },
        },
        returns: { type: 'boolean' },
        examples: ['starts_with("mira", "mi") // true'],
    },
);
export const ends_with = VmLib(
    (str, search) => {
        return expectString('str', str).endsWith(expectString('search', search));
    },
    {
        summary: '检查字符串是否以指定子串结尾',
        params: {
            str: { type: 'string', description: '要检查的字符串' },
            search: { type: 'string', description: '要匹配的子串' },
        },
        returns: { type: 'boolean' },
        examples: ['ends_with("mira", "ra") // true'],
    },
);

export const contains = VmLib(
    (str, search) => {
        return expectString('str', str).includes(expectString('search', search));
    },
    {
        summary: '检查字符串是否包含指定子串',
        params: {
            str: { type: 'string', description: '要检查的字符串' },
            search: { type: 'string', description: '要匹配的子串' },
        },
        returns: { type: 'boolean' },
        examples: ['contains("hello", "ll") // true'],
    },
);
