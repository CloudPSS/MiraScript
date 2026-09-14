import { toBoolean } from '../../../../helpers/convert/index.js';
import { entries } from '../../../../helpers/utils.js';
import { Cp } from '../../../checkpoint.js';
import { $Call } from '../../../operations/index.js';
import { isVmArray, type VmAny } from '../../../types/index.js';
import { expectArrayOrRecord, expectCallable, VmLib } from '../../helpers.js';

/** 生成函数 */
function build(every: boolean, summary: string, examples: string[]): VmLib<(data: VmAny, predicate: VmAny) => boolean> {
    return VmLib(
        (data, predicate) => {
            expectArrayOrRecord('data', data, null);
            expectCallable('predicate', predicate, data);
            if (isVmArray(data)) {
                for (let i = 0; i < data.length; i++) {
                    Cp();
                    /* c8 ignore next */
                    const value = data[i] ?? null;
                    const ret = toBoolean($Call(predicate, [value, i, data]), undefined);
                    if (ret === !every) return !every;
                }
                return every;
            } else {
                for (const [key, v] of entries(data)) {
                    Cp();
                    /* c8 ignore next */
                    const value = v ?? null;
                    const ret = toBoolean($Call(predicate, [value, key, data]), undefined);
                    if (ret === !every) return !every;
                }
                return every;
            }
        },
        {
            summary,
            params: {
                data: { type: 'array | record', description: '要检查的数组或记录' },
                predicate: {
                    type: 'fn(value: any, key: number | string, input: type(data)) -> boolean',
                    description: '用于测试每个键值对的函数',
                },
            },
            returns: { type: 'boolean' },
            examples,
        },
    );
}

export const all = build(true, '检查数组或记录中的所有键值对是否都满足条件', ['all([1, 2, 3], fn { it > 0 }) // true']);

export const any = build(false, '检查数组或记录中的是否存在满足条件的键值对', [
    'any([0, 1, 2], fn { it > 1 }) // true',
]);
