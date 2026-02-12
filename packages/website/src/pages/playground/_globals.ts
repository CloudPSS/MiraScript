import type { VmContext } from '@mirascript/mirascript';
import * as mirascript from '@mirascript/mirascript';
import * as mirascriptSubtle from '@mirascript/mirascript/subtle';

Object.defineProperty(globalThis, 'mirascript', {
    value: { ...mirascript, subtle: mirascriptSubtle },
});

/** 创建全局环境 */
export function globals(): VmContext {
    const { createVmContext } = mirascript;
    return createVmContext(
        {
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
            globalThis,
        },
        (key) => {
            if (['title', 'name', 'age', 'active', 'register_date', 'scores'].includes(key)) {
                return `用于 “Template” 示例的全局变量`;
            }
            if (key === 'globalThis') {
                return '作为 extern 导入的 JavaScript 全局对象';
            }
            return undefined;
        },
    );
}
