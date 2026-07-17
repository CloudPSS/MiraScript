import { DiagnosticCode } from '@mirascript/constants';
import { type editor, Range } from '../monaco-api.js';
import {
    getVmFunctionInfo,
    isVmArray,
    isVmExtern,
    isVmFunction,
    isVmModule,
    isVmPrimitive,
    isVmRecord,
    isVmWrapper,
    serialize,
    type VmAny,
    type VmModule,
    type VmFunctionInfo,
    type VmValue,
    type VmExtern,
} from '@mirascript/mirascript';
import { lib, operations, serializeRecordKey, serializeString } from '@mirascript/mirascript/subtle';
import type { LocalDefinition } from './compile-result.js';
import type { MonacoContext } from './providers/base.js';

const UNKNOWN_REPR = '/* .. */';

/** 参数签名 */
export type ParamSignature = [name: string, sig: string, doc: string];

/** 缩进 */
function indent(str: string, level: number, skipFirstLine = false): string {
    const prefix = ' '.repeat(level);
    const lines = str.split('\n');
    // remove leading and trailing empty lines
    while (lines.length && lines[0]!.trim() === '') {
        lines.shift();
    }
    while (lines.length && lines.at(-1)!.trim() === '') {
        lines.pop();
    }
    const lineCount = lines.length;
    for (let i = skipFirstLine ? 1 : 0; i < lineCount; i++) {
        lines[i] = prefix + lines[i];
    }
    return lines.join('\n');
}

/** 生成参数签名 */
function globalParamsSignature(info: VmFunctionInfo | undefined): ParamSignature[] {
    if (!info?.params) return [['..', '..', '']];
    const paramItems: ParamSignature[] = [];
    for (const [key, value] of Object.entries(info.params)) {
        const type = value.type ?? '';
        const doc = value.description ?? '';
        paramItems.push([key, `${key}: ${type || 'any'}`, doc ? `\`${key}\`: ${indent(doc, 0, true)}` : '']);
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
    const returns = info.returns?.type ? ` -> ${info.returns.type}` : '';
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
                p = `(\n${params.map((item) => indent(item[1], 2) + ',').join('\n')}\n)`;
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
        let argName: string;
        if (argsInParam.length > 0) {
            argName = argsInParam.map((arg) => model.getValueInRange(arg.definition.range)).join('_');
        } else if (rest) {
            argName = 'args';
        } else {
            argName = `arg${i}`;
        }
        if (rest) return [`..${argName}`, `..${argName}`, ''];
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
        for (const [key, { description }] of Object.entries(info.params)) {
            if (!description) continue;
            paramDoc.push(`- \`${key}\`: ${indent(description, 2, true)}`);
        }
    }
    if (info.returns?.description) {
        paramDoc.push(`- **返回值**: ${indent(info.returns.description, 2, true)}`);
    }
    if (paramDoc.length) {
        doc.push('', paramDoc.join('\n'));
    }
    if (info.examples?.length) {
        let exp = `### 示例`;
        for (const example of info.examples) {
            exp += codeblock(example);
        }
        doc.push('', exp);
    }
    return doc;
}

