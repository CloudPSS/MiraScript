import { isArray, isSafeInteger } from '../../../../helpers/utils.js';
import { Element } from '../../../helpers.js';
import { $ToNumber, $ToString } from '../../../operations.js';
import {
    isVmArray,
    isVmRecord,
    VM_ARRAY_MAX_LENGTH,
    type VmArray,
    type VmConst,
    type VmValue,
} from '../../../types/index.js';
import { VmLib, expectArrayOrRecord, expectConst, throwError } from '../../_helpers.js';

const arrIndex = (index: NonNullable<VmConst>): number => {
    const idx = Math.trunc($ToNumber(index));
    if (!isSafeInteger(idx) || idx < 0 || idx >= VM_ARRAY_MAX_LENGTH) {
        return -1;
    }
    return idx;
};

const withInner = (obj: VmConst | undefined, key: VmArray, keyIndex: number, value: VmConst): VmConst => {
    if (keyIndex >= key.length) {
        return value;
    }
    const k = key[keyIndex]!;
    let result: Array<VmConst | undefined> | Record<string, VmConst | undefined>;
    if (isVmArray(obj)) {
        result = [...obj];
    } else if (isVmRecord(obj)) {
        result = { ...obj };
    } else if (arrIndex(k) === k) {
        result = [];
    } else {
        result = {};
    }
    if (isArray(result)) {
        const index = arrIndex(k);
        while (index > result.length) {
            result.push(null);
        }
        result[index] = withInner(result[index], key, keyIndex + 1, value);
    } else {
        const prop = $ToString(k);
        result[prop] = withInner(result[prop], key, keyIndex + 1, value);
    }
    return result;
};

const normalizeEntries = (data: VmConst, entries: Array<VmValue | undefined>): Map<NonNullable<VmConst>, VmConst> => {
    if (entries.length % 2 !== 0) {
        throwError('Expected even number of entries', data);
    }
    const entryData = new Map<NonNullable<VmConst>, VmConst>();
    for (let i = 0; i < entries.length; i += 2) {
        let key = entries[i]!;
        expectConst('key', key, data);
        if (key == null) {
            continue;
        }
        if (isVmArray(key)) {
            if (key.length === 0 || key.includes(null) || key.includes(undefined)) {
                continue;
            } else if (key.length === 1) {
                key = key[0]!;
            }
        }
        const value = entries[i + 1]!;
        entryData.set(key, Element(value));
    }
    return entryData;
};

const _with = VmLib(
    (data, ...entries) => {
        expectArrayOrRecord('data', data, data);
        if (entries.length === 0) {
            return data;
        }

        const entryData = normalizeEntries(data, entries);
        if (isVmArray(data)) {
            const result: Array<VmConst | undefined> = [...data];
            for (const [key, value] of entryData) {
                let index: number;
                let val: VmConst;
                if (isVmArray(key)) {
                    index = arrIndex(key[0]!);
                    if (index < 0) continue;
                    val = withInner(result[index], key, 1, value);
                } else {
                    index = arrIndex(key);
                    if (index < 0) continue;
                    val = value;
                }
                while (index > result.length) {
                    result.push(null);
                }
                result[index] = val;
            }
            return result;
        } else {
            const result: Record<string, VmConst | undefined> = { ...data };
            for (const [key, value] of entryData) {
                let prop: string;
                let val: VmConst;
                if (isVmArray(key)) {
                    const firstKey = key[0]!;
                    prop = $ToString(firstKey);
                    val = withInner(result[prop], key, 1, value);
                } else {
                    prop = $ToString(key);
                    val = value;
                }
                result[prop] = val;
            }
            return result;
        }
    },
    {
        summary: '在数组或记录中设置多个键值对',
        params: {
            data: '要设置的数组或记录',
            '..entries': '要设置的键值对，成对出现',
        },
        paramsType: {
            data: 'array | record',
            '..entries': `[..[number | string | (number | string)[], any][]]`,
        },
        returnsType: 'type(data)',
        examples: [
            `with([10, 20], 2, 99, 3 ,100) // [10, 20, 99, 100]`,
            `(a: 1)::with(["b", 1], 2) // (a: 1, b: [nil, 2])`,
        ],
    },
);
export { _with as 'with' };
