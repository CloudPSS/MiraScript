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
} from '@mirascript/mirascript';
import { operations, serializePropName, serializeString } from '@mirascript/mirascript/subtle';
import type { LocalDefinition } from './compile-result.js';

/** 参数签名 */
export type ParamSignature = [name: string, sig: string, doc: string];

/** 生成参数签名 */
function globalParamsSignature(info: VmFunctionInfo | undefined): ParamSignature[] {
    if (!info?.params) return [['..', '..', '']];
    const paramItems: ParamSignature[] = [];
    for (const key of Object.keys(info.params)) {
        const type = info.paramsType?.[key];
        const doc = info.params[key] ?? '';
        paramItems.push([key, type ? `${key}: ${type}` : key, doc ? `\`${key}\`: ${doc}` : '']);
    }
    return paramItems;
}
const SIG_WIDTH = 60;
/** 生成函数签名 */
export function fnSignature(
    id: string | undefined,
    info: VmFunctionInfo,
): {
    params: ParamSignature[];
    returns: string;
    /** @inheritdoc */
    toString(): string;
} {
    const prefix = id ? `fn ${id}` : 'fn';
    const params = globalParamsSignature(info);
    const returns = info.returnsType ? ` -> ${info.returnsType}` : '';
    return {
        params,
        returns,
        toString() {
            let p;
            if (
                this.params.length >= 1 &&
                (prefix.length + this.returns.length > SIG_WIDTH ||
                    prefix.length + this.returns.length + params.reduce((a, b) => a + b[1].length, 0) > SIG_WIDTH)
            ) {
                p = `(\n${params.map((item) => `  ${item[1]},`).join('\n')}\n)`;
            } else {
                p = `(${params.map((item) => item[1]).join(', ')})`;
            }
            return `${prefix}${p}${this.returns}`;
        },
    };
}
/** 生成函数参数 */
export function localParamSignature(
    model: editor.ITextModel,
    info: NonNullable<LocalDefinition['fn']>,
): ParamSignature[] {
    const {
        args,
        scope: { params },
    } = info;
    if (params[0]?.code === DiagnosticCode.ParameterIt) {
        return params[0].references.length ? [['it', 'it', '']] : [];
    }
    return params.map((a, i) => {
        const rest =
            a.code === DiagnosticCode.ParameterRestPattern ||
            a.code === DiagnosticCode.ParameterMutableRest ||
            a.code === DiagnosticCode.ParameterImmutableRest;
        const argsInParam = args.filter((arg) => Range.containsRange(a.range, arg.definition.range));
        const argName =
            argsInParam.length === 0
                ? `arg_${i}`
                : argsInParam.map((arg) => model.getValueInRange(arg.definition.range)).join('_');
        if (rest) return ['...${argName}', '...${argName}', ''];
        return [argName, argName, ''];
    });
}

/** 生成函数参数列表 */
export function paramsList(model: editor.ITextModel, info: VmFunctionInfo | LocalDefinition['fn'] | undefined): string {
    if (!info) return '(..)';
    if ('scope' in info) {
        return `(${localParamSignature(model, info)
            .map((p) => p[1])
            .join(', ')})`;
    } else {
        if (!info.params) return '(..)';
        const paramItems = Object.keys(info.params).join(', ');
        return `(${paramItems})`;
    }
}

