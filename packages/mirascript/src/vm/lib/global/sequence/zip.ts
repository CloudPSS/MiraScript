import { serialize } from '../../../../subtle.js';
import { Cp } from '../../../helpers.js';
import { isVmArray, type VmConst, type VmArray } from '../../../types/index.js';
import { VmLib, throwError } from '../../_helpers.js';
import { entries } from './entries.js';

export const zip = VmLib(
    (data) => {
        const ets = entries(data);
        let len = 0;
        for (const { 0: key, 1: arr } of ets) {
            if (!isVmArray(arr)) {
                throwError(`data[${serialize(key)}] is not an array`, null);
            }
            len = Math.max(len, arr.length);
        }
        if (len === 0) return [];
        const result: Array<Record<string | number, VmConst>> = [];
        const isArr = isVmArray(data);
        for (let i = 0; i < len; i++) {
            Cp();
            const obj: Record<number | string, VmConst> = isArr ? ([] as Record<number, VmConst>) : {};
            for (const { 0: key, 1: arr } of ets) {
                obj[key] = (arr as VmArray)[i] ?? null;
            }
            result.push(obj);
        }
        return result;
    },
    {
        summary: '将数组的数组/记录转换为数组/记录的数组',
        params: { data: '要转换的数组/记录' },
        paramsType: { data: 'array | record' },
        returnsType: '(array | record)[]',
        examples: [
            'zip((x: [1, 2], y: ["a", "b"])) // [(x: 1, y: "a"), (x: 2, y: "b")]',
            `zip([[1, 2], ["a", "b"]]) // [[1, "a"], [2, "b"]]`,
        ],
    },
);
