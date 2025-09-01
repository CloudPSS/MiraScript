import { $ToNumber } from '../../../operations.js';
import { VmLib } from '../../_helpers.js';

export const and = VmLib(
    (x, y) => {
        return $ToNumber(x) & $ToNumber(y);
    },
    {
        summary: '返回两个数的按位与',
        params: { x: '第一个操作数', y: '第二个操作数' },
        paramsType: { x: 'number', y: 'number' },
        returnsType: 'number',
    },
);

export const or = VmLib(
    (x, y) => {
        return $ToNumber(x) | $ToNumber(y);
    },
    {
        summary: '返回两个数的按位或',
        params: { x: '第一个操作数', y: '第二个操作数' },
        paramsType: { x: 'number', y: 'number' },
        returnsType: 'number',
    },
);

export const not = VmLib(
    (x) => {
        return ~$ToNumber(x);
    },
    {
        summary: '返回一个数的按位取反',
        params: { x: '操作数' },
        paramsType: { x: 'number' },
        returnsType: 'number',
    },
);

export const xor = VmLib(
    (x, y) => {
        return $ToNumber(x) ^ $ToNumber(y);
    },
    {
        summary: '返回两个数的按位异或',
        params: { x: '第一个操作数', y: '第二个操作数' },
        paramsType: { x: 'number', y: 'number' },
        returnsType: 'number',
    },
);