/** 生成函数文档 */
export function globalFnDoc(info: VmFunctionInfo): string[] {
    const doc = [];
    if (info.summary) {
        doc.push(info.summary);
    }
    const paramDoc = [];
    if (info.params) {
        for (const [key, value] of Object.entries(info.params)) {
            paramDoc.push(`- \`${key}\`: ${value}`);
        }
    }
    if (info.returns) {
        paramDoc.push(`- **返回值**: ${info.returns}`);
    }
    if (paramDoc.length) {
        doc.push(paramDoc.join('\n'));
    }
    if (info.examples?.length) {
        let exp = `### 示例`;
        for (const example of info.examples) {
            exp += codeblock(example);
        }
        doc.push(exp);
    }
    return doc;
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

/** 获取单词 */
export function wordAt(model: editor.ITextModel, position: IPosition): { word: string; range: Range } | undefined {
    const word = model.getWordAtPosition(position);
    if (!word) return undefined;
    const range = new Range(position.lineNumber, word.startColumn, position.lineNumber, word.endColumn);
    return { word: word.word, range };
}

/** 将值序列化为便于展示的字符串 */
function serializeForDisplayInner(value: VmValue, maxWidth: number): string {
    if (maxWidth < 10) maxWidth = 10;
    if (typeof value === 'string') {
        if (value.length < maxWidth) {
            return serializeString(value);
        }
        return `${serializeString(value.slice(0, maxWidth))}..`;
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
export function serializeForDisplay(value: VmValue, maxEntries = 100, maxWidth = 40): string {
    if (isVmPrimitive(value) || isVmFunction(value)) {
        return serializeForDisplayInner(value, maxWidth);
    }
    let begin, end;
    const entries = [];
    let resultLength = 0;
    if (isVmArray(value)) {
        begin = '[';
        end = ']';
        for (const v of value) {
            if (entries.length > maxEntries) {
                entries.push(`../* x${value.length - entries.length} */`);
                break;
            }
            const entry = serializeForDisplayInner(v ?? null, maxWidth - 2);
            entries.push(entry);
            resultLength += entry.length;
        }
    } else if (isVmRecord(value)) {
        begin = '(';
        end = ')';
        const e = Object.entries(value);
        for (const [key, value] of e) {
            if (entries.length > maxEntries) {
                entries.push(`../* x${e.length - entries.length} */`);
                break;
            }
            const sk = serializePropName(key);
            const entry = `${sk}: ${serializeForDisplayInner(value ?? null, maxWidth - sk.length - 4)}`;
            entries.push(entry);
            resultLength += entry.length;
        }
    } else {
        const hint = serializeForDisplayInner(value, 100);
        const isArray = isVmExtern(value) && Array.isArray(value.value);
        begin = `${hint} ${isArray ? '[' : '('}`;
        end = isArray ? ']' : ')';
        const keys = value.keys();
        for (const [index, key] of keys.entries()) {
            if (entries.length > maxEntries) {
                entries.push(`../* x${keys.length - entries.length} */`);
                break;
            }
            let entry;
            if (isArray && String(index) === key) {
                // 数组索引
                entry = serializeForDisplayInner(value.get(key) ?? null, maxWidth - 2);
            } else {
                const sk = serializePropName(key);
                entry = `${sk}: ${serializeForDisplayInner(value.get(key) ?? null, maxWidth - sk.length - 4)}`;
            }
            entries.push(entry);
            resultLength += entry.length;
        }
    }
    if (resultLength >= maxWidth) {
        return `${begin}\n  ${entries.join(',\n  ')}\n${end}`;
    }
    return `${begin}${entries.join(', ')}${end}`;
}

/** 获取变量文档 */
export function valueDoc(
    name: string,
    value: VmAny,
    type: 'field' | 'declare' | 'hint',
): { script: string; doc: string[] } {
    const info = getVmFunctionInfo(value);
    if (info) {
        return {
            script: fnSignature(name, info).toString() + (type === 'declare' ? ';' : ''),
            doc: globalFnDoc(info),
        };
    }
    let prefix;
    let suffix = '';
    if (type === 'hint') {
        prefix = `${name} = `;
    } else if (type === 'declare') {
        if (name.startsWith('@')) {
            prefix = `const ${name} = `;
        } else {
            prefix = `let ${name} = `;
        }
        suffix = ';';
    } else if (/^\d/.test(name)) {
        prefix = `[${name}]: `;
    } else {
        prefix = `${name}: `;
    }
    if (isVmModule(value)) {
        const doc = `模块 \`${value.name}\``;
        let script;
        if (type === 'declare') {
            const exports = value.keys();
            script = '\n';
            for (const k of exports) {
                const v = value.get(k);
                const vDoc = valueDoc(k, v, isVmModule(v) ? 'field' : 'declare');
                const code = [
                    `/**`,
                    ...vDoc.doc.flatMap((sec) => sec.split('\n')).map((line) => ` * ${line}`),
                    ` */`,
                    'export ' + vDoc.script,
                    '',
                    '',
                ];
                script += code.join('\n');
            }
            script = script.trimEnd();
        } else {
            script = `(module) ${value.name}`;
            if (value.name !== name) {
                script = `${prefix}${script}`;
            }
        }
        return { script, doc: doc ? [doc] : [] };
    }
    let valueStr;
    if (value === undefined) {
        valueStr = '/* ... */';
    } else {
        valueStr = serializeForDisplay(value, type === 'declare' ? 1000 : 100, type === 'declare' ? 80 : 40);
    }
    return { script: `${prefix}${valueStr}${suffix}`, doc: [] };
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