/** 获取代码块格式化字符串 */
export function codeblock(value: string): string {
    const lang = value.startsWith('\0') ? 'mirascript-doc' : 'mirascript';
    const includeFences = /`{3,}/.exec(value);
    const CODEBLOCK_FENCE = includeFences ? '`'.repeat(includeFences[0].length + 1) : '```';
    return `\n${CODEBLOCK_FENCE}${lang}\n${value}\n${CODEBLOCK_FENCE}\n`;
}

/** 格式化数字 */
function serializeIntegerImpl(num: number, base: number, prefix: string, sep: number): string {
    let str = Math.abs(num).toString(base);
    if (base > 10) str = str.toUpperCase();
    const sepSize = Math.abs(sep);
    if (sep !== 0 && str.length > sepSize) {
        const seg = [];
        if (sep > 0) {
            while (str.length > sepSize) {
                seg.unshift(str.slice(-sepSize));
                str = str.slice(0, -sepSize);
            }
            if (str.length > 0) {
                seg.unshift(str);
            }
        } else {
            while (str.length > sepSize) {
                seg.push(str.slice(0, sepSize));
                str = str.slice(sepSize);
            }
            if (str.length > 0) {
                seg.push(str);
            }
        }
        str = seg.join('_');
    }
    return (num < 0 ? '-' : '') + prefix + str;
}

/** 格式化数字 */
export function serializeInteger(num: number, base: 2 | 8 | 16, sep = true): string {
    const prefix = base === 2 ? '0b' : base === 8 ? '0o' : '0x';
    const sepSize = sep ? (base === 2 ? 8 : base === 8 ? 6 : 4) : 0;
    return serializeIntegerImpl(num, base, prefix, sepSize);
}

/** 格式化数字 */
export function serializeNumber(num: number): string {
    if (!Number.isFinite(num)) {
        return serialize(num);
    }
    const str = String(num);
    const dot = str.indexOf('.');
    const exp = str.indexOf('e');
    let intPart: string;
    let fracPart: string;
    let expPart: string;
    if (dot >= 0) {
        intPart = str.slice(0, dot);
        if (exp >= 0) {
            fracPart = str.slice(dot + 1, exp);
            expPart = str.slice(exp);
        } else {
            fracPart = str.slice(dot + 1);
            expPart = '';
        }
    } else {
        if (exp >= 0) {
            intPart = str.slice(0, exp);
            fracPart = '';
            expPart = str.slice(exp);
        } else {
            intPart = str;
            fracPart = '';
            expPart = '';
        }
    }
    if (intPart.length > 5) intPart = serializeIntegerImpl(Number(intPart), 10, '', 3);
    if (fracPart.length > 5) fracPart = serializeIntegerImpl(Number(fracPart), 10, '', -3);
    return intPart + (fracPart ? '.' + fracPart : '') + expPart;
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
    if (typeof value === 'number') {
        return serializeNumber(value);
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
    if (isVmExtern(value)) {
        return `/* <extern ${value.tag}> */`;
    }
    return `/* ${operations.$ToString(value)} */`;
}

/** 获取并序列化字段 */
function serializeField(obj: VmAny, key: string | number, maxWidth: number): string {
    const value = getField(obj, key);
    if (value === undefined) {
        return UNKNOWN_REPR;
    }
    return serializeForDisplayInner(value, maxWidth);
}

/** 将值序列化为便于展示的字符串 */
function serializeForDisplay(value: Exclude<VmValue, VmModule>, maxEntries = 100, maxWidth = 40): string {
    if (isVmPrimitive(value) || isVmFunction(value)) {
        return serializeForDisplayInner(value, maxWidth);
    }
    let begin, end;
    const entries = [];
    let resultLength = 0;
    if (isVmArray(value)) {
        begin = '[';
        end = ']';
        const len = value.length;
        if (len === 0) return '[]';
        for (let i = 0; i < len; i++) {
            if (entries.length > maxEntries) {
                entries.push(`../* x${value.length - entries.length} */`);
                break;
            }
            const entry = serializeField(value, i, maxWidth - 2);
            entries.push(entry);
            resultLength += entry.length;
        }
    } else if (isVmRecord(value)) {
        const keys = Object.keys(value);
        if (keys.length === 0) return '()';
        begin = '(';
        end = ')';

        for (const key of keys) {
            if (entries.length > maxEntries) {
                entries.push(`../* x${keys.length - entries.length} */`);
                break;
            }
            const sk = serializeRecordKey(key);
            const sv = serializeField(value, key, maxWidth - sk.length - 4);
            const entry = `${sk}: ${sv}`;
            entries.push(entry);
            resultLength += entry.length;
        }
    } else {
        const hint = serializeForDisplayInner(value satisfies VmExtern, 100);
        const isArray = value.isArrayLike();
        begin = `${hint} ${isArray ? '[' : '('}`;
        end = isArray ? ']' : ')';
        const keys = value.keys();
        if (keys.length === 0 && !isArray) {
            if (typeof value.value == 'object') {
                // 没有可枚举属性，尝试获取所有属性，仅在 LSP 进行展示时使用
                // 实际程序运行时这些属性仍然不可通过 `keys()` 或 `for .. in` 访问
                for (const key of Object.getOwnPropertyNames(value.value)) {
                    if (value.has(key)) keys.push(key);
                }
            } else {
                // 没有可枚举属性的函数
                begin = hint;
                end = '';
            }
        }
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
                const sk = serializeRecordKey(key);
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

/** 生成 doc comment */
export function docComment(doc: string[]): string[] {
    const lines = doc.flatMap((sec) => sec.split('\n').map((s) => s.trimEnd()));
    const firstLine = lines.findIndex((line) => line.length > 0);
    const lastLine = lines.findLastIndex((line) => line.length > 0);
    if (firstLine < 0 || lastLine < 0) return [];
    return [`/**`, ...lines.slice(firstLine, lastLine + 1).map((line) => ` * ${line}`), ` */`];
}

/** 获取变量文档 */
export function valueDoc(
    name: string,
    value: VmAny,
    type: 'field' | 'declare' | 'hint',
    parent: VmModule | VmExtern | MonacoContext | null,
): { script: string; doc: string[] } {
    const info = getVmFunctionInfo(value);
    const describe = parent?.describe?.(name);
    if (info) {
        const doc = globalFnDoc(info);
        if (describe && doc[0] !== describe) {
            doc.unshift(describe);
        }
        return {
            script: fnSignature(name, info).toString() + (type === 'declare' ? ';' : ''),
            doc,
        };
    }
    let prefix;
    let suffix = '';
    if (type === 'hint') {
        prefix = `${serializeRecordKey(name)} = `;
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
        let script;
        if (type === 'declare') {
            const exports = value.keys();
            if (!exports.length) {
                script = `mod ${value.name} { }`;
            } else {
                script = `mod ${value.name} {\n`;
                for (const k of exports) {
                    const v = value.get(k);
                    const vDoc = valueDoc(k, v, 'declare', value);
                    const code = [...docComment(vDoc.doc), 'pub ' + vDoc.script];
                    const lines = code.flatMap((line) => line.split('\n'));
                    for (const line of lines) {
                        script += line ? '  ' + line + '\n' : '\n';
                    }
                    script += '\n';
                }
                script = script.trimEnd() + '\n}';
            }
        } else {
            script = `mod ${value.name}`;
            if (value.name !== name) {
                script = `${prefix}${script}`;
            }
        }
        return { script, doc: [`模块 \`${value.name}\``] };
    }
    let valueStr = UNKNOWN_REPR;
    if (value !== undefined) {
        try {
            valueStr = serializeForDisplay(value, type === 'declare' ? 1000 : 100, type === 'declare' ? 80 : 40);
        } catch (ex) {
            // 序列化失败，保持默认值
        }
    }
    return {
        script: `${prefix}${valueStr}${suffix}`,
        doc: describe ? [describe] : [],
    };
}

