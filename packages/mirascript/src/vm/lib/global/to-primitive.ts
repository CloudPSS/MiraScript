import { $ToString, $ToNumber, $ToBoolean } from '../../operations.js';
import { VmLib } from '../helpers.js';

export const to_string = VmLib((data) => $ToString(data), {
    summary: '将数据转换为字符串',
    params: { data: '要转换的数据' },
    paramsType: { data: 'any' },
    returnsType: 'string',
});

export const to_number = VmLib((data) => $ToNumber(data), {
    summary: '将数据转换为数字',
    params: { data: '要转换的数据' },
    paramsType: { data: 'any' },
    returnsType: 'number',
});

export const to_boolean = VmLib((data) => $ToBoolean(data), {
    summary: '将数据转换为布尔值',
    params: { data: '要转换的数据' },
    paramsType: { data: 'any' },
    returnsType: 'boolean',
});
