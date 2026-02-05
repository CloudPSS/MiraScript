import { DiagnosticCode } from '@mirascript/constants';
import { VmLib } from '../../helpers.js';

export const PI = VmLib(Math.PI, {
    summary: '圆周率',
    returnsType: 'number',
});
export const E = VmLib(Math.E, {
    summary: '自然对数的底数',
    returnsType: 'number',
});
export const pi = VmLib(Math.PI, {
    summary: '圆周率',
    returnsType: 'number',
    deprecated: {
        use: 'PI',
        message: DiagnosticCode.PreferUppercaseConstant,
    },
});
export const e = VmLib(Math.E, {
    summary: '自然对数的底数',
    returnsType: 'number',
    deprecated: {
        use: 'E',
        message: DiagnosticCode.PreferUppercaseConstant,
    },
});
export const SQRT1_2 = VmLib(Math.SQRT1_2, {
    summary: '½ 的平方根',
    returnsType: 'number',
});
export const SQRT2 = VmLib(Math.SQRT2, {
    summary: '2 的平方根',
    returnsType: 'number',
});
export const LN2 = VmLib(Math.LN2, {
    summary: '2 的自然对数',
    returnsType: 'number',
});
export const LN10 = VmLib(Math.LN10, {
    summary: '10 的自然对数',
    returnsType: 'number',
});
export const LOG2E = VmLib(Math.LOG2E, {
    summary: 'e 以 2 为底的对数',
    returnsType: 'number',
});
export const LOG10E = VmLib(Math.LOG10E, {
    summary: 'e 以 10 为底的对数',
    returnsType: 'number',
});
