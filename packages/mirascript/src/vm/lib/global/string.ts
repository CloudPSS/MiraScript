import { $ToString } from '../../operations.js';
import { required, VmLib } from '../helpers.js';

export const chars = VmLib(
    (str) => {
        required('str', str, null);
        return [...$ToString(str)];
    },
    {
        summary: '将字符串转换为字符数组',
        params: { str: '要转换的字符串' },
        paramsType: { str: 'string' },
        returnsType: '[string]',
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
