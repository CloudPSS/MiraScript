import { toBoolean, toFormat, toNumber, toString } from '../../../helpers/convert/index.js';
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
            data: '要转换的数据',
            fallback: '转换失败时的返回值',
        },
        paramsType: { data: 'any', fallback: 'any' },
        returnsType: 'string | type(fallback)',
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
            data: '要转换的数据',
            fallback: '转换失败时的返回值',
        },
        paramsType: { data: 'any', fallback: 'any' },
        returnsType: 'number | type(fallback)',
        examples: ['to_number("1.5") // 1.5'],
    },
);

export const to_boolean = VmLib(
    (data, fallback) => {
        required('data', data, false);
        return toBoolean(data, fallback);
    },
    {
        summary: '将数据转换为布尔值',
        params: {
            data: '要转换的数据',
            fallback: '转换失败时的返回值',
        },
        paramsType: { data: 'any', fallback: 'any' },
        returnsType: 'boolean | type(fallback)',
        examples: ['to_boolean(nil) // false'],
    },
);

export const format = VmLib(
    (data, format) => {
        required('data', data, '');
        return toFormat(data, expectString('format', format));
    },
    {
        summary: '将数据格式化为指定格式的字符串',
        params: { data: '要格式化的数据', format: '格式字符串' },
        paramsType: { data: 'any', format: 'string' },
        returnsType: 'string',
        examples: ['format(12, ".3") // "12.000"'],
    },
);
