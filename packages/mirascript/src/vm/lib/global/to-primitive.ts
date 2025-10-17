import { $ToString, $ToNumber, $ToBoolean, $Format } from '../../operations.ts';
import { required, VmLib } from '../_helpers.ts';

export const to_string = VmLib(
    (data) => {
        required('data', data, '');
        return $ToString(data);
    },
    {
        summary: '将数据转换为字符串',
        params: { data: '要转换的数据' },
        paramsType: { data: 'any' },
        returnsType: 'string',
        examples: ['to_string([1, 2]) // "1, 2"'],
    },
);

export const to_number = VmLib(
    (data) => {
        required('data', data, Number.NaN);
        return $ToNumber(data);
    },
    {
        summary: '将数据转换为数字',
        params: { data: '要转换的数据' },
        paramsType: { data: 'any' },
        returnsType: 'number',
        examples: ['to_number("1.5") // 1.5'],
    },
);

export const to_boolean = VmLib(
    (data) => {
        required('data', data, false);
        return $ToBoolean(data);
    },
    {
        summary: '将数据转换为布尔值',
        params: { data: '要转换的数据' },
        paramsType: { data: 'any' },
        returnsType: 'boolean',
        examples: ['to_boolean(nil) // false'],
    },
);

export const format = VmLib(
    (data, format) => {
        required('data', data, '');
        return $Format(data, format);
    },
    {
        summary: '将数据格式化为指定格式的字符串',
        params: { data: '要格式化的数据', format: '格式字符串' },
        paramsType: { data: 'any', format: 'string' },
        returnsType: 'string',
        examples: ['format(12, ".3") // "12.000"'],
    },
);
