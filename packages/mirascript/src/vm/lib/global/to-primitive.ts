import { toFormat, toNumber, toString } from '../../../helpers/convert/index.js';
import { NotNumber } from '../../../helpers/utils.js';
import { expectString, required, VmLib } from '../helpers.js';

export const to_string = VmLib(
    (data, fallback) => {
        required('data', data, '');
        return toString(data, fallback);
    },
    {
        summary: '将数据转换为字符串',
        params: {
            data: { type: 'any', description: '要转换的数据' },
            fallback: { type: 'any', description: '转换失败时的返回值' },
        },
        returns: { type: 'string | type(fallback)' },
        examples: ['to_string([1, 2]) // "1, 2"'],
    },
);

export const to_number = VmLib(
    (data, fallback) => {
        required('data', data, NotNumber);
        return toNumber(data, fallback);
    },
    {
        summary: '将数据转换为数字',
        params: {
            data: { type: 'string | number | boolean', description: '要转换的数据' },
            fallback: { type: 'any', description: '转换失败时的返回值' },
        },
        returns: { type: 'number | type(fallback)' },
        examples: ['to_number("1.5") // 1.5'],
    },
);

export const format = VmLib(
    (data, format) => {
        required('data', data, '');
        return toFormat(data, expectString('format', format));
    },
    {
        summary: '将数据格式化为指定格式的字符串',
        params: {
            data: { type: 'any', description: '要格式化的数据' },
            format: { type: 'string', description: '格式字符串' },
        },
        returns: { type: 'string' },
        examples: ['format(12, ".3") // "12.000"'],
    },
);
