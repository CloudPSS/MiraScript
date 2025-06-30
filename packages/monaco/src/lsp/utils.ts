import { DiagnosticCode } from '@mirascript/wasm';
import { type editor, Range, type IPosition, type IRange } from '../monaco-api.js';
import {
    getVmFunctionInfo,
    isVmArray,
    isVmExtern,
    isVmFunction,
    isVmModule,
    isVmPrimitive,
    isVmRecord,
    serialize,
    type VmAny,
    type VmFunctionInfo,
    type VmValue,
} from 'mirascript';
import { operations, serializePropName, serializeString } from 'mirascript/subtle';
import type { LocalDefinition } from './compile-result.js';

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
    return `${prefix}${params}${returns}`;
}

/** 生成函数参数列表 */
export function paramsList(model: editor.ITextModel, info: VmFunctionInfo | LocalDefinition['fn']): string {
    if (!info) return '(..)';
    if ('scope' in info) {
        const {
            args,
            scope: { params },
        } = info;
        if (params[0]?.code === DiagnosticCode.ParameterIt) {
            return params[0].references.length ? '(it)' : '()';
        }
        return `(${params
            .map((a, i) => {
                const rest =
                    a.code === DiagnosticCode.ParameterRestPattern ||
                    a.code === DiagnosticCode.ParameterMutableRest ||
                    a.code === DiagnosticCode.ParameterImmutableRest;
                const argsInParam = args.filter((arg) => Range.containsRange(a.range, arg.definition.range));
                const argName =
                    argsInParam.length === 0
                        ? `arg_${i}`
                        : argsInParam.map((arg) => model.getValueInRange(arg.definition.range)).join('_');
                if (rest) return `..${argName}`;
                return argName;
            })
            .join(', ')})`;
    } else {
        if (!info.params) return '(..)';
        const paramItems = Object.keys(info.params).join(', ');
        return `(${paramItems})`;
    }
}

/** 生成函数文档 */
export function globalFnDoc(info: VmFunctionInfo): string {
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
export function strictContainsPosition(range: IRange, position: IPosition): boolean {
    return !Range.isEmpty(range) && Range.containsPosition(range, position);
}

const MAX_WIDTH = 40;
const MAX_ENTRIES = 100;

/** 将值序列化为便于展示的字符串 */
function serializeForDisplayInner(value: VmValue): string {
    if (typeof value === 'string') {
        if (value.length < MAX_WIDTH) {
            return serializeString(value);
        }
        return `${serializeString(value.slice(0, MAX_WIDTH))}..`;
    }
    if (isVmPrimitive(value)) {
        return serialize(value);
    }
    if (isVmArray(value)) {
        const len = value.length;
        if (!len) return '[]';
        return `[../* x${len} */]`;
    }
    if (isVmRecord(value)) {
        const len = Object.keys(value).length;
        if (!len) return '()';
        return `(../* x${len} */)`;
    }
    return `/* ${operations.$ToString(value)} */`;
}

/** 将值序列化为便于展示的字符串 */
export function serializeForDisplay(value: VmValue): string {
    if (isVmPrimitive(value) || isVmFunction(value)) {
        return serializeForDisplayInner(value);
    }
    let begin, end;
    const entries = [];
    let resultLength = 0;
    if (isVmArray(value)) {
        begin = '[';
        end = ']';
        for (const v of value) {
            if (entries.length > MAX_ENTRIES) {
                entries.push(`../* x${value.length - entries.length} */`);
                break;
            }
            const entry = serializeForDisplayInner(v ?? null);
            entries.push(entry);
            resultLength += entry.length;
        }
    } else if (isVmRecord(value)) {
        begin = '(';
        end = ')';
        const e = Object.entries(value);
        for (const [key, value] of e) {
            if (entries.length > MAX_ENTRIES) {
                entries.push(`../* x${e.length - entries.length} */`);
                break;
            }
            const entry = `${serializePropName(key)}: ${serializeForDisplayInner(value ?? null)}`;
            entries.push(entry);
            resultLength += entry.length;
        }
    } else {
        const hint = serializeForDisplayInner(value);
        const isArray = isVmExtern(value) && Array.isArray(value.value);
        begin = `${hint} ${isArray ? '[' : '('}`;
        end = isArray ? ']' : ')';
        const keys = value.keys();
        for (const [index, key] of keys.entries()) {
            if (entries.length > MAX_ENTRIES) {
                entries.push(`../* x${keys.length - entries.length} */`);
                break;
            }
            let entry;
            if (isArray && String(index) === key) {
                // 数组索引
                entry = serializeForDisplayInner(value.get(key) ?? null);
            } else {
                entry = `${serializePropName(key)}: ${serializeForDisplayInner(value.get(key) ?? null)}`;
            }
            entries.push(entry);
            resultLength += entry.length;
        }
    }
    if (resultLength >= MAX_WIDTH) {
        return `${begin}\n  ${entries.join(',\n  ')}\n${end}`;
    }
    return `${begin}${entries.join(', ')}${end}`;
}

/** 获取全局变量脚本 */
export function globalDoc(name: string, value: VmAny): { script: string; doc: string } {
    const info = getVmFunctionInfo(value);
    if (info) {
        return {
            script: signature(name, info),
            doc: globalFnDoc(info),
        };
    }
    if (isVmModule(value)) {
        return {
            script: `(module) ${name}`,
            doc: `模块 \`${name}\``,
        };
    }
    let valueStr;
    if (value === undefined) {
        valueStr = '/* ... */';
    } else {
        valueStr = serializeForDisplay(value);
    }
    return { script: `${name} = ${valueStr};`, doc: '' };
}

/** 获取深层属性 */
export function getDeep(value: VmAny, path: readonly string[]): VmValue {
    let current: VmAny = value;
    for (const key of path) {
        if (current == null) return null;
        current = operations.$Get(current, key);
    }
    return current ?? null;
}
