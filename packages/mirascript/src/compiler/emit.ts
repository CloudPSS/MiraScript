import { OpCode } from '@mirascript/bindings';
import type { VmConst, VmPrimitive } from '../vm/index.js';
import { $ToString } from '../vm/operations.js';
import type { ScriptInput, TranspileOptions } from './types.js';
import type { IRange } from './diagnostic.js';
import { SourceMapGenerator } from 'source-map-js';

/** 生成代码 */
export function emit(
    source: ScriptInput,
    chunk: Uint8Array,
    sourcemaps: readonly IRange[],
    options: TranspileOptions,
): string {
    const gen = new Emitter(source, chunk, sourcemaps, options);
    gen.read();
    const code = gen.codeLines.join('\n');
    return code;
}

/** 解析常量 */
function readConst(reader: DataView, offset: number): [value: VmPrimitive, consumed: number] {
    const type = reader.getUint8(offset);
    switch (type) {
        /* c8 ignore next 2 */
        case 0:
            return [null, 1];
        case 1:
            return [true, 1];
        case 2:
            return [false, 1];
        case 3: {
            const ordinal = reader.getInt32(offset + 1, true);
            return [ordinal, 5];
        }
        case 4: {
            const num = reader.getFloat64(offset + 1, true);
            return [num, 9];
        }
        case 5: {
            const len = reader.getUint32(offset + 1, true);
            const str = new TextDecoder().decode(new Uint8Array(reader.buffer, reader.byteOffset + offset + 5, len));
            return [str, 5 + len];
        }
        /* c8 ignore next 2 */
        default:
            throw new Error(`Unknown constant type: ${type}`);
    }
}

/** 将值转为 JS */
function toJavascript(value: VmConst | undefined): string {
    /* c8 ignore next 2 */
    if (value === null) return 'null';
    if (value === undefined) return 'undefined';
    if (typeof value == 'object' || typeof value == 'string') {
        return JSON.stringify(value);
    }
    // JSON 无法处理 NaN 等特殊数字
    if (value === 0) {
        if (1 / value === -Infinity) return '-0';
        return '0';
    }
    return String(value);
}

const ORIGIN = `mira://MiraScript/`;
let sourceId = 1;

/** 创建数组 */
function createArray<T>(length: number, fn: (index: number) => T): T[] {
    // micro bench shows that this is faster than Array.from
    const result: T[] = [];
    for (let i = 0; i < length; i++) {
        result.push(fn(i));
    }
    return result;
}

const SCRIPT_PREFIX = `'use strict'; return ((global = GlobalFallback()) => { try { CpEnter();`;
const GLOBAL_HINT = `/* globals */`;
/** 代码生成 */
class Emitter {
    constructor(
        readonly source: ScriptInput,
        readonly chunk: Uint8Array,
        readonly sourcemaps: readonly IRange[],
        readonly options: TranspileOptions,
    ) {
        this.pretty = options.pretty ?? false;

        const reader = new DataView(chunk.buffer, chunk.byteOffset, chunk.byteLength);
        this.chunkSize = reader.getUint32(0, true);
        this.codeSize = reader.getUint32(4, true);
        this.constSize = reader.getUint32(8 + this.codeSize, true);

        this.codeReader = new DataView(chunk.buffer, chunk.byteOffset + 8, this.codeSize);
        this.constReader = new DataView(chunk.buffer, chunk.byteOffset + 12 + this.codeSize, this.constSize);
    }
    readonly pretty;
    readonly chunkSize: number;
    readonly codeSize: number;
    private readonly constReader: DataView;
    /** 读取常量表 */
    private readConsts(): void {
        for (let i = 0, index = 0; i < this.constSize; index++) {
            const [constant, size] = readConst(this.constReader, i);
            this.constVals.push(constant);
            this.constLits.push(toJavascript(constant));
            i += size;
        }
    }
    readonly constVals: VmPrimitive[] = [];
    readonly constLits: string[] = [];

    readonly constSize: number;
    private readonly codeReader: DataView;
    private codeOffset = 0;
    private closureCounter = 0;
    private identCounter = 0;

