import { expectNumber, VmLib } from '../../helpers.js';
const { atan2: _atan2, pow: _pow, random: _random } = Math;

export const atan2 = VmLib((x, y) => _atan2(expectNumber(0, x), expectNumber(1, y)), {
    summary: '返回从原点到点 (x, y) 的角度（弧度）',
    params: { x: 'x 坐标', y: 'y 坐标' },
    paramsType: { x: 'number', y: 'number' },
    returnsType: 'number',
});
export const pow = VmLib((x, y) => _pow(expectNumber(0, x), expectNumber(1, y)), {
    summary: '返回 x 的 y 次幂',
    params: { x: '底数', y: '指数' },
    paramsType: { x: 'number', y: 'number' },
    returnsType: 'number',
});
export const random = VmLib(() => _random(), {
    summary: '返回 [0, 1) 之间的伪随机数',
    params: {},
    paramsType: {},
    returnsType: 'number',
});

export * from './arr.js';
export * from './const.js';
export * from './unary.js';
export * from './round.js';
export { gamma, factorial } from './gamma.js';
