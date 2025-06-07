import { OpCode } from 'mira-wasm';
import type { VmConst, VmPrimitive } from '../vm';
import type { GenerateOptions } from './options';

/** 生成代码 */
export function generate(chunk: Uint8Array, options: GenerateOptions): string {
    const gen = new CodeGenerator(chunk, options);
    gen.read();
    const code = options.pretty ? gen.codeLines.join('\n') : gen.codeLines.join('');
    return code;
}

/** 解析常量 */
function readConst(reader: DataView, offset: number): [VmPrimitive, number] {
    const type = reader.getUint8(offset);
    switch (type) {
        case 0:
            return [null, 1];
        case 1:
            return [true, 1];
        case 2:
            return [false, 1];
        case 3: {
            const num = reader.getFloat64(offset + 1, true);
            return [num, 9];
        }
        case 4: {
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

/** 代码生成 */
class CodeGenerator {
    constructor(
        readonly chunk: Uint8Array,
        readonly options: GenerateOptions,
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
            if (opcode !== OpCode.Else) {
                this.readCode();
                continue;
            }
            this.codeOffset++;
            const body = this.ident(-1) + `} else {`;
            this.codeLines.push(body);
            break;
        }
        return this.readBlockEnd(OpCode.IfEnd);
    }

    /** 读取 record */
    private readRecord(obj: number): void {
        this.identCounter++;
        const optional: string[] = [];
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
                    // Use computed property names to avoid prototype pollution
                    code = `[${field_name}]: Element(${this.rv(value)}),`;
                    if (opcode === OpCode.FieldOpt) {
                        optional.push(field_name);
                    }
                    break;
                }
                case OpCode.FieldOptDyn:
                case OpCode.FieldDyn: {
                    const field = read();
                    const value = read();
                    code = `[${this.rv(field)}]: Element(${this.rv(value)}),`;
                    if (opcode === OpCode.FieldOptDyn) {
                        optional.push(this.rv(field));
                    }
                    break;
                }
                case OpCode.FieldOptIndex:
                case OpCode.FieldIndex: {
                    const field = read();
                    const value = read();
                    code = `[${field}]: Element(${this.rv(value)}),`;
                    if (opcode === OpCode.FieldOptIndex) {
                        optional.push(this.rv(field));
                    }
                    break;
                }
                case OpCode.Spread: {
                    const value = read();
                    code = `...$RecordSpread(${this.rv(value)}),`;
                    break;
                }
                case OpCode.Freeze: {
                    this.identCounter--;
                    code = optional.length ? `}); RecordFreeze(${this.rv(obj)}, [${optional.join(', ')}]);` : `});`;
                    break;
                }
                default: {
                    code = `;// ?${OpCode[opcode] ?? opcode}`;
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
                    code = `;// ?${OpCode[opcode] ?? opcode}`;
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
            case OpCode.Func: {
                const script = this.codeOffset === 1;
                reg = read();
                const argn = read();
                const regn = read();
                const args = Array.from({ length: argn }, (_, i) => `${this.wv(i + 1, -1)} = null`).join(', ');
                const regs = Array.from({ length: regn - argn + 1 }, (_, i) =>
                    i ? this.wv(i + argn, -1) : this.wv(0, -1),
                ).join(', ');
                if (script) {
                    code = `return ((global = GlobalFallback(), ${args}) => { try{ CpEnter(); let ${regs};`;
                } else {
                    code = `${this.wv(reg)} = Function((${args}) => { try{ CpEnter(); let ${regs};`;
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
            case OpCode.Geq:
            case OpCode.Lt:
            case OpCode.Leq:
            case OpCode.Eq:
            case OpCode.Neq:
            case OpCode.Aeq:
            case OpCode.Naeq:
            case OpCode.In: {
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
            case OpCode.Call: {
                reg = read();
                const func = read();
                const n = read();
                const args = Array.from({ length: n }, (_, i) => read());
                const funcName = this.constants[func];
                code = `${this.wv(reg)} = $CallDyn(global[${funcName}], [${args.map((a) => this.rv(a)).join(', ')}]);`;
                break;
            }
            case OpCode.CallDyn: {
                reg = read();
                const func = read();
                const n = read();
                const args = Array.from({ length: n }, (_, i) => read());
                code = `${this.wv(reg)} = $CallDyn(${this.rv(func)}, [${args.map((a) => this.rv(a)).join(', ')}]);`;
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
            case OpCode.ToString: {
                reg = read();
                const value = read();
                code = `${this.wv(reg)} = $${OpCode[opcode]}(${this.rv(value)});`;
                break;
            }
            case OpCode.NonNil: {
                reg = read();
                code = `$NonNil(${this.rv(reg)})`;
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
                code = `for (let ${this.rv(1, -1)} of $Iterable(${this.rv(iterable)})) { Cp(); let _, ${regs};`;
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
                code = `;// ?${OpCode[opcode] ?? opcode}`;
                break;
            }
        }
        this.codeLines.push(ident + code);
        switch (opcode) {
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
            case OpCode.LoopFor: {
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
    }
}
