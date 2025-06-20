import { OpCode } from '@mirascript/wasm';
import type { VmConst, VmPrimitive } from '../vm';
import type { ScriptInput, TranspileOptions } from './types';
import { encodeURL } from 'js-base64';

/** 生成代码 */
export function emit(source: ScriptInput, chunk: Uint8Array, options: TranspileOptions): string {
    const gen = new Emitter(source, chunk, options);
    gen.read();
    const code = gen.codeLines.join('\n');
    return code;
}

/** 解析常量 */
function readConst(reader: DataView, offset: number): [value: VmPrimitive, consumed: number] {
    const type = reader.getUint8(offset);
    switch (type) {
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
        default:
            throw new Error(`Unknown constant type: ${type}`);
    }
}

/** 将值转为 JS */
function toJavascript(value: VmConst | undefined): string {
    if (value === null) return 'null';
    if (value === undefined) return 'undefined';
    if (typeof value == 'object' || typeof value == 'string') {
        return JSON.stringify(value);
    }
    // JSON 无法处理 NaN 等特殊数字
    return String(value);
}

const ORIGIN = `mira://MiraScript`;
let sourceId = 1;

/** 代码生成 */
class Emitter {
    constructor(
        readonly source: ScriptInput,
        readonly chunk: Uint8Array,
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
            this.constants.push(toJavascript(constant));
            i += size;
        }
    }
    readonly constants: string[] = [];

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
                    const field_name = this.constants[field];
                    if (!field_name) throw new Error(`Unknown field ${field}`);
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
                    const field = read();
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
                    const start = read();
                    const end = read();
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
        const ident = this.ident();
        let code = '';
        let reg = 0;
        switch (opcode) {
            case OpCode.FuncVarg:
            case OpCode.Func: {
                const script = this.codeOffset === 1;
                reg = read();
                const varg = opcode === OpCode.FuncVarg;
                const argn = read();
                const regn = read();
                const args = Array.from({ length: argn }, (_, i) => {
                    const wv = this.wv(i + 1, -1);
                    if (varg && i === argn - 1) {
                        // 最后一个参数为可变参数
                        return `...${wv}`;
                    }
                    return `${wv} = null`;
                });
                const regs = Array.from({ length: regn - argn + 1 }, (_, i) =>
                    i ? this.wv(i + argn, -1) : this.wv(0, -1),
                ).join(', ');
                if (script) {
                    args.unshift(`global = GlobalFallback()`);
                    code = `return (function script(${args.join(', ')}) { try{ CpEnter(); let ${regs};`;
                } else {
                    code = `${this.wv(reg)} = Function(function (${args.join(', ')}) { try{ CpEnter(); let ${regs};`;
                }
                break;
            }
            case OpCode.Constant: {
                reg = read();
                const i = read();
                const c = this.constants[i];
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
            case OpCode.Or: {
                reg = read();
                const left = read();
                const right = read();
                code = `${this.wv(reg)} = $${OpCode[opcode]}(${this.rv(left)}, ${this.rv(right)});`;
                break;
            }
            case OpCode.InGlobal: {
                reg = read();
                const left = read();
                code = `${this.wv(reg)} = global[${this.rv(left)}] !== undefined;`;
                break;
            }
            case OpCode.Concat: {
                reg = read();
                const n = read();
                const args = Array.from({ length: n }, (_, i) => read());
                code = `${this.wv(reg)} = $${OpCode[opcode]}(${args.map((a) => this.rv(a)).join(', ')});`;
                break;
            }
            case OpCode.Omit:
            case OpCode.Pick: {
                reg = read();
                const value = read();
                const n = read();
                const args = Array.from({ length: n }, (_, i) => this.constants[read()]!);

                code = `${this.wv(reg)} = $${OpCode[opcode]}(${this.rv(value)}, [${args.join(', ')}]);`;
                break;
            }
            case OpCode.Call: {
                reg = read();
                const func = read();
                const n = read();
                const args = Array.from({ length: n }, (_, i) => read());
                const funcName = this.constants[func];
                code = `${this.wv(reg)} = $Call(global[${funcName}], [${args.map((a) => this.rv(a)).join(', ')}]);`;
                break;
            }
            case OpCode.CallDyn: {
                reg = read();
                const func = read();
                const n = read();
                const args = Array.from({ length: n }, (_, i) => read());
                code = `${this.wv(reg)} = $Call(${this.rv(func)}, [${args.map((a) => this.rv(a)).join(', ')}]);`;
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
                const prop = this.constants[read()];
                code = `${this.wv(reg)} = $Get(${this.rv(obj)}, ${prop});`;
                break;
            }
            case OpCode.GetIndex: {
                reg = read();
                const obj = read();
                const index = read();
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
                const prop = this.constants[read()];
                code = `${this.wv(reg)} = $Has(${this.rv(obj)}, ${prop});`;
                break;
            }
            case OpCode.HasIndex: {
                reg = read();
                const obj = read();
                const index = read();
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
                const prop = this.constants[read()];
                code = `$Set(${this.rv(obj)}, ${prop}, ${this.rv(reg)});`;
                break;
            }
            case OpCode.SetIndex: {
                reg = read();
                const obj = read();
                const index = read();
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
                const c = this.constants[i];
                code = `${this.wv(reg)} = global[${c}] ?? null;`;
                break;
            }
            case OpCode.GetGlobalDyn: {
                reg = read();
                const name = read();
                code = `${this.wv(reg)} = global[${this.rv(name)}] ?? null;`;
                break;
            }
            case OpCode.GetUpvalue: {
                reg = read();
                const level = read();
                const up = read();
                code = `${this.wv(reg)} = ${this.rv(up, level)};`;
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
                const start = read();
                const end = read();
                code = `${this.wv(reg)} = $Slice(${this.rv(obj)}, ${start}, ${end});`;
                break;
            }
            case OpCode.SliceStart: {
                reg = read();
                const obj = read();
                const end = read();
                code = `${this.wv(reg)} = $Slice(${this.rv(obj)}, null, ${end});`;
                break;
            }
            case OpCode.SliceEnd: {
                reg = read();
                const obj = read();
                const start = read();
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
                code = `if ($ToBoolean(${this.rv(cond)})) {`;
                break;
            }
            case OpCode.IfNot: {
                const cond = read();
                code = `if (!$ToBoolean(${this.rv(cond)})) {`;
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
                const regs = Array.from({ length: nreg - 1 }, (_, i) => this.wv(i + 2, -1)).join(', ');
                code = `for (let ${this.wv(1, -1)} of $Iterable(${this.rv(iterable)})) { Cp(); let _, ${regs};`;
                break;
            }
            case OpCode.LoopRange:
            case OpCode.LoopRangeExclusive: {
                const nreg = read();
                const start = read();
                const end = read();
                const exclusive = opcode === OpCode.LoopRangeExclusive;
                const regs = Array.from({ length: nreg - 1 }, (_, i) => this.wv(i + 2, -1)).join(', ');
                const i = this.wv(1, -1);
                code = `for (let start = $ToNumber(${this.rv(start)}), end = $ToNumber(${this.rv(end)}), ${i} = start; ${i} ${exclusive ? '<' : '<='} end; ${i} += 1) { Cp(); let _, ${regs};`;
                break;
            }
            case OpCode.Loop: {
                const nreg = read();
                const regs = Array.from({ length: nreg }, (_, i) => this.wv(i + 1, -1)).join(', ');
                code = `while (true) { Cp(); let _, ${regs};`;
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
        this.addSourceMap();
    }
    /** 添加源映射 */
    addSourceMap(): void {
        if (!this.options.sourceMap) return;
        let fileName = (this.options.fileName ?? '').trim();
        if (fileName.startsWith('/')) {
            fileName = fileName.replace(/^\\+\s*/, '');
        }
        if (!fileName) {
            fileName = `${sourceId++}.${this.options.input_mode === 'Template' ? 'miratpl' : 'mira'}`;
        }
        const data = {
            version: 3,
            file: fileName,
            sourceRoot: ORIGIN,
            sources: [fileName],
            sourcesContent: [ArrayBuffer.isView(this.source) ? null : this.source],
            names: [],
            mappings: '',
        };
        this.codeLines.unshift(
            `//# sourceURL=${ORIGIN}/${fileName}.js`,
            `//# sourceMappingURL=data:application/json;base64,${encodeURL(JSON.stringify(data))}`,
        );
    }
}
