import { $Call } from '../../../operations/index.js';
import type { VmValue } from '../../../types/index.js';
import { VmLib, expectCallable, expectConst, iterate, required } from '../../helpers.js';

export const fold = VmLib(
    (data, initial, f) => {
        required('initial', initial, null);
        expectConst('data', data, null);
        expectCallable('f', f, data);

        let acc: VmValue = initial;
        iterate(data, (value, index, data) => {
            acc = $Call(f, [acc, value, index, data]);
        });
        return acc;
    },
    {
        summary: '对数组或记录中的每个元素应用函数，并返回累积结果',
        params: {
            data: { type: 'array | record', description: '要折叠的数组或记录' },
            initial: { type: 'any', description: '初始累积值' },
            f: {
                type: 'fn(acc: type(initial), value: any, key: number | string, input: type(data)) -> type(initial)',
                description: '应用于每个元素的函数',
            },
        },
        returns: { type: 'type(initial)' },
        examples: ['fold([1, 2, 3], 0, fn (acc, x) { acc + x }) // 6'],
    },
);
