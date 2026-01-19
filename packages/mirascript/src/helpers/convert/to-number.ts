import type { VmAny } from '../../vm/index.js';
import { VmError } from '../error.js';
import { display } from '../serialize.js';
import { isNaN, NegativeInfinity, NotNumber, PositiveInfinity } from '../utils.js';

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
                return PositiveInfinity;
            }
            if (value === '-inf' || value === '-Infinity') {
                return NegativeInfinity;
            }
            if (value === 'nan' || value === 'NaN') {
                return NotNumber;
            }
            const num = Number(value);
            if (!isNaN(num)) return num;
        }
    }
    if (fallback === undefined) {
        throw new VmError(`Failed to convert value to number: ${display(value)}`, NotNumber);
    }
    return fallback as Exclude<F, undefined>;
}
