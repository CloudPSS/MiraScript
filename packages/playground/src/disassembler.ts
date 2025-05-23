import { OpCode } from 'mira-wasm';

const ENV = (global: Record<string, unknown>) => ({
    $Mul: (a: number, b: number) => Number(a) * Number(b),
    $Add: (a: number, b: number) => Number(a) + Number(b),
    $Sub: (a: number, b: number) => Number(a) - Number(b),
    $Div: (a: number, b: number) => Number(a) / Number(b),
    $Pow: (a: number, b: number) => Number(a) ** Number(b),
    $Concat: (...args: string[]) => args.join(''),
    $Pos: Number,
    $Neg: (a: number) => -Number(a),
    $Not: (a: boolean) => !a,
    $And: (a: boolean, b: boolean) => a && b,
    $Or: (a: boolean, b: boolean) => a || b,
    $Call: (name: string, ...args: unknown[]) => {
        return (global[name] as (...args: unknown[]) => unknown)(...args) ?? null;
    },
    $CallDyn: (func: (...args: unknown[]) => unknown, ...args: unknown[]) => {
        return func(...args) ?? null;
    },
    $GetGlobal: (name: string) => global[name] ?? null,
});

/** 将值转为 JS */
function print(value: unknown): string {
    if (value === null) return 'null';
    if (value === undefined) return 'undefined';
    if (typeof value == 'object' || typeof value == 'string') {
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
    const env = ENV({
        // eslint-disable-next-line no-console
        print: console.log,
    });
    // eslint-disable-next-line @typescript-eslint/no-implied-eval, @typescript-eslint/no-unsafe-call
    const fn = new Function(...Object.keys(env), `let _; ${code}; return _;`)(...Object.values(env)) as () => unknown;
    const r = fn();
    return [
        `Chunk length: ${disassembler.chunkSize}`,
        '',
        `Constants length: ${disassembler.constSize}`,
        ...disassembler.constants.map((c, i) => `  [${i}]:\t${c}`),
        '',
        `Code length: ${disassembler.codeSize}`,
        ...disassembler.disassembly.map((d) => {
            return `  [${d.index}]:\t${OpCode[d.opcode].padEnd(8)}${d.wide ? 'W' : ''} \t${d.body}`;
        }),
        '',
        `Code:`,
        code,
        `Result: `,
        `  ${print(r)}`,
    ].join('\n');
}

/** 解析常量 */
function readConst(reader: DataView, offset: number): [Constant, number] {
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

/** 常量表中的常量 */
type Constant = number | string | boolean | null;

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
            this.constants.push(print(constant));
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

    /** 读取 if 结束 */
    private readIfEnd(): void {
        while (this.codeOffset < this.codeSize) {
            const opcode_raw = this.codeReader.getUint8(this.codeOffset);
            const opcode = opcode_raw & 0x7f;
            if (opcode !== OpCode.IfEnd) {
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
                return this.readIfEnd();
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
        return this.readIfEnd();
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
        switch (opcode) {
            case OpCode.Func: {
                const f = read();
                const argn = read();
                const regn = read();
                body = `${this.reg(f)} = fn (${Array.from({ length: argn }, (_, i) => this.reg(i + 1)).join(', ')}) { maxreg ${regn}`;
                code = `${this.wv(f)} = (${Array.from({ length: argn }, (_, i) => this.wv(i + 1, -1)).join(', ')}) => { let ${Array.from({ length: regn - argn + 1 }, (_, i) => (i ? this.wv(i + argn, -1) : this.wv(0, -1))).join(', ')};`;
                break;
            }
            case OpCode.If: {
                const cond = read();
                body = `if (${this.reg(cond)}) {`;
                code = `if (${this.rv(cond)}) {`;
                break;
            }
            case OpCode.Constant: {
                const reg = read();
                const i = read();
                const c = this.constants[i];
                body = `${this.reg(reg)} = const ${i} ; ${c}`;
                code = `${this.wv(reg)} = ${c};`;
                break;
            }
            case OpCode.Return: {
                const value = read();
                body = `return ${this.reg(value)}`;
                code = `return ${this.rv(value)};`;
                break;
            }
            case OpCode.Add:
            case OpCode.Sub:
            case OpCode.Mul:
            case OpCode.Div:
            case OpCode.Mod:
            case OpCode.Pow: {
                const ret = read();
                const left = read();
                const right = read();
                body = `${this.reg(ret)} = ${OpCode[opcode]} ${this.reg(left)} ${this.reg(right)}`;
                code = `${this.wv(ret)} = $${OpCode[opcode]}(${this.rv(left)}, ${this.rv(right)});`;
                break;
            }
            case OpCode.Concat: {
                const ret = read();
                const n = read();
                const args = Array.from({ length: n }, (_, i) => read());
                body = `${this.reg(ret)} = ${OpCode[opcode]} ${args.map((a) => this.reg(a)).join(' ')}`;
                code = `${this.wv(ret)} = $${OpCode[opcode]}(${args.map((a) => this.rv(a)).join(', ')});`;
                break;
            }
            case OpCode.Call: {
                const ret = read();
                const func = read();
                const n = read();
                const args = Array.from({ length: n }, (_, i) => read());
                const funcName = this.constants[func];
                body = `${this.reg(ret)} = ${OpCode[opcode]} ${funcName} (${args.map((a) => this.reg(a)).join(' ')})`;
                code = `${this.wv(ret)} = $${OpCode[opcode]}(${funcName}, ${args.map((a) => this.rv(a)).join(', ')});`;
                break;
            }
            case OpCode.CallDyn: {
                const ret = read();
                const func = read();
                const n = read();
                const args = Array.from({ length: n }, (_, i) => read());
                body = `${this.reg(ret)} = ${OpCode[opcode]} ${this.reg(func)} (${args.map((a) => this.reg(a)).join(' ')})`;
                code = `${this.wv(ret)} = $${OpCode[opcode]}(${this.rv(func)}, ${args.map((a) => this.rv(a)).join(', ')});`;
                break;
            }
            case OpCode.Assign: {
                const ret = read();
                const value = read();
                body = `${this.reg(ret)} = ${this.reg(value)}`;
                code = `${this.wv(ret)} = ${this.rv(value)};`;
                break;
            }
            case OpCode.Pos:
            case OpCode.Neg:
            case OpCode.Not:
            case OpCode.Type: {
                const ret = read();
                const value = read();
                body = `${this.reg(ret)} = ${OpCode[opcode]} ${this.reg(value)};`;
                code = `${this.wv(ret)} = $${OpCode[opcode]}(${this.rv(value)});`;
                break;
            }
            case OpCode.GetGlobal: {
                const reg = read();
                const i = read();
                const c = this.constants[i];
                body = `${this.reg(reg)} = ${OpCode[opcode]} ${i} ; ${c}`;
                code = `${this.wv(reg)} = $${OpCode[opcode]}(${c});`;
                break;
            }
            case OpCode.GetUpvalue: {
                const v = read();
                const level = read();
                const up = read();
                body = `${this.reg(v)} = up ${level} %${this.reg(up)}`;
                code = `${this.wv(v)} = ${this.rv(up, level)};`;
                break;
            }
            case OpCode.SetUpvalue: {
                const v = read();
                const level = read();
                const up = read();
                body = `up ${level} %${this.reg(up)} = ${this.reg(v)}`;
                code = `${this.rv(up, level)} = ${this.wv(v)};`;
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
            case OpCode.If: {
                this.readIfElse();
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
