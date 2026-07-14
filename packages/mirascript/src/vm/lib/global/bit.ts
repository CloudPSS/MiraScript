import { VmLib, expectNumber } from '../helpers.js';

export const b_and = VmLib(
    (x, y) => {
        return expectNumber(0, x) & expectNumber(1, y);
    },
    {
        summary: '返回两个数的按位与',
        params: {
            x: { type: 'number', description: '第一个操作数' },
            y: { type: 'number', description: '第二个操作数' },
        },
        returns: { type: 'number' },
        examples: ['b_and(6, 3) // 2'],
    },
);

export const b_or = VmLib(
    (x, y) => {
        return expectNumber(0, x) | expectNumber(1, y);
    },
    {
        summary: '返回两个数的按位或',
        params: {
            x: { type: 'number', description: '第一个操作数' },
            y: { type: 'number', description: '第二个操作数' },
        },
        returns: { type: 'number' },
        examples: ['b_or(5, 2) // 7'],
    },
);

export const b_not = VmLib(
    (x) => {
        return ~expectNumber('x', x);
    },
    {
        summary: '返回一个数的按位取反',
        params: { x: { type: 'number', description: '操作数' } },
        returns: { type: 'number' },
        examples: ['b_not(0) // -1'],
    },
);

export const b_xor = VmLib(
    (x, y) => {
        return expectNumber(0, x) ^ expectNumber(1, y);
    },
    {
        summary: '返回两个数的按位异或',
        params: {
            x: { type: 'number', description: '第一个操作数' },
            y: { type: 'number', description: '第二个操作数' },
        },
        returns: { type: 'number' },
        examples: ['b_xor(5, 3) // 6'],
    },
);

export const shl = VmLib(
    (x, y) => {
        return expectNumber(0, x) << expectNumber(1, y);
    },
    {
        summary: '返回第一个操作数左移指定的位数',
        params: { x: { type: 'number', description: '第一个操作数' }, y: { type: 'number', description: '位数' } },
        returns: { type: 'number' },
        examples: ['shl(3, 2) // 12'],
    },
);

export const sar = VmLib(
    (x, y) => {
        return expectNumber(0, x) >> expectNumber(1, y);
    },
    {
        summary: '返回第一个操作数右移指定的位数',
        params: { x: { type: 'number', description: '第一个操作数' }, y: { type: 'number', description: '位数' } },
        returns: { type: 'number' },
        examples: ['sar(-8, 1) // -4'],
    },
);

export const shr = VmLib(
    (x, y) => {
        return expectNumber(0, x) >>> expectNumber(1, y);
    },
    {
        summary: '返回第一个操作数无符号右移指定的位数',
        params: { x: { type: 'number', description: '第一个操作数' }, y: { type: 'number', description: '位数' } },
        returns: { type: 'number' },
        examples: ['shr(8, 1) // 4'],
    },
);
