import { $ToNumber } from '../../operations.js';
import { VmLib } from '../_helpers.js';

export const b_and = VmLib(
    (x, y) => {
        return $ToNumber(x) & $ToNumber(y);
    },
    {
        summary: '返回两个数的按位与',
        params: { x: '第一个操作数', y: '第二个操作数' },
        paramsType: { x: 'number', y: 'number' },
        returnsType: 'number',
        examples: ['b_and(6, 3) // 2'],
    },
);

export const b_or = VmLib(
    (x, y) => {
        return $ToNumber(x) | $ToNumber(y);
    },
    {
        summary: '返回两个数的按位或',
        params: { x: '第一个操作数', y: '第二个操作数' },
        paramsType: { x: 'number', y: 'number' },
        returnsType: 'number',
        examples: ['b_or(5, 2) // 7'],
    },
);

export const b_not = VmLib(
    (x) => {
        return ~$ToNumber(x);
    },
    {
        summary: '返回一个数的按位取反',
        params: { x: '操作数' },
        paramsType: { x: 'number' },
        returnsType: 'number',
        examples: ['b_not(0) // -1'],
    },
);

export const b_xor = VmLib(
    (x, y) => {
        return $ToNumber(x) ^ $ToNumber(y);
    },
    {
        summary: '返回两个数的按位异或',
        params: { x: '第一个操作数', y: '第二个操作数' },
        paramsType: { x: 'number', y: 'number' },
        returnsType: 'number',
        examples: ['b_xor(5, 3) // 6'],
    },
);

export const shl = VmLib(
    (x, y) => {
        return $ToNumber(x) << $ToNumber(y);
    },
    {
        summary: '返回第一个操作数左移指定的位数',
        params: { x: '第一个操作数', y: '位数' },
        paramsType: { x: 'number', y: 'number' },
        returnsType: 'number',
        examples: ['shl(3, 2) // 12'],
    },
);

export const sar = VmLib(
    (x, y) => {
        return $ToNumber(x) >> $ToNumber(y);
    },
    {
        summary: '返回第一个操作数右移指定的位数',
        params: { x: '第一个操作数', y: '位数' },
        paramsType: { x: 'number', y: 'number' },
        returnsType: 'number',
        examples: ['sar(-8, 1) // -4'],
    },
);

export const shr = VmLib(
    (x, y) => {
        return $ToNumber(x) >>> $ToNumber(y);
    },
    {
        summary: '返回第一个操作数无符号右移指定的位数',
        params: { x: '第一个操作数', y: '位数' },
        paramsType: { x: 'number', y: 'number' },
        returnsType: 'number',
        examples: ['shr(8, 1) // 4'],
    },
);
