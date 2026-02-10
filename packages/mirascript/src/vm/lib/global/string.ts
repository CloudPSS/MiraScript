import { expectArray, expectString, VmLib } from '../helpers.js';

export const chars = VmLib(
    (str) => {
        return Array.from(expectString('str', str));
    },
    {
        summary: '将字符串转换为字符数组',
        params: { str: '要转换的字符串' },
        paramsType: { str: 'string' },
        returnsType: 'string[]',
        examples: ['chars("Mira") // ["M", "i", "r", "a"]'],
    },
);

export const starts_with = VmLib(
    (str, search) => {
        return expectString('str', str).startsWith(expectString('search', search));
    },
    {
        summary: '检查字符串是否以指定子串开头',
        params: { str: '要检查的字符串', search: '要匹配的子串' },
        paramsType: { str: 'string', search: 'string' },
        returnsType: 'boolean',
        examples: ['starts_with("mira", "mi") // true'],
    },
);
export const ends_with = VmLib(
    (str, search) => {
        return expectString('str', str).endsWith(expectString('search', search));
    },
    {
        summary: '检查字符串是否以指定子串结尾',
        params: { str: '要检查的字符串', search: '要匹配的子串' },
        paramsType: { str: 'string', search: 'string' },
        returnsType: 'boolean',
        examples: ['ends_with("mira", "ra") // true'],
    },
);

export const contains = VmLib(
    (str, search) => {
        return expectString('str', str).includes(expectString('search', search));
    },
    {
        summary: '检查字符串是否包含指定子串',
        params: { str: '要检查的字符串', search: '要匹配的子串' },
        paramsType: { str: 'string', search: 'string' },
        returnsType: 'boolean',
        examples: ['contains("hello", "ll") // true'],
    },
);

export const trim_start = VmLib(
    (str) => {
        return expectString('str', str).trimStart();
    },
    {
        summary: '去除字符串开头的空白字符',
        params: { str: '要处理的字符串' },
        paramsType: { str: 'string' },
        returnsType: 'string',
        examples: ['trim_start("  mira") // "mira"'],
    },
);

export const trim_end = VmLib(
    (str) => {
        return expectString('str', str).trimEnd();
    },
    {
        summary: '去除字符串结尾的空白字符',
        params: { str: '要处理的字符串' },
        paramsType: { str: 'string' },
        returnsType: 'string',
        examples: ['trim_end("mira  ") // "mira"'],
    },
);

export const trim = VmLib(
    (str) => {
        return expectString('str', str).trim();
    },
    {
        summary: '去除字符串两端的空白字符',
        params: { str: '要处理的字符串' },
        paramsType: { str: 'string' },
        returnsType: 'string',
        examples: ['trim("  mira  ") // "mira"'],
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
        params: { str: '要处理的字符串', search: '要替换的子串', replacement: '替换后的字符串' },
        paramsType: { str: 'string', search: 'string', replacement: 'string' },
        returnsType: 'string',
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
        params: { str: '要拆分的字符串', separator: '分隔符' },
        paramsType: { str: 'string', separator: 'string' },
        returnsType: 'string[]',
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
        params: { arr: '要连接的字符串数组', separator: '分隔符' },
        paramsType: { arr: 'string[]', separator: 'string' },
        returnsType: 'string',
        examples: ['join(["a", "b", "c"], "-") // "a-b-c"'],
    },
);
