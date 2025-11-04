import { lib, VmSharedContext } from '@mirascript/mirascript/subtle';
import {
    createVmContext,
    getVmFunctionInfo,
    VmExtern,
    VmFunction,
    VmModule,
    type VmAny,
    type VmContext,
} from '@mirascript/mirascript';
import type { ConsoleManager } from './console-manager.js';
import { escapeHtml, print } from './utils.js';

/** 创建全局环境 */
export function globals(consoleManager: ConsoleManager): VmContext {
    const arr = [1, 2, [1, 2], { x: 0 }];
    arr[100] = 100; // make a sparse array

    /** 创建简单的 debug_print 函数 */
    function debugPrint(...args: VmAny[]) {
        lib.debug_print(...args);
        const messages = args.map(async (arg) => {
            if (typeof arg === 'string') return escapeHtml(arg);
            return print(arg);
        });
        consoleManager.log(Promise.all(messages).then((message) => message.join(' ')));
    }
    return createVmContext(
        {
            extern_arr: new VmExtern(arr),
            obj: { a: [], b: 1, c: '2', d: { e: 3 } },
            arr: [1, 2, 3],
            long_str: 'Long string content'.repeat(10000),
            mod: new VmModule('test', {
                s: VmSharedContext.sin,
                inner: new VmModule('inner', {
                    s: VmSharedContext.sin,
                }),
            }),
            debug_print: VmFunction(debugPrint, getVmFunctionInfo(VmSharedContext.debug_print)),

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
                s: VmSharedContext.sin,
                m: VmSharedContext.matrix,
                undefined: undefined,
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
