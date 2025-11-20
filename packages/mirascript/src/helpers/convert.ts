import type { VmAny, VmRecord } from '../vm/index.js';
import { display, displayFunction } from './serialize.js';
import { VmError } from './error.js';
import { isVmArray, isVmWrapper } from './types.js';
import { keys, isNaN, isFinite } from './utils.js';
const { POSITIVE_INFINITY, NEGATIVE_INFINITY } = Number;

/** 转换为 number */
export function toNumber<F = undefined>(value: VmAny, fallback?: F): number | Exclude<F, undefined> {
    if (typeof value === 'number') {
        return value;
    }
    if (typeof value == 'boolean') {
        return value ? 1 : 0;
    }
    if (typeof value == 'string') {
        value = value.trim();
        if (value !== '') {
            if (value === 'inf' || value === '+inf' || value === 'Infinity' || value === '+Infinity') {
                return POSITIVE_INFINITY;
            }
            if (value === '-inf' || value === '-Infinity') {
                return NEGATIVE_INFINITY;
            }
            if (value === 'nan' || value === 'NaN') {
                return Number.NaN;
            }
            const num = Number(value);
            if (!isNaN(num)) return num;
        }
    }
    if (fallback === undefined) {
        throw new VmError(`Failed to convert value to number: ${display(value)}`, Number.NaN);
    }
    return fallback as Exclude<F, undefined>;
}

/** 转换为 boolean */
export function toBoolean<F = undefined>(value: VmAny, fallback?: F): boolean | Exclude<F, undefined> {
    if (typeof value === 'boolean') return value;
    if (fallback === undefined) {
        throw new VmError(`Failed to convert value to boolean: ${display(value)}`, false);
    }
    return fallback as Exclude<F, undefined>;
}

/** 转换为 string */
function numberToString(value: number): string {
    if (isNaN(value)) return 'nan';
    if (value === Infinity) return 'inf';
    if (value === -Infinity) return '-inf';
    return String(value);
}
/** 转换为 string */
export function innerToString(value: VmAny, useBraces: boolean): string {
    if (value == null) return 'nil';
    if (typeof value == 'number') return numberToString(value);
    if (typeof value == 'string' || typeof value == 'boolean') return String(value);
    if (typeof value == 'function') return displayFunction(value);
    if (isVmWrapper(value)) return value.toString(useBraces);
    if (isVmArray(value)) {
        const strings: string[] = [];
        for (const item of value) {
            strings.push(innerToString(item, true));
        }
        // 在 join 过程中会自动把 null/undefined 和 empty slot 转为 ''
        // 与 innerToString 行为不一致
        const results = strings.join(', ');
        if (!useBraces) return results;
        return `[${results}]`;
    } else {
        const entries = keys(value satisfies VmRecord)
            .map((key) => `${key}: ${innerToString(value[key], true)}`)
            .join(', ');
        if (!useBraces) return entries;
        return `(${entries})`;
    }
}

/** 转换为 string */
export function toString<F = undefined>(value: VmAny, fallback?: F): string | Exclude<F, undefined> {
    if (typeof value === 'string') return value;
    if (value == null) return '';
    try {
        return innerToString(value, false);
    } catch (ex) {
        if (fallback === undefined) {
            const e = new VmError(`Failed to convert value to string: ${display(value)}`, '');
            e.cause = ex;
            throw e;
        }
        return fallback as Exclude<F, undefined>;
    }
}

/** 渲染数字 */
function formatNumber(value: number): string {
    if (!isFinite(value)) return toString(value, undefined as never);
    if (value === 0) return '0';
    const s = value.toString();
    let ps;
    const abs = Math.abs(value);
    if (abs >= 1000 || abs < 0.001) {
        const ps1 = value.toExponential();
        const ps2 = value.toExponential(5);
        ps = ps1.length < ps2.length ? ps1 : ps2;
    } else {
        ps = value.toPrecision(6);
    }
    return ps.length < s.length ? ps : s;
}

/** 格式化为 string */
export function toFormat(value: VmAny, format: string | null | undefined): string {
    const f = format == null ? '' : format.trim();

    if (typeof value == 'number') {
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
