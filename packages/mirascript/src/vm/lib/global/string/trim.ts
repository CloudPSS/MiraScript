import { expectString, VmLib } from '../../helpers.js';

export const trim_start = VmLib(
    (str) => {
        return expectString('str', str).trimStart();
    },
    {
        summary: '去除字符串开头的空白字符',
        params: { str: { type: 'string', description: '要处理的字符串' } },
        returns: { type: 'string' },
        examples: ['trim_start("  mira") // "mira"'],
    },
);

export const trim_end = VmLib(
    (str) => {
        return expectString('str', str).trimEnd();
    },
    {
        summary: '去除字符串结尾的空白字符',
        params: { str: { type: 'string', description: '要处理的字符串' } },
        returns: { type: 'string' },
        examples: ['trim_end("mira  ") // "mira"'],
    },
);

export const trim = VmLib(
    (str) => {
        return expectString('str', str).trim();
    },
    {
        summary: '去除字符串两端的空白字符',
        params: { str: { type: 'string', description: '要处理的字符串' } },
        returns: { type: 'string' },
        examples: ['trim("  mira  ") // "mira"'],
    },
);
