import type { VmAny, VmContext, VmFunction } from '@mirascript/mirascript';
import type { ConsoleManager } from './console-manager.js';
import { mirascript, mirascriptSubtle } from './loader.js';

/** 创建全局环境 */
export function globals(consoleManager: ConsoleManager): VmContext {
    const arr = [1, 2, [1, 2], { x: 0 }];
    arr[100] = 100; // make a sparse array

    const { DefaultVmContext } = mirascriptSubtle;
    const { VmExtern, VmModule, VmFunction, getVmFunctionInfo, createVmContext } = mirascript;
    return createVmContext(
        {
            'invalid-key!': 'This key is invalid in MiraScript',
            null_value: null,
            undefined_value: undefined,
            extern_arr: new VmExtern(arr),
            extern_buf: new VmExtern(new Uint8Array(100)),
            // eslint-disable-next-line no-sparse-arrays
            sparse_arr: [1, 2, , 4],
            obj: { a: [], b: 1, c: '2', d: { e: 3 } },
            arr: [1, 2, 3],
            long_str: 'Long string content'.repeat(10000),
            mod: new VmModule('test', {
                s: DefaultVmContext.get('sin') as VmFunction,
                inner: new VmModule('inner', {
                    s: DefaultVmContext.get('sin') as VmFunction,
                }),
            }),
            debug_print: VmFunction(
                (...args: VmAny[]) => {
                    consoleManager.log(args);
                },
                getVmFunctionInfo(DefaultVmContext.get('debug_print')),
            ),
            // for template examples
            title: 'MiraScript 示例',
            name: 'MiraScript',
            age: 18,
            active: true,
            register_date: Date.parse('2023-01-01T12:33:10Z'),
            scores: [
                { subject: '语文', points: 90 },
                { subject: '数学', points: 95 },
                { subject: '英语', points: 85 },
            ],
        },
        {
            extern_obj: {
                a: [],
                b: 1,
                c: '2',
                d: { e: 3 },
                s: DefaultVmContext.get('sin'),
                m: DefaultVmContext.get('matrix'),
                undefined: undefined,
                'invalid-key!': 'This key is invalid in MiraScript',
            },
            globalThis,
        },
        (key) => {
            if (['title', 'name', 'age', 'active', 'register_date', 'scores'].includes(key)) {
                return `用于 “Template” 示例的全局变量`;
            }
            return undefined;
        },
    );
}
