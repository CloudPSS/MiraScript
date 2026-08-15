import styles from 'ansi-styles';
import type { VmAny, VmRecord } from '@mirascript/mirascript';
import {
    serialize,
    serializeNumber,
    serializeBoolean,
    serializeNil,
    type SerializeOptions,
    operations,
} from '@mirascript/mirascript/subtle';
import { noColor, format } from './color.js';

const options: Partial<SerializeOptions> = {
    maxDepth: 3,
    serializeNil: noColor ? undefined : () => format(serializeNil(), styles.gray),
    serializeBoolean: noColor ? undefined : (v) => format(serializeBoolean(v), styles.blue),
    serializeNumber: noColor ? undefined : (v) => format(serializeNumber(v), styles.yellow),
    serializeStringQuote: noColor
        ? undefined
        : (v, open) => {
              const q = styles.dim.open + v + styles.dim.close;
              if (open) {
                  return styles.green.open + q;
              } else {
                  return q + styles.green.close;
              }
          },
    serializeStringEscape: noColor ? undefined : (v) => format(v, styles.bold),
    serializePropName: noColor ? undefined : (v) => format(String(v), styles.whiteBright),
    serializeFunction: (v) => format(operations.$ToString(v), styles.cyan),
    serializeModule: (v, depth, options) => {
        return (
            format(operations.$ToString(v), styles.magenta) +
            ' ' +
            options.serializeRecord(v.value as VmRecord, depth, options)
        );
    },
};

/** 序列化值 */
export function print(value: VmAny, depth = 3): string {
    if (value === undefined) {
        return format('<undefined>', styles.gray);
    }
    return serialize(value, { ...options, maxDepth: depth });
}
