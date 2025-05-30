import { OpCode } from 'mira-wasm';
import * as ENV from './operations.js';
import { VmExtern } from './extern';
import type { VmPrimitive } from './types';

/** 将值转为 JS */
function toJavascript(value: unknown): string {
    if (value === null) return 'null';
    if (value === undefined) return 'undefined';
    if (typeof value == 'object' || typeof value == 'string') {
        return JSON.stringify(value);
    }
    return String(value);
}

/** 将值转为显示 */
function print(value: unknown): string {
    if (value === null) return 'null';
    if (value === undefined) return 'undefined';
    if (typeof value == 'object' || typeof value == 'string') {
        if (value instanceof VmExtern) {
            return `Extern ${print(value.value)}`;
        }
        if (value instanceof Error) {
            return String(value);
        }
        if (Array.isArray(value)) {
            return `[${value.map(print).join(', ')}]`;
        }
        return JSON.stringify(value);
    }
    return String(value);
}

/** 反编译 */
export function disassemble(chunk: Uint8Array | undefined): string {
    if (!chunk) return 'Compilation failed';
    const disassembler = new Disassembler(chunk);
    disassembler.read();
    const code = disassembler.codeLines.map((line) => `  ${line}`).join('\n');
    // eslint-disable-next-line no-console
    console.log(code);
    let r;
    try {
        // eslint-disable-next-line @typescript-eslint/no-implied-eval, @typescript-eslint/no-unsafe-call
        const fn = new Function(...Object.keys(ENV), 'global', `let _; ${code}; return _;`)(...Object.values(ENV), {
            if: 12,
            // eslint-disable-next-line no-console
            print: console.log,
            globalThis: new VmExtern(globalThis),
            map: (
                self: unknown[],
                ...args: [callbackfn: (value: unknown, index: number, array: unknown[]) => unknown, thisArg?: unknown]
            ) => Array.prototype.map.apply(self, args),
        }) as () => unknown;
        ENV.$ClearCp();
        r = fn();
    } catch (e) {
        r = e;
    }
    return [
        `Chunk length: ${disassembler.chunkSize}`,
        '',
        `Constants length: ${disassembler.constSize}`,
        ...disassembler.constants.map(
            (c, i) => `  [${i.toString().padStart(disassembler.constSize.toString().length)}]:\t${c}`,
        ),
        '',
        `Code length: ${disassembler.codeSize}`,
        ...disassembler.disassembly.map((d) => {
            return `  [${d.index.toString().padStart(disassembler.codeSize.toString().length)}]: ${String(OpCode[d.opcode] ?? d.opcode).padEnd(12)}${d.wide ? 'W' : ''}  ${d.body}`;
        }),
        '',
        `Code:`,
        code,
        `Result: `,
        `  ${print(r)}`,
    ].join('\n');
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

/** 反编译/代码生成 */
class Disassembler {
    constructor(readonly chunk: Uint8Array) {
        const reader = new DataView(chunk.buffer, chunk.byteOffset, chunk.byteLength);
        this.chunkSize = reader.getUint32(0, true);
        this.codeSize = reader.getUint32(4, true);
        this.constSize = reader.getUint32(8 + this.codeSize, true);

        this.codeReader = new DataView(chunk.buffer, chunk.byteOffset + 8, this.codeSize);
        this.constReader = new DataView(chunk.buffer, chunk.byteOffset + 12 + this.codeSize, this.constSize);
    }
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
        return '  '.repeat(this.identCounter + len);
    }

    readonly codeLines: string[] = [];
    readonly disassembly: Array<{ index: number; opcode: OpCode; wide: boolean; body: string }> = [];

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
    /** Register */
    private reg(i: number): string {
        if (!i) return '%_';
        return `%${i}`;
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
            const index = this.codeOffset;
            this.codeOffset++;
            const body = this.ident(-1) + `};`;
            this.disassembly.push({
                index,
                opcode,
                wide: opcode_raw >= 0x80,
                body,
            });
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
            const index = this.codeOffset;
            this.codeOffset++;
            this.identCounter--;
            const body = this.ident() + `};`;
            this.disassembly.push({
                index,
                opcode,
                wide: opcode_raw >= 0x80,
                body,
            });
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
            const index = this.codeOffset;
            this.codeOffset++;
            const body = this.ident(-1) + `} else {`;
            this.disassembly.push({
                index,
                opcode,
                wide: opcode_raw >= 0x80,
                body,
            });
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
            const index = this.codeOffset;
            const opcode_raw = this.codeReader.getUint8(this.codeOffset++);
            const opcode = opcode_raw & 0x7f;
            const wide = opcode_raw >= 0x80;
            const read = () => this.readParam(wide);
            let body = '';
            let code = '';
            switch (opcode) {
                case OpCode.FieldOpt:
                case OpCode.Field: {
                    const field = read();
                    const field_name = this.constants[field];
                    if (!field_name) throw new Error(`Unknown field ${field}`);
                    const value = read();
                    body = `${OpCode[opcode]} ${field} ${this.reg(value)} ; ${field_name}`;
                    // Use computed property names to avoid prototype pollution
                    code = `[${field_name}]: ${this.rv(value)},`;
                    if (opcode === OpCode.FieldOpt) {
                        optional.push(field_name);
                    }
                    break;
                }
                case OpCode.FieldOptDyn:
                case OpCode.FieldDyn: {
                    const field = read();
                    const value = read();
                    body = `${OpCode[opcode]} ${field} ${this.reg(value)}`;
                    code = `[$ToString(${this.rv(field)})]: ${this.rv(value)},`;
                    if (opcode === OpCode.FieldOptDyn) {
                        optional.push(this.rv(field));
                    }
                    break;
                }
                case OpCode.FieldOptIndex:
                case OpCode.FieldIndex: {
                    const field = read();
                    const value = read();
                    body = `${OpCode[opcode]} ${field} ${this.reg(value)}`;
                    code = `[${field}]: ${this.rv(value)},`;
                    if (opcode === OpCode.FieldOptIndex) {
                        optional.push(this.rv(field));
                    }
                    break;
                }
                case OpCode.Spread: {
                    const value = read();
                    body = `Spread ${this.reg(value)}`;
                    code = `...$RecordSpread(${this.rv(value)}),`;
                    break;
                }
                case OpCode.Freeze: {
                    this.identCounter--;
                    body = `Freeze`;
                    code = `}); $RecordFreeze(${this.rv(obj)}, [${optional.join(', ')}])`;
                    break;
                }
                default: {
                    body = `?${OpCode[opcode] ?? opcode}`;
                    code = `;`;
                    break;
                }
            }
            const ident = this.ident();
            this.codeLines.push(ident + code);
            this.disassembly.push({
                index,
                opcode,
                wide,
                body: ident + body,
            });
            if (opcode === OpCode.Freeze) {
                return;
            }
        }
    }

    /** 读取 array */
    private readArray(arr: number): void {
        this.identCounter++;
        while (this.codeOffset < this.codeSize) {
            const index = this.codeOffset;
            const opcode_raw = this.codeReader.getUint8(this.codeOffset++);
            const opcode = opcode_raw & 0x7f;
            const wide = opcode_raw >= 0x80;
            const read = () => this.readParam(wide);
            let body = '';
            let code = '';
            switch (opcode) {
                case OpCode.Item: {
                    const value = read();
                    body = `Item ${this.reg(value)}`;
                    code = `${this.rv(value)},`;
                    break;
                }
                case OpCode.ItemRange: {
                    const start = read();
                    const end = read();
                    body = `ItemRange ${start} ${end}`;
                    code = `...$ArrayRange(${start}, ${end}),`;
                    break;
                }
                case OpCode.ItemRangeDyn: {
                    const start = read();
                    const end = read();
                    body = `ItemRangeDyn ${this.reg(start)} ${this.reg(end)}`;
                    code = `...$ArrayRange(${this.rv(start)}, ${this.rv(end)}),`;
                    break;
                }
                case OpCode.ItemRangeExclusiveDyn: {
                    const start = read();
                    const end = read();
                    body = `ItemRangeExclusiveDyn ${this.reg(start)} ${this.reg(end)}`;
                    code = `...$ArrayRangeExclusive(${this.rv(start)}, ${this.rv(end)}),`;
                    break;
                }
                case OpCode.Spread: {
                    const value = read();
                    body = `Spread ${this.reg(value)}`;
                    code = `...$ArraySpread(${this.rv(value)}),`;
                    break;
                }
                case OpCode.Freeze: {
                    this.identCounter--;
                    body = `Freeze`;
                    code = `]); $ArrayFreeze(${this.rv(arr)})`;
                    break;
                }
                default: {
                    body = `?${OpCode[opcode] ?? opcode}`;
                    code = `;`;
                    break;
                }
            }
            const ident = this.ident();
            this.codeLines.push(ident + code);
            this.disassembly.push({
                index,
                opcode,
                wide,
                body: ident + body,
            });
            if (opcode === OpCode.Freeze) {
                return;
            }
        }
    }

    /** 读取代码 */
    private readCode(): void {
        const index = this.codeOffset;
        const opcode_raw = this.codeReader.getUint8(this.codeOffset++);
        const opcode = opcode_raw & 0x7f;
        const wide = opcode_raw >= 0x80;
        const read = () => this.readParam(wide);
        const ident = this.ident();
        let body = '';
        let code = '';
        let reg = 0;
        switch (opcode) {
            case OpCode.Func: {
                reg = read();
                const argn = read();
                const regn = read();
                body = `${this.reg(reg)} = fn (${Array.from({ length: argn }, (_, i) => this.reg(i + 1)).join(', ')}) { maxreg ${regn}`;
                code = `${this.wv(reg)} = (${Array.from({ length: argn }, (_, i) => this.wv(i + 1, -1)).join(', ')}) => { $Cp(); let ${Array.from({ length: regn - argn + 1 }, (_, i) => (i ? this.wv(i + argn, -1) : this.wv(0, -1))).join(', ')};`;
                break;
            }
            case OpCode.Constant: {
                reg = read();
                const i = read();
                const c = this.constants[i];
                body = `${this.reg(reg)} = Const ${i} ; ${c}`;
                code = `${this.wv(reg)} = ${c};`;
                break;
            }
            case OpCode.Uninit: {
                reg = read();
                body = `${this.reg(reg)} = Uninit`;
                code = `${this.wv(reg)} = undefined;`;
                break;
            }
            case OpCode.Return: {
                reg = read();
                body = `return ${this.reg(reg)}`;
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
                body = `${this.reg(reg)} = ${OpCode[opcode]} ${this.reg(left)} ${this.reg(right)}`;
                code = `${this.wv(reg)} = $${OpCode[opcode]}(${this.rv(left)}, ${this.rv(right)});`;
                break;
            }
            case OpCode.Concat: {
                reg = read();
                const n = read();
                const args = Array.from({ length: n }, (_, i) => read());
                body = `${this.reg(reg)} = ${OpCode[opcode]} ${args.map((a) => this.reg(a)).join(' ')}`;
                code = `${this.wv(reg)} = $${OpCode[opcode]}(${args.map((a) => this.rv(a)).join(', ')});`;
                break;
            }
            case OpCode.Call: {
                reg = read();
                const func = read();
                const n = read();
                const args = Array.from({ length: n }, (_, i) => read());
                const funcName = this.constants[func];
                body = `${this.reg(reg)} = ${OpCode[opcode]} ${funcName} (${args.map((a) => this.reg(a)).join(' ')})`;
                code = `${this.wv(reg)} = $CallDyn($Get(global, ${funcName}), ${args.map((a) => this.rv(a)).join(', ')});`;
                break;
            }
            case OpCode.CallDyn: {
                reg = read();
                const func = read();
                const n = read();
                const args = Array.from({ length: n }, (_, i) => read());
                body = `${this.reg(reg)} = ${OpCode[opcode]} ${this.reg(func)} (${args.map((a) => this.reg(a)).join(' ')})`;
                code = `${this.wv(reg)} = $CallDyn(${this.rv(func)}, ${args.map((a) => this.rv(a)).join(', ')});`;
                break;
            }
            case OpCode.Assign: {
                reg = read();
                const value = read();
                body = `${this.reg(reg)} = ${this.reg(value)}`;
                code = `${this.wv(reg)} = ${this.rv(value)};`;
                break;
            }
            case OpCode.Pos:
            case OpCode.Neg:
            case OpCode.Not:
            case OpCode.Type:
            case OpCode.ToBool:
            case OpCode.ToNumber:
            case OpCode.ToString: {
                reg = read();
                const value = read();
                body = `${this.reg(reg)} = ${OpCode[opcode]} ${this.reg(value)};`;
                code = `${this.wv(reg)} = $${OpCode[opcode]}(${this.rv(value)});`;
                break;
            }
            case OpCode.NonNil: {
                reg = read();
                body = `NonNil ${this.reg(reg)}`;
                code = `$NonNil(${this.rv(reg)})`;
                break;
            }
            case OpCode.Get: {
                reg = read();
                const obj = read();
                const prop = this.constants[read()];
                body = `${this.reg(reg)} = ${OpCode[opcode]} ${this.reg(obj)} ${prop}`;
                code = `${this.wv(reg)} = $Get(${this.rv(obj)}, ${prop});`;
                break;
            }
            case OpCode.GetIndex: {
                reg = read();
                const obj = read();
                const index = read();
                body = `${this.reg(reg)} = ${OpCode[opcode]} ${this.reg(obj)} ${index}`;
                code = `${this.wv(reg)} = $Get(${this.rv(obj)}, ${index});`;
                break;
            }
            case OpCode.GetDyn: {
                reg = read();
                const obj = read();
                const index = read();
                body = `${this.reg(reg)} = ${OpCode[opcode]} ${this.reg(obj)} ${this.reg(index)}`;
                code = `${this.wv(reg)} = $Get(${this.rv(obj)}, ${this.rv(index)});`;
                break;
            }
            case OpCode.GetGlobal: {
                reg = read();
                const i = read();
                const c = this.constants[i];
                body = `${this.reg(reg)} = ${OpCode[opcode]} ${i} ; ${c}`;
                code = `${this.wv(reg)} = $Get(global, ${c});`;
                break;
            }
            case OpCode.GetGlobalDyn: {
                reg = read();
                const name = read();
                body = `${this.reg(reg)} = ${OpCode[opcode]} ${this.reg(name)}`;
                code = `${this.wv(reg)} = $Get(global, ${this.rv(name)});`;
                break;
            }
            case OpCode.GetUpvalue: {
                reg = read();
                const level = read();
                const up = read();
                body = `${this.reg(reg)} = up ${level} %${this.reg(up)}`;
                code = `${this.wv(reg)} = ${this.rv(up, level)};`;
                break;
            }
            case OpCode.SetUpvalue: {
                reg = read();
                const level = read();
                const up = read();
                body = `up ${level} %${this.reg(up)} = ${this.reg(reg)}`;
                code = `${this.rv(up, level)} = ${this.wv(reg)};`;
                break;
            }
            case OpCode.Record: {
                reg = read();
                body = `${this.reg(reg)} = Record`;
                code = `${this.wv(reg)} = ({`;
                break;
            }
            case OpCode.Array: {
                reg = read();
                body = `${this.reg(reg)} = Array`;
                code = `${this.wv(reg)} = ([`;
                break;
            }
            case OpCode.If: {
                const cond = read();
                body = `If ${this.reg(cond)}`;
                code = `if ($ToBool(${this.rv(cond)})) {`;
                break;
            }
            case OpCode.IfNot: {
                const cond = read();
                body = `IfNot ${this.reg(cond)}`;
                code = `if (!$ToBool(${this.rv(cond)})) {`;
                break;
            }
            case OpCode.IfInit: {
                const cond = read();
                body = `IfInit ${this.reg(cond)}`;
                code = `if (${this.rv(cond)} !== undefined) {`;
                break;
            }
            case OpCode.IfNotInit: {
                const cond = read();
                body = `IfNotInit ${this.reg(cond)}`;
                code = `if (${this.rv(cond)} === undefined) {`;
                break;
            }
            case OpCode.IfNil: {
                const cond = read();
                body = `IfNil ${this.reg(cond)}`;
                code = `if (${this.rv(cond)} === null) {`;
                break;
            }
            case OpCode.IfNotNil: {
                const cond = read();
                body = `IfNotNil ${this.reg(cond)}`;
                code = `if (${this.rv(cond)} !== null) {`;
                break;
            }
            case OpCode.LoopFor: {
                const iterator = read();
                const iterable = read();
                body = `LoopFor ${this.reg(iterator)} ${this.reg(iterable)}`;
                code = `for (${this.rv(iterator)} of $Iterable(${this.rv(iterable)})) { $Cp();`;
                break;
            }
            case OpCode.Loop: {
                body = `Loop`;
                code = `while (true) { $Cp();`;
                break;
            }
            case OpCode.Break: {
                body = `Break`;
                code = `break;`;
                break;
            }
            case OpCode.Continue: {
                body = `Continue`;
                code = `continue;`;
                break;
            }
            default: {
                body = `?${OpCode[opcode] ?? opcode}`;
                code = `;`;
                break;
            }
        }
        this.codeLines.push(ident + code);
        this.disassembly.push({
            index,
            opcode,
            wide,
            body: ident + body,
        });
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
