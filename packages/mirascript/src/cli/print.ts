import styles from 'ansi-styles';
import supportsColor from 'supports-color';
import {
    serialize,
    serializeNumber,
    serializeBoolean,
    serializeNil,
    type SerializeOptions,
    operations,
} from '../subtle.js';
import type { VmAny, VmRecord } from '../vm/index.js';

const noColor = !supportsColor.stdout;

const options: Partial<SerializeOptions> = {
    maxDepth: 3,
    serializeNil: noColor ? undefined : () => styles.gray.open + serializeNil() + styles.gray.close,
    serializeBoolean: noColor ? undefined : (v) => styles.blue.open + serializeBoolean(v) + styles.blue.close,
    serializeNumber: noColor ? undefined : (v) => styles.yellow.open + serializeNumber(v) + styles.yellow.close,
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
    serializeStringEscape: noColor ? undefined : (v) => styles.bold.open + v + styles.bold.close,
    serializePropName: noColor ? undefined : (v) => styles.whiteBright.open + String(v) + styles.whiteBright.close,
    serializeFunction: noColor
        ? operations.$ToString
        : (v) => styles.cyan.open + operations.$ToString(v) + styles.cyan.close,
    serializeModule: noColor
        ? (v, depth, options) => {
              return operations.$ToString(v) + ' ' + options.serializeRecord(v.value as VmRecord, depth, options);
          }
        : (v, depth, options) => {
              return (
                  styles.magenta.open +
                  operations.$ToString(v) +
                  styles.magenta.close +
                  ' ' +
                  options.serializeRecord(v.value as VmRecord, depth, options)
              );
          },
};

/** 序列化值 */
export function print(value: VmAny, depth = 3): string {
    if (value === undefined) {
        if (noColor) return '<uninitialized>';
        return styles.gray.open + '<uninitialized>' + styles.gray.close;
    }
    return serialize(value, { ...options, maxDepth: depth });
}