    /** 制造缩进 */
    private ident(len = 0): string {
        if (!this.pretty) return '';
        return '  '.repeat(this.identCounter + len);
    }

    readonly globals = new Map<number, [val: string, lit: string, eager: boolean, expr: string, name: string]>();
    /** 读取全局变量 */
    private rg(constIdx: number, eager: boolean): string {
        const cached = this.globals.get(constIdx);
        if (cached != null) {
            if (eager && !cached[2]) {
                cached[2] = true;
                cached[3] = cached[0];
            }
            return cached[3];
        }
        const constName = this.constVals[constIdx]!;
        const name = $ToString(constName);
        const lit = typeof constName == 'string' ? this.constLits[constIdx]! : JSON.stringify(name);
        const val = `g${this.globals.size + 1}`;
        const expr = eager ? val : `(${val} === undefined ? (${val} = global.get(${lit})) : ${val})`;
        this.globals.set(constIdx, [val, lit, eager, expr, name]);
        return expr;
    }

    readonly codeLines: string[] = [];

    /** Read variable */
    private rv(i: number, level = 0): string {
        if (!i) return 'null';
        const c = this.closureCounter - level;
        return `var_${c}_${i}`;
    }
    /** Write variable */
    private wv(i: number, level = 0): string {
        if (!i) return '_';
        return this.rv(i, level);
    }
    /** 读取 code param */
    private readParam(wide: boolean): number {
        const value = wide
            ? this.codeReader.getUint32(this.codeOffset, true)
            : this.codeReader.getUint8(this.codeOffset);
        this.codeOffset += wide ? 4 : 1;
        return value;
    }
    /** 读取 code param */
    private readIndex(wide: boolean): number {
        const value = wide ? this.codeReader.getInt32(this.codeOffset, true) : this.codeReader.getInt8(this.codeOffset);
        this.codeOffset += wide ? 4 : 1;
        return value;
    }
    /** 读取闭包 */
    private readClosure(): void {
        this.closureCounter++;
        this.identCounter++;
        while (this.codeOffset < this.codeSize) {
            const opcode_raw = this.codeReader.getUint8(this.codeOffset);
            const opcode = opcode_raw & 0x7f;
            if (opcode !== OpCode.FuncEnd) {
                this.readCode();
                continue;
            }
            this.codeOffset++;
            const body = this.ident(-1) + `} finally { CpExit(); } });`;
            this.codeLines.push(body);
            this.closureCounter--;
            this.identCounter--;
            break;
        }
    }

    /** 读取块结束 */
    private readBlockEnd(end: OpCode): void {
        while (this.codeOffset < this.codeSize) {
            const opcode_raw = this.codeReader.getUint8(this.codeOffset);
            const opcode = opcode_raw & 0x7f;
            if (opcode !== end) {
                this.readCode();
                continue;
            }
            this.codeOffset++;
            this.identCounter--;
            if (end === OpCode.LoopEnd) {
                this.closureCounter--;
            }
            const body = this.ident() + `};`;
            this.codeLines.push(body);
            break;
        }
    }

    /** 读取 if else 或 if 结束 */
    private readIfElse(): void {
        this.identCounter++;
        while (this.codeOffset < this.codeSize) {
            const opcode_raw = this.codeReader.getUint8(this.codeOffset);
            const opcode = opcode_raw & 0x7f;
            if (opcode === OpCode.IfEnd) {
                return this.readBlockEnd(OpCode.IfEnd);
            }
            if (opcode === OpCode.Else) {
                this.codeOffset++;
                const body = this.ident(-1) + `} else {`;
                this.codeLines.push(body);
                break;
            }
            if (opcode === OpCode.ElIf) {
                this.codeOffset++;
                const body = this.ident(-1) + `} else `;
                this.codeLines.push(body);
                return this.readCode();
            }
            this.readCode();
        }
        return this.readBlockEnd(OpCode.IfEnd);
    }

