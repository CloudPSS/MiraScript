import { expectArray, expectString, VmLib } from '../../helpers.js';

export * from './trim.js';
export * from './case.js';
export * from './search.js';

export const chars = VmLib(
    (str) => {
        return Array.from(expectString('str', str));
    },
    {
        summary: '将字符串转换为字符数组',
        params: { str: { type: 'string', description: '要转换的字符串' } },
        returns: { type: 'string[]' },
        examples: ['chars("Mira") // ["M", "i", "r", "a"]'],
    },
);

export const replace = VmLib(
    (str, search, replacement = '') => {
        return expectString('str', str).replaceAll(
            expectString('search', search),
            expectString('replacement', replacement),
        );
    },
    {
        summary: '替换字符串中的指定子串',
        params: {
            str: { type: 'string', description: '要处理的字符串' },
            search: { type: 'string', description: '要替换的子串' },
            replacement: { type: 'string', description: '替换后的字符串' },
        },
        returns: { type: 'string' },
        examples: ['replace("foo bar foo", "foo", "baz") // "baz bar baz"'],
    },
);

export const split = VmLib(
    (str, separator = '') => {
        const s = expectString('str', str);
        const p = expectString('separator', separator);
        if (!p) return [...s];
        return s.split(p);
    },
    {
        summary: '将字符串拆分为子串数组',
        params: {
            str: { type: 'string', description: '要拆分的字符串' },
            separator: { type: 'string', description: '分隔符' },
        },
        returns: { type: 'string[]' },
        examples: ['split("a,b,c", ",") // ["a", "b", "c"]'],
    },
);

export const join = VmLib(
    (arr, separator = '') => {
        expectArray('arr', arr, null);
        const s = expectString('separator', separator);
        return arr.map((v) => expectString(null, v)).join(s);
    },
    {
        summary: '将字符串数组连接为单个字符串',
        params: {
            arr: { type: 'string[]', description: '要连接的字符串数组' },
            separator: { type: 'string', description: '分隔符' },
        },
        returns: { type: 'string' },
        examples: ['join(["a", "b", "c"], "-") // "a-b-c"'],
    },
);
