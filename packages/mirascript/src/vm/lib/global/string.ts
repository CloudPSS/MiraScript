import { $ToString } from '../../operations.js';
import { expectArray, required, VmLib } from '../_helpers.js';

export const chars = VmLib(
    (str) => {
        required('str', str, null);
        return [...$ToString(str)];
    },
    {
        summary: '将字符串转换为字符数组',
        params: { str: '要转换的字符串' },
        paramsType: { str: 'string' },
        returnsType: 'string[]',
    },
);

export const starts_with = VmLib(
    (str, search) => {
        required('str', str, null);
        required('search', search, null);
        return $ToString(str).startsWith($ToString(search));
    },
    {
        summary: '检查字符串是否以指定子串开头',
        params: { str: '要检查的字符串', search: '要匹配的子串' },
        paramsType: { str: 'string', search: 'string' },
        returnsType: 'boolean',
    },
);
export const ends_with = VmLib(
    (str, search) => {
        required('str', str, null);
        required('search', search, null);
        return $ToString(str).endsWith($ToString(search));
    },
    {
        summary: '检查字符串是否以指定子串结尾',
        params: { str: '要检查的字符串', search: '要匹配的子串' },
        paramsType: { str: 'string', search: 'string' },
        returnsType: 'boolean',
    },
);

export const contains = VmLib(
    (str, search) => {
        required('str', str, null);
        required('search', search, null);
        return $ToString(str).includes($ToString(search));
    },
    {
        summary: '检查字符串是否包含指定子串',
        params: { str: '要检查的字符串', search: '要匹配的子串' },
        paramsType: { str: 'string', search: 'string' },
        returnsType: 'boolean',
    },
);

export const trim_start = VmLib(
    (str) => {
        required('str', str, null);
        return $ToString(str).trimStart();
    },
    {
        summary: '去除字符串开头的空白字符',
        params: { str: '要处理的字符串' },
        paramsType: { str: 'string' },
        returnsType: 'string',
    },
);

export const trim_end = VmLib(
    (str) => {
        required('str', str, null);
        return $ToString(str).trimEnd();
    },
    {
        summary: '去除字符串结尾的空白字符',
        params: { str: '要处理的字符串' },
        paramsType: { str: 'string' },
        returnsType: 'string',
    },
);

export const trim = VmLib(
    (str) => {
        required('str', str, null);
        return $ToString(str).trim();
    },
    {
        summary: '去除字符串两端的空白字符',
        params: { str: '要处理的字符串' },
        paramsType: { str: 'string' },
        returnsType: 'string',
    },
);

export const replace = VmLib(
    (str, search, replacement = '') => {
        required('str', str, null);
        required('search', search, str);
        return $ToString(str).replaceAll($ToString(search), $ToString(replacement));
    },
    {
        summary: '替换字符串中的指定子串',
        params: { str: '要处理的字符串', search: '要替换的子串', replacement: '替换后的字符串' },
        paramsType: { str: 'string', search: 'string', replacement: 'string' },
        returnsType: 'string',
    },
);

export const split = VmLib(
    (str, separator = '') => {
        required('str', str, null);
        const s = $ToString(str);
        const p = $ToString(separator);
        if (!p) return [...s];
        return s.split(p);
    },
    {
        summary: '将字符串拆分为子串数组',
        params: { str: '要拆分的字符串', separator: '分隔符' },
        paramsType: { str: 'string', separator: 'string' },
        returnsType: 'string[]',
    },
);

export const join = VmLib(
    (arr, separator = '') => {
        expectArray('arr', arr, null);
        const s = $ToString(separator);
        return arr.map((v) => $ToString(v)).join(s);
    },
    {
        summary: '将字符串数组连接为单个字符串',
        params: { arr: '要连接的字符串数组', separator: '分隔符' },
        paramsType: { arr: 'string[]', separator: 'string' },
        returnsType: 'string',
    },
);
