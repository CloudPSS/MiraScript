import { expectString, VmLib } from '../../helpers.js';

export const to_uppercase = VmLib(
    (str) => {
        return expectString('str', str).toUpperCase();
    },
    {
        summary: '将字符串转换为大写',
        params: { str: { type: 'string', description: '要转换的字符串' } },
        returns: { type: 'string' },
        examples: ['to_uppercase("mira") // "MIRA"'],
    },
);

export const to_lowercase = VmLib(
    (str) => {
        return expectString('str', str).toLowerCase();
    },
    {
        summary: '将字符串转换为小写',
        params: { str: { type: 'string', description: '要转换的字符串' } },
        returns: { type: 'string' },
        examples: ['to_lowercase("MIRA") // "mira"'],
    },
);