    /** 读取 record */
    private readRecord(obj: number): void {
        this.identCounter++;
        while (this.codeOffset < this.codeSize) {
            const opcode_raw = this.codeReader.getUint8(this.codeOffset++);
            const opcode = opcode_raw & 0x7f;
            const wide = opcode_raw >= 0x80;
            const read = () => this.readParam(wide);
            let code = '';
            switch (opcode) {
                case OpCode.FieldOpt:
                case OpCode.Field: {
                    const field = read();
                    const field_name = this.constLits[field];
                    /* c8 ignore next 3 */
                    if (!field_name) {
                        throw new Error(`Unknown field ${field}`);
                    }
                    const value = read();
                    const opt = opcode === OpCode.FieldOpt;
                    // Use computed property names to avoid prototype pollution
                    code = opt
                        ? `...ElementOpt(${field_name}, ${this.rv(value)}),`
                        : `[${field_name}]: Element(${this.rv(value)}),`;
                    break;
                }
                case OpCode.FieldOptDyn:
                case OpCode.FieldDyn: {
                    const field = read();
                    const value = read();
                    const opt = opcode === OpCode.FieldOptDyn;
                    code = opt
                        ? `...ElementOpt(${this.rv(field)}, ${this.rv(value)}),`
                        : `[${this.rv(field)}]: Element(${this.rv(value)}),`;
                    break;
                }
                case OpCode.FieldOptIndex:
                case OpCode.FieldIndex: {
                    const field = this.readIndex(wide);
                    const value = read();
                    const opt = opcode === OpCode.FieldOptIndex;
                    code = opt
                        ? `...ElementOpt(${field}, ${this.rv(value)}),`
                        : `[${field}]: Element(${this.rv(value)}),`;
                    break;
                }
                case OpCode.Spread: {
                    const value = read();
                    code = `...$RecordSpread(${this.rv(value)}),`;
                    break;
                }
                case OpCode.Freeze: {
                    this.identCounter--;
                    code = `});`;
                    break;
                }
                default: {
                    code = `// ?${OpCode[opcode] ?? opcode}`;
                    break;
                }
            }
            const ident = this.ident();
            this.codeLines.push(ident + code);
            if (opcode === OpCode.Freeze) {
                return;
            }
        }
    }

    /** 读取 array */
    private readArray(arr: number): void {
        this.identCounter++;
        while (this.codeOffset < this.codeSize) {
            const opcode_raw = this.codeReader.getUint8(this.codeOffset++);
            const opcode = opcode_raw & 0x7f;
            const wide = opcode_raw >= 0x80;
            const read = () => this.readParam(wide);
            let code = '';
            switch (opcode) {
                case OpCode.Item: {
                    const value = read();
                    code = `Element(${this.rv(value)}),`;
                    break;
                }
                case OpCode.ItemRange: {
                    const start = this.readIndex(wide);
                    const end = this.readIndex(wide);
                    code = `...ArrayRange(${start}, ${end}),`;
                    break;
                }
                case OpCode.ItemRangeDyn: {
                    const start = read();
                    const end = read();
                    code = `...ArrayRange(${this.rv(start)}, ${this.rv(end)}),`;
                    break;
                }
                case OpCode.ItemRangeExclusiveDyn: {
                    const start = read();
                    const end = read();
                    code = `...ArrayRangeExclusive(${this.rv(start)}, ${this.rv(end)}),`;
                    break;
                }
                case OpCode.Spread: {
                    const value = read();
                    code = `...$ArraySpread(${this.rv(value)}),`;
                    break;
                }
                case OpCode.Freeze: {
                    this.identCounter--;
                    code = `]);`;
                    break;
                }
                default: {
                    code = `// ?${OpCode[opcode] ?? opcode}`;
                    break;
                }
            }
            const ident = this.ident();
            this.codeLines.push(ident + code);
            if (opcode === OpCode.Freeze) {
                return;
            }
        }
    }

