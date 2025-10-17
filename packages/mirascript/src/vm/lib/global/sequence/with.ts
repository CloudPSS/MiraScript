import { Element } from '../../../helpers.ts';
import { $ToNumber, $ToString } from '../../../operations.ts';
import { isVmArray, type VmConst } from '../../../types/index.ts';
import { VmLib, expectArrayOrRecord, throwError } from '../../_helpers.ts';
import { isSafeInteger } from '../../../../helpers/utils.ts';

const _with = VmLib(
    (data, ...entries) => {
        expectArrayOrRecord('data', data, data);
        if (entries.length % 2 !== 0) {
            throwError('Expected even number of entries', data);
        }
        if (isVmArray(data)) {
            const result: Array<VmConst | undefined> = [...data];
            for (let i = 0; i < entries.length; i += 2) {
                const index = Math.trunc($ToNumber(entries[i]));
                if (!isSafeInteger(index) || index < 0) continue;
                const value = entries[i + 1];
                while (index > result.length) {
                    result.push(null);
                }
                result[index] = Element(value);
            }
            return result;
        } else {
            const result: Record<string, VmConst | undefined> = { ...data };
            for (let i = 0; i < entries.length; i += 2) {
                const key = $ToString(entries[i]);
                const value = entries[i + 1];
                result[key] = Element(value);
            }
            return result;
        }
    },
    {
        summary: '在数组或记录中设置多个键值对',
        params: { data: '要设置的数组或记录', '..entries': '要设置的键值对，成对出现' },
        paramsType: { data: 'array | record', '..entries': '[..[string | number, any][]]' },
        returnsType: 'type(data)',
        examples: ['with([10, 20], 2, 99) // [10, 20, 99]', 'with((a: 1), "b", 2) // (a: 1, b: 2)'],
    },
);
export { _with as 'with' };