/** 获取深层属性 */
export function getDeep(
    globals: MonacoContext,
    name: string,
    path: readonly string[],
): [parent: VmModule | VmExtern | MonacoContext | null, value: VmAny] {
    if (!globals.has(name)) return [null, undefined];
    let current: VmAny = globals.get(name);
    if (!path.length) {
        return [globals, current];
    }
    let parent: VmAny = null;
    for (const key of path) {
        if (current == null) return [null, undefined];
        if (!operations.$Has(current, key)) return [null, undefined];
        parent = current;
        current = operations.$Get(parent, key);
    }
    return [isVmWrapper(parent) ? parent : null, current];
}

/** 获取属性 */
export function getField(obj: VmAny, key: string | number): VmAny {
    if (obj == null) return undefined;
    try {
        if (!operations.$Has(obj, key)) return undefined;
    } catch {
        return undefined;
    }
    try {
        return operations.$Get(obj, key);
    } catch {
        return undefined;
    }
}

/** 列出属性 */
export function listFields(obj: VmAny, includeNonEnumerable: boolean): Array<string | number> {
    if (obj == null || typeof obj != 'object') return [];
    if (isVmWrapper(obj)) {
        try {
            return obj.keys(includeNonEnumerable);
        } catch {
            return [];
        }
    }
    return lib.keys(obj);
}

/** 是否已弃用 */
export function isDeprecatedGlobal(globals: MonacoContext, name: string): VmFunctionInfo['deprecated'] {
    if (!globals.has(name)) {
        return undefined;
    }
    const value = globals.get(name);
    const funcInfo = getVmFunctionInfo(value);
    if (funcInfo) {
        return funcInfo.deprecated;
    }
    const info = lib[name as 'PI'];
    if (!info?.deprecated) return undefined;
    if (info.value !== value) return undefined;
    const { use } = info.deprecated;
    if (use) {
        // Check that the replacement refers to the same value in the current context
        if (!globals.has(use)) return undefined;
        const replacement = globals.get(use);
        if (replacement !== info.value) return undefined;
    }
    return info.deprecated;
}