    /** 读取代码 */
    private readCode(): void {
        const opcode_raw = this.codeReader.getUint8(this.codeOffset++);
        const opcode = opcode_raw & 0x7f;
        const wide = opcode_raw >= 0x80;
        const read = () => this.readParam(wide);
        const readIndex = () => this.readIndex(wide);
        const ident = this.ident();
        let code = '';
        let reg = 0;
        switch (opcode) {
            case OpCode.FuncVarg:
            case OpCode.Func: {
                const startFunc = this.codeOffset === 1;
                reg = read();
                const varg = opcode === OpCode.FuncVarg;
                const argn = read();
                const regn = read();
                const args = createArray(argn, (i) => {
                    const wv = this.wv(i + 1, -1);
                    if (varg && i === argn - 1) {
                        // 最后一个参数为可变参数
                        return `...vargs`;
                    }
                    return `${wv} = null`;
                });
                const regs = createArray(regn - argn + 1, (i) => (i ? this.wv(i + argn, -1) : this.wv(0, -1))).join(
                    ', ',
                );
                const script = startFunc && !varg && argn === 0;
                if (script) {
                    code = `${SCRIPT_PREFIX} var ${regs};`;
                } else {
                    code = `${this.wv(reg)} = Function((${args.join(', ')}) => { try { CpEnter(); var ${regs};`;
                }
                if (varg) {
                    code += ` var ${this.wv(argn, -1)} = Vargs(vargs);`;
                }
                break;
            }
            case OpCode.Constant: {
                reg = read();
                const i = read();
                const c = this.constLits[i];
                code = `${this.wv(reg)} = ${c};`;
                break;
            }
            case OpCode.Uninit: {
                reg = read();
                code = `${this.wv(reg)} = undefined;`;
                break;
            }
            case OpCode.Return: {
                reg = read();
                code = `return ${this.rv(reg)};`;
                break;
            }
            case OpCode.Add:
            case OpCode.Sub:
            case OpCode.Mul:
            case OpCode.Div:
            case OpCode.Mod:
            case OpCode.Pow:
            case OpCode.Gt:
            case OpCode.Gte:
            case OpCode.Lt:
            case OpCode.Lte:
            case OpCode.Eq:
            case OpCode.Neq:
            case OpCode.Aeq:
            case OpCode.Naeq:
            case OpCode.Same:
            case OpCode.Nsame:
            case OpCode.In:
            case OpCode.And:
            case OpCode.Or:
            case OpCode.Format: {
                reg = read();
                const left = read();
                const right = read();
                code = `${this.wv(reg)} = $${OpCode[opcode]}(${this.rv(left)}, ${this.rv(right)});`;
                break;
            }
            case OpCode.InGlobal: {
                reg = read();
                const left = read();
                code = `${this.wv(reg)} = global.has($ToString(${this.rv(left)}));`;
                break;
            }
            case OpCode.Concat: {
                reg = read();
                const n = read();
                const args = createArray(n, () => read());
                code = `${this.wv(reg)} = $${OpCode[opcode]}(${args.map((a) => this.rv(a)).join(', ')});`;
                break;
            }
            case OpCode.Omit:
            case OpCode.Pick: {
                reg = read();
                const value = read();
                const n = read();
                const args = createArray(n, () => this.constLits[read()]!);

                code = `${this.wv(reg)} = $${OpCode[opcode]}(${this.rv(value)}, [${args.join(', ')}]);`;
                break;
            }
            case OpCode.Call:
            case OpCode.CallDyn: {
                reg = read();
                const func = read();
                const n = read();
                const args = createArray(n, () => read());
                const ns = read();
                const spreads = createArray(ns, () => read());
                const callTarget = opcode === OpCode.Call ? this.rg(func, false) : this.rv(func);
                code = `${this.wv(reg)} = $Call(${callTarget}, [${args
                    .map((a, i) => {
                        if (spreads.includes(i)) return `...$ArraySpread(${this.rv(a)})`;
                        else return this.rv(a);
                    })
                    .join(', ')}]);`;
                break;
            }
            case OpCode.Assign: {
                reg = read();
                const value = read();
                code = `${this.wv(reg)} = ${this.rv(value)};`;
                break;
            }
            case OpCode.Pos:
            case OpCode.Neg:
            case OpCode.Not:
            case OpCode.Type:
            case OpCode.ToBoolean:
            case OpCode.ToNumber:
            case OpCode.ToString:
            case OpCode.IsBoolean:
            case OpCode.IsNumber:
            case OpCode.IsString:
            case OpCode.IsRecord:
            case OpCode.IsArray:
            case OpCode.Length: {
                reg = read();
                const value = read();
                code = `${this.wv(reg)} = $${OpCode[opcode]}(${this.rv(value)});`;
                break;
            }
            case OpCode.AssertInit:
            case OpCode.AssertNonNil: {
                reg = read();
                code = `$${OpCode[opcode]}(${this.rv(reg)})`;
                break;
            }
            case OpCode.Get: {
                reg = read();
                const obj = read();
                const prop = this.constLits[read()];
                code = `${this.wv(reg)} = $Get(${this.rv(obj)}, ${prop});`;
                break;
            }
            case OpCode.GetIndex: {
                reg = read();
                const obj = read();
                const index = readIndex();
                code = `${this.wv(reg)} = $Get(${this.rv(obj)}, ${index});`;
                break;
            }
            case OpCode.GetDyn: {
                reg = read();
                const obj = read();
                const index = read();
                code = `${this.wv(reg)} = $Get(${this.rv(obj)}, ${this.rv(index)});`;
                break;
            }
            case OpCode.Has: {
                reg = read();
                const obj = read();
                const prop = this.constLits[read()];
                code = `${this.wv(reg)} = $Has(${this.rv(obj)}, ${prop});`;
                break;
            }
            case OpCode.HasIndex: {
                reg = read();
                const obj = read();
                const index = readIndex();
                code = `${this.wv(reg)} = $Has(${this.rv(obj)}, ${index});`;
                break;
            }
            case OpCode.HasDyn: {
                reg = read();
                const obj = read();
                const index = read();
                code = `${this.wv(reg)} = $Has(${this.rv(obj)}, ${this.rv(index)});`;
                break;
            }
            case OpCode.Set: {
                reg = read();
                const obj = read();
                const prop = this.constLits[read()];
                code = `$Set(${this.rv(obj)}, ${prop}, ${this.rv(reg)});`;
                break;
            }
            case OpCode.SetIndex: {
                reg = read();
                const obj = read();
                const index = readIndex();
                code = `$Set(${this.rv(obj)}, ${index}, ${this.rv(reg)});`;
                break;
            }
            case OpCode.SetDyn: {
                reg = read();
                const obj = read();
                const index = read();
                code = `$Set(${this.rv(obj)}, ${this.rv(index)}, ${this.rv(reg)});`;
                break;
            }
            case OpCode.GetGlobal: {
                reg = read();
                const i = read();
                code = `${this.wv(reg)} = ${this.rg(i, false)};`;
                break;
            }
            case OpCode.GetGlobalDyn: {
                reg = read();
                const name = read();
                code = `${this.wv(reg)} = global.get($ToString(${this.rv(name)}));`;
                break;
            }
            case OpCode.GetUpvalue: {
                reg = read();
                const level = read();
                const up = read();
                code = `${this.wv(reg)} = Upvalue(${this.rv(up, level)});`;
                break;
            }
            case OpCode.SetUpvalue: {
                reg = read();
                const level = read();
                const up = read();
                code = `${this.wv(up, level)} = ${this.rv(reg)};`;
                break;
            }
            case OpCode.Slice: {
                reg = read();
                const obj = read();
                const start = readIndex();
                const end = readIndex();
                code = `${this.wv(reg)} = $Slice(${this.rv(obj)}, ${start}, ${end});`;
                break;
            }
            case OpCode.SliceStart: {
                reg = read();
                const obj = read();
                const end = readIndex();
                code = `${this.wv(reg)} = $Slice(${this.rv(obj)}, null, ${end});`;
                break;
            }
            case OpCode.SliceEnd: {
                reg = read();
                const obj = read();
                const start = readIndex();
                code = `${this.wv(reg)} = $Slice(${this.rv(obj)}, ${start}, null);`;
                break;
            }
            case OpCode.SliceDyn: {
                reg = read();
                const obj = read();
                const start = read();
                const end = read();
                code = `${this.wv(reg)} = $Slice(${this.rv(obj)}, ${this.rv(start)}, ${this.rv(end)});`;
                break;
            }
            case OpCode.SliceExclusiveDyn: {
                reg = read();
                const obj = read();
                const start = read();
                const end = read();
                code = `${this.wv(reg)} = $SliceExclusive(${this.rv(obj)}, ${this.rv(start)}, ${this.rv(end)});`;
                break;
            }
            case OpCode.Record: {
                reg = read();
                code = `${this.wv(reg)} = ({`;
                break;
            }
            case OpCode.Array: {
                reg = read();
                code = `${this.wv(reg)} = ([`;
                break;
            }
            case OpCode.If: {
                const cond = read();
                code = `if (${this.rv(cond)} !== false && $ToBoolean(${this.rv(cond)}) !== false) {`;
                break;
            }
            case OpCode.IfNot: {
                const cond = read();
                code = `if (${this.rv(cond)} !== true && $ToBoolean(${this.rv(cond)}) !== true) {`;
                break;
            }
            case OpCode.IfInit: {
                const cond = read();
                code = `if (${this.rv(cond)} !== undefined) {`;
                break;
            }
            case OpCode.IfNotInit: {
                const cond = read();
                code = `if (${this.rv(cond)} === undefined) {`;
                break;
            }
            case OpCode.IfNil: {
                const cond = read();
                code = `if (${this.rv(cond)} === null) {`;
                break;
            }
            case OpCode.IfNotNil: {
                const cond = read();
                code = `if (${this.rv(cond)} !== null) {`;
                break;
            }
            case OpCode.LoopFor: {
                const nreg = read();
                const iterable = read();
                const regs = createArray(nreg - 1, (i) => this.wv(i + 2, -1));
                regs.unshift('_');
                const ir = this.wv(1, -1);
                code = `for (let ${ir} of $Iterable(${this.rv(iterable)})) { ${ir} ??= null; Cp(); let ${regs.join(', ')};`;
                break;
            }
            case OpCode.LoopRange:
            case OpCode.LoopRangeExclusive: {
                const nreg = read();
                const start = read();
                const end = read();
                const exclusive = opcode === OpCode.LoopRangeExclusive;
                const regs = createArray(nreg - 1, (i) => this.wv(i + 2, -1));
                regs.unshift('_');
                const i = this.wv(1, -1);
                code = `for (let start = $ToNumber(${this.rv(start)}), end = $ToNumber(${this.rv(end)}), ${i} = start; ${i} ${exclusive ? '<' : '<='} end; ${i} += 1) { Cp(); let ${regs.join(', ')};`;
                break;
            }
            case OpCode.Loop: {
                const nreg = read();
                const regs = createArray(nreg, (i) => this.wv(i + 1, -1));
                regs.unshift('_');
                code = `while (true) { Cp(); let ${regs.join(', ')};`;
                break;
            }
            case OpCode.Break: {
                code = `break;`;
                break;
            }
            case OpCode.Continue: {
                code = `continue;`;
                break;
            }
            case OpCode.Noop: {
                return;
            }
            default: {
                code = `; // ${OpCode[opcode] ?? opcode}`;
                break;
            }
        }
        this.codeLines.push(ident + code);
        switch (opcode) {
            case OpCode.FuncVarg:
            case OpCode.Func: {
                this.readClosure();
                break;
            }
            case OpCode.If:
            case OpCode.IfNot:
            case OpCode.IfNil:
            case OpCode.IfNotNil:
            case OpCode.IfInit:
            case OpCode.IfNotInit: {
                this.readIfElse();
                break;
            }
            case OpCode.Loop:
            case OpCode.LoopFor:
            case OpCode.LoopRange:
            case OpCode.LoopRangeExclusive: {
                this.identCounter++;
                this.closureCounter++;
                this.readBlockEnd(OpCode.LoopEnd);
                break;
            }
            case OpCode.Record: {
                this.readRecord(reg);
                break;
            }
            case OpCode.Array: {
                this.readArray(reg);
                break;
            }
        }
    }

