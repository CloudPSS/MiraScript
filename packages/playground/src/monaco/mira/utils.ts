import { DiagnosticCode } from '@mirascript/wasm';
import { type editor, Range, type IPosition, type IRange } from '@private/monaco-editor';
import { VmSharedGlobal } from 'mirascript/subtle';
import { getVmFunctionInfo, isVmModule, type VmImmutable, serialize, type VmFunctionInfo } from 'mirascript';
import type { LocalDefinition } from './compile-result';

/** 生成函数签名 */
export function signature(id: string | undefined, info: VmFunctionInfo): string {
    const prefix = id ? `fn ${id}` : 'fn';
    let params;
    if (!info.params) {
        params = '(..)';
    } else {
        const paramItems = Object.keys(info.params).map((key) => {
            const type = info.paramsType?.[key];
            const typeStr = type ? `: ${type}` : '';
            return `${key}${typeStr}`;
        });
        const len = paramItems.reduce((acc, item) => acc + item.length, 0);
        if (len <= 60) {
            params = `(${paramItems.join(', ')})`;
        } else {
            params = `(\n${paramItems.map((item) => `  ${item},`).join('\n')}\n)`;
        }
    }
    const returns = info.returnsType ? ` -> ${info.returnsType}` : '';
    return `${prefix}${params}${returns};`;
}

/** 生成函数参数列表 */
export function paramsList(model: editor.ITextModel, info: VmFunctionInfo | LocalDefinition['fn']): string {
    if (!info) return '(..)';
    if ('args' in info) {
        const { args } = info;
        if (args[0]?.definition.code === DiagnosticCode.ParameterIt) {
            return args[0].references.length ? '(it)' : '()';
        } else {
            return `(${args.map((a) => model.getValueInRange(a.definition.range)).join(', ')})`;
        }
    } else {
        if (!info.params) return '(..)';
        const paramItems = Object.keys(info.params).join(', ');
        return `(${paramItems})`;
    }
}

/** 生成函数文档 */
export function globalFnDocument(info: VmFunctionInfo): string {
    const doc = [];
    if (info.summary) {
        doc.push(info.summary);
    }
    if (info.params) {
        for (const [key, value] of Object.entries(info.params)) {
            doc.push(`- \`${key}\`: ${value}`);
        }
    }
    if (info.returns) {
        doc.push(`- **返回值**: ${info.returns}`);
    }
    return doc.join('\n');
}

const CODEBLOCK_FENCE = '`'.repeat(16);
/** 获取代码块格式化字符串 */
export function codeblock(value: string): string {
    return `\n${CODEBLOCK_FENCE}mirascript\n${value}\n${CODEBLOCK_FENCE}\n`;
}

/** 检查位置是否在范围内，且范围非空 */
export function strictInRange(range: IRange, position: IPosition): boolean {
    return !Range.isEmpty(range) && Range.containsPosition(range, position);
}

/** 获取全局变量脚本 */
export function getGlobal(name: string): { value: VmImmutable | undefined; script: string; doc: string } {
    const value = VmSharedGlobal[name];
    const info = getVmFunctionInfo(value);
    if (info) {
        return {
            value,
            script: signature(name, info),
            doc: globalFnDocument(info),
        };
    }
    if (isVmModule(value)) {
        return {
            value,
            script: `module ${name};`,
            doc: `模块 \`${name}\``,
        };
    }
    const valueStr = value !== undefined ? serialize(value) : '/* … */';
    if (name.startsWith('@')) return { value, script: `const ${name} = ${valueStr};`, doc: '' };
    return { value, script: `let ${name} = ${valueStr};`, doc: '' };
}
