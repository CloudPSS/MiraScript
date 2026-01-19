import type { VmAny } from '../../vm/index.js';
import { isFinite } from '../utils.js';
import { toString } from './to-string.js';

/** 渲染数字 */
function formatNumber(finite: number): string {
    if (finite === 0) return '0';
    const s = finite.toString();
    let ps;
    const abs = Math.abs(finite);
    if (abs >= 1000 || abs < 0.001) {
        const ps1 = finite.toExponential();
        const ps2 = finite.toExponential(5);
        ps = ps1.length < ps2.length ? ps1 : ps2;
    } else {
        ps = finite.toPrecision(6);
    }
    return ps.length < s.length ? ps : s;
}

/** 格式化为 string */
export function toFormat(value: VmAny, format: string | null | undefined): string {
    const f = format == null ? '' : format.trim();

    if (typeof value == 'number') {
        if (!isFinite(value)) {
            return toString(value, undefined as never);
        }
        if (/^\.\d+$/.test(f)) {
            let digits = Math.trunc(Number(f.slice(1)));
            if (!(digits <= 100)) digits = 100;
            return value.toFixed(digits);
        } else {
            return formatNumber(value);
        }
    }

    return toString(value, undefined as never);
}