    /** 读取 chunk */
    read(): void {
        this.readConsts();
        this.readCode();
        if (this.globals.size > 0) {
            let globalsInit = '';
            for (const [val, lit, eager] of this.globals.values()) {
                const expr = eager ? `${val} = global.get(${lit})` : val;
                globalsInit += globalsInit ? `, ${expr}` : `var ${GLOBAL_HINT} ${expr}`;
            }
            this.codeLines[0] += globalsInit + ';';
        }
        this.addSourceMap();
    }
    /** 添加源映射 */
    addSourceMap(): void {
        if (!this.options.sourceMap) return;
        let fileName = (this.options.fileName ?? '').trim();
        const hasSchema = /^\w+:/.test(fileName);
        if (!hasSchema) {
            if (fileName.startsWith('/')) {
                fileName = fileName.replace(/^\/+\s*/, '');
            }
            if (!fileName) {
                fileName = `${sourceId++}.${this.options.input_mode === 'Template' ? 'miratpl' : 'mira'}`;
            }
        }
        const map = new SourceMapGenerator({
            file: fileName + '.js',
        });
        if (typeof this.source === 'string') {
            map.setSourceContent(fileName, this.source);
        }
        let hasStartMap = false;
        for (let i = 1; i < this.sourcemaps.length; i++) {
            const range = this.sourcemaps[i];
            if (!range) break;
            if (!hasStartMap && range.startLineNumber === 1 && range.startColumn === 1) {
                hasStartMap = true;
            }
            map.addMapping({
                generated: {
                    // 前两行固定为：
                    // (function anonymous($Add,$Aeq, ...
                    // ) {
                    line: i + 3,
                    column: 0,
                },
                original: {
                    line: range.startLineNumber,
                    column: range.startColumn - 1,
                },
                source: fileName,
            });
        }
        if (!hasStartMap) {
            map.addMapping({
                generated: {
                    line: 3,
                    column: SCRIPT_PREFIX.length - 'CpEnter();'.length,
                },
                original: {
                    line: 1,
                    column: 0,
                },
                source: fileName,
            });
        }
        {
            const line0 = this.codeLines[0]!;
            const file = `${fileName} <globals>`;
            let globals = `global;\n`;
            map.addMapping({
                generated: {
                    line: 3,
                    column: line0.indexOf(`global = `),
                },
                original: {
                    line: 1,
                    column: 0,
                },
                source: file,
                name: 'global',
            });
            map.addMapping({
                generated: {
                    line: 3,
                    column: SCRIPT_PREFIX.length,
                },
                original: {
                    line: 1,
                    column: 7,
                },
                source: file,
                name: '',
            });
            let i = 1;
            let pos = line0.indexOf(GLOBAL_HINT, SCRIPT_PREFIX.length) + GLOBAL_HINT.length;
            for (const p of this.globals.values()) {
                i++;
                if (pos < 0) break;
                const val = p[0];
                pos = line0.indexOf(val, pos);
                if (pos < 0) break;
                const name = p[4];
                map.addMapping({
                    generated: {
                        line: 3,
                        column: pos,
                    },
                    original: {
                        line: i,
                        column: 0,
                    },
                    source: file,
                    name,
                });
                globals += `${name};\n`;
            }
            map.addMapping({
                generated: {
                    line: 3,
                    column: pos,
                },
                original: {
                    line: i,
                    column: 0,
                },
                source: file,
                name: '',
            });
            map.setSourceContent(file, globals);
        }
        const prefix = '//# ';
        const sourceURL = hasSchema ? fileName : `${ORIGIN}${fileName}`;
        this.codeLines.push(
            // Prevent source map from being recognized as of this file
            `${prefix}sourceURL=${sourceURL}.js`,
            `${prefix}sourceMappingURL=data:application/json,${encodeURIComponent(map.toString())}`,
        );
    }
}
