import { $ToNumber } from '../../operations.ts';
import { VmLib } from '../_helpers.ts';
const { atan2: _atan2, pow: _pow } = Math;

export const atan2 = VmLib((x, y) => _atan2($ToNumber(x), $ToNumber(y)), {
    summary: '返回从原点到点 (x, y) 的角度（弧度）',
    params: { x: 'x 坐标', y: 'y 坐标' },
    paramsType: { x: 'number', y: 'number' },
    returnsType: 'number',
});
export const pow = VmLib((x, y) => _pow($ToNumber(x), $ToNumber(y)), {
    summary: '返回 x 的 y 次幂',
    params: { x: '底数', y: '指数' },
    paramsType: { x: 'number', y: 'number' },
    returnsType: 'number',
});
export const random = VmLib(Math.random, {
    summary: '返回 0 到 1 之间的伪随机数',
    params: {},
    paramsType: {},
    returnsType: 'number',
});

export * from './math-arr.ts';
export * from './math-const.ts';
export * from './math-unary.ts';
export * from './math-additional.ts';
