import styles from 'ansi-styles';
import { serialize, type SerializeOptions } from '../subtle.js';
import type { VmAny, VmRecord } from '../vm/index.js';

const options: Partial<SerializeOptions> = {
    maxDepth: 3,
    serializeNil: () => styles.gray.open + 'nil' + styles.gray.close,
    serializeBoolean: (v) => styles.blue.open + (v ? 'true' : 'false') + styles.blue.close,
    serializeNumber: (v) =>
        styles.yellow.open +
        (Number.isNaN(v) ? 'nan' : Number.isFinite(v) ? v.toString() : v > 0 ? 'inf' : '-inf') +
        styles.yellow.close,
    serializeStringQuote: (v, open) => {
        const q = styles.dim.open + v + styles.dim.close;
        if (open) {
            return styles.green.open + q;
        } else {
            return q + styles.green.close;
        }
    },
    serializeStringEscape: (v) => styles.bold.open + v + styles.bold.close,
    serializeFunction: (v) => styles.cyan.open + `<function ${v.name || 'anonymous'}>` + styles.cyan.close,
    serializePropName: (v) => styles.whiteBright.open + String(v) + styles.whiteBright.close,
    serializeModule: (v, depth, options) => {
        return (
            styles.magenta.open +
            `<module ${v.name || 'anonymous'}>` +
            styles.magenta.close +
            ' ' +
            options.serializeRecord(v.value as VmRecord, depth, options)
        );
    },
};

/** 序列化值 */
export function print(value: VmAny, depth = 3): string {
    if (value === undefined) {
        return styles.gray.open + '<uninitialized>' + styles.gray.close;
    }
    return serialize(value, { ...options, maxDepth: depth });
}
