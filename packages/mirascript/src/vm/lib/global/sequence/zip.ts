import { display } from '../../../../helpers/serialize.js';
import { setRecord } from '../../../../helpers/utils.js';
import { Cp } from '../../../checkpoint.js';
import { isVmArray, type VmConst, type VmArray, type VmRecord } from '../../../types/index.js';
import { VmLib, throwError } from '../../helpers.js';
import { entries } from './entries.js';

export const zip = VmLib(
    (data) => {
        const ets = entries(data) as Array<{ 0: string | number; 1: VmArray }>;
        let length = 0;
        for (const { 0: key, 1: arr } of ets) {
            if (!isVmArray(arr)) {
                throwError(`data[${display(key)}] is not an array: ${display(arr)}`, null);
            }
            length = Math.max(length, arr.length);
        }
        if (length === 0) return [];
        const result: Array<VmArray | VmRecord> = [];
        if (isVmArray(data)) {
            for (let i = 0; i < length; i++) {
                Cp();
                const obj: VmConst[] = [];
                for (const { 0: key, 1: arr } of ets) {
                    obj[key as number] = arr[i] ?? null;
                }
                result.push(obj);
            }
        } else {
            for (let i = 0; i < length; i++) {
                Cp();
                const obj: Record<string, VmConst> = {};
                for (const { 0: key, 1: arr } of ets) {
                    setRecord(obj, key as string, arr[i]);
                }
                result.push(obj);
            }
        }
        return result;
    },
    {
        summary: '将数组的数组/记录转换为数组/记录的数组',
        params: { data: { type: 'array | record', description: '要转换的数组/记录' } },
        returns: { type: '(array | record)[]' },
        examples: [
            'zip((x: [1, 2], y: ["a", "b"])) // [(x: 1, y: "a"), (x: 2, y: "b")]',
            `zip([[1, 2], ["a", "b"]]) // [[1, "a"], [2, "b"]]`,
        ],
    },
);
