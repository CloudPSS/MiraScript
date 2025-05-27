import { OpCode } from 'mira-wasm';

/**
 * 可外部调用的对象
 */
// eslint-disable-next-line @typescript-eslint/no-unsafe-function-type
type ExternObject = Record<string, unknown> & Function;

/** Wrapper for `extern` objects */
class Extern {
    constructor(
        readonly value: ExternObject,
        readonly caller: Extern | null = null,
    ) {}

    /** Check if the object has a property */
    protected access(key: string | number, read: boolean): boolean {
        if (typeof this.value == 'function' && (key === 'prototype' || key === 'arguments' || key === 'caller'))
            return false;
        if (Object.hasOwn(this.value, key)) return true;
        if (key === '__proto__') return false;
        if (!read) return true;
        const prop = this.value[key];
        if (key in Function.prototype && prop === Function.prototype[key as keyof (() => void)]) return false;
        if (key in Array.prototype && prop === Array.prototype[key as keyof unknown[]]) return false;
        if (key in Object.prototype && prop === Object.prototype[key as keyof object]) return false;
        return true;
    }

    /** Check if the object has a property */
    has(key: string | number): boolean {
        return this.access(key, true);
    }
    /** Get a property from the object */
    get(key: string | number): unknown {
        if (!this.has(key)) return null;
        const prop = this.value[key] ?? null;
        if (prop == null) return null;
        switch (typeof prop) {
            case 'function':
            case 'object':
                return new Extern(prop as ExternObject, this);
            case 'string':
            case 'number':
            case 'boolean':
                return prop;
            case 'bigint':
                return Number(prop);
            case 'symbol':
            case 'undefined':
            default:
                return null;
        }
    }
    /** Set a property on the object */
    set(key: string | number, value: unknown): boolean {
        if (!this.access(key, false)) return false;
        this.value[key] = value;
        return true;
    }
    /** Call extern value */
    call(...args: unknown[]): unknown {
        if (typeof this.value == 'function') {
            return this.value.apply(this.caller?.value ?? null, args);
        }
        return null;
    }
    /** Iterate over value */
    keys(): string[] {
        const keys: string[] = [];
        for (const key in this.value) {
            if (this.has(key)) keys.push(key);
        }
        return keys;
    }
    /** Convert the object to JSON */
    toJSON() {
        return String(this.value);
    }
}

const ENV = (global: Record<string, unknown>) => {
    const $Mul = (a: number, b: number) => $ToNumber(a) * $ToNumber(b);
    const $Add = (a: number, b: number) => $ToNumber(a) + $ToNumber(b);
    const $Sub = (a: number, b: number) => $ToNumber(a) - $ToNumber(b);
    const $Div = (a: number, b: number) => $ToNumber(a) / $ToNumber(b);
    const $Pow = (a: number, b: number) => $ToNumber(a) ** $ToNumber(b);
    const $Gt = (a: number, b: number) => $ToNumber(a) > $ToNumber(b);
    const $Gte = (a: number, b: number) => $ToNumber(a) >= $ToNumber(b);
    const $Lt = (a: number, b: number) => $ToNumber(a) < $ToNumber(b);
    const $Lte = (a: number, b: number) => $ToNumber(a) <= $ToNumber(b);
    const $Eq = (a: unknown, b: unknown) => a === b;
    const $Neq = (a: unknown, b: unknown) => !$Eq(a, b);
    const $Concat = (...args: string[]) => {
        return args.map($ToString).join('');
    };
    const $Pos = (a: number) => $ToNumber(a);
    const $Neg = (a: number) => -$ToNumber(a);
    const $Not = (a: boolean) => !$ToBool(a);
    const $Call = (name: string, ...args: unknown[]) => {
        return (global[name] as (...args: unknown[]) => unknown)(...args) ?? null;
    };
    const $CallDyn = (func: (...args: unknown[]) => unknown, ...args: unknown[]) => {
        if (func instanceof Extern) {
            func = func.value as unknown as (...args: unknown[]) => unknown;
        }
        if (typeof func != 'function') {
            throw new TypeError(`Expected function, got ${$Type(func)}`);
        }
        return func(...args) ?? null;
    };
    const $GetGlobal = (name: string) => $Get(global, name);
    const $Type = (value: unknown) => {
        if (value === null) return 'nil';
        if (Array.isArray(value)) return 'array';
        if (typeof value == 'object') return 'record';
        return typeof value;
    };
    const $ToBool = (value: unknown) => {
        return value != null && value !== false;
    };
    const $ToString = (value: unknown) => {
        if (value === null) return '';
        if (Array.isArray(value)) return value.map(toJavascript).join(', ');
        if (typeof value == 'object') return JSON.stringify(value);
        if (typeof value == 'function') return `<function ${value.name}>`;
        return String(value);
    };
    const $ToNumber = Number;
    const $NonNil = (value: unknown): asserts value is NonNullable<unknown> => {
        if (value === null) throw new Error('Expected non-nil value');
    };
    const $Get = (obj: unknown, key: string | number) => {
        if (obj == null || typeof obj != 'object') return null;
        if (obj instanceof Extern) {
            return obj.get(key);
        }
        if (!Object.hasOwn(obj, key)) return null;
        return (obj as Record<string, unknown>)[key] ?? null;
    };
    const $ArrayRange = (start: number, end: number): unknown[] => {
        const arr: unknown[] = [];
        for (let i = start; i <= end; i++) {
            arr.push(i);
        }
        return arr;
    };
    const $ArrayRangeExclusive = (start: number, end: number): unknown[] => {
        const arr: unknown[] = [];
        for (let i = start; i < end; i++) {
            arr.push(i);
        }
        return arr;
    };
    const $ArraySpread = (array: unknown[]): unknown[] => {
        return array;
    };
    const $RecordSpread = (record: object): object => {
        return record;
    };
    const $Iterable = (value: unknown): unknown[] => {
        if (value == null) return [];
        if (value instanceof Extern) return value.keys().map((key) => value.get(key));
        if (Array.isArray(value)) return value;
        if (typeof value == 'object') return Object.values(value);
        return [value];
    };
    let cp: number = Number.NaN;
    const $Cp = (): void => {
        if (!cp) {
            cp = Date.now();
        } else if (Date.now() - cp > 100) {
            throw new RangeError('Execution timeout');
        }
    };
    const $ClearCp = (): void => {
        cp = Number.NaN;
    };
    return {
        $Cp,
        $ClearCp,
        $Mul,
        $Add,
        $Sub,
        $Div,
        $Pow,
        $Concat,
        $Gt,
        $Gte,
        $Lt,
        $Lte,
        $Eq,
        $Neq,
        $Pos,
        $Neg,
        $Not,
        $Call,
        $CallDyn,
        $GetGlobal,
        $Get,
        $Type,
        $ToString,
        $ToBool,
        $ToNumber,
        $NonNil,
        $ArrayRange,
        $ArrayRangeExclusive,
        $ArraySpread,
        $RecordSpread,
        $Iterable,
    };
};

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
        if (value instanceof Extern) {
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
    const env = ENV({
        // eslint-disable-next-line no-console
        print: console.log,
        globalThis: new Extern(globalThis as unknown as ExternObject),
        map: (
            self: unknown[],
            ...args: [callbackfn: (value: unknown, index: number, array: unknown[]) => unknown, thisArg?: unknown]
        ) => Array.prototype.map.apply(self, args),
    });
    let r;
    try {
        // eslint-disable-next-line @typescript-eslint/no-implied-eval, @typescript-eslint/no-unsafe-call
        const fn = new Function(...Object.keys(env), `let _; ${code}; return _;`)(
            ...Object.values(env),
        ) as () => unknown;
        r = fn();
    } catch (e) {
        r = e;
    }
    return [
        `Chunk length: ${disassembler.chunkSize}`,
        '',
        `Constants length: ${disassembler.constSize}`,
        ...disassembler.constants.map((c, i) => `  [${i}]:\t${c}`),
        '',
        `Code length: ${disassembler.codeSize}`,
        ...disassembler.disassembly.map((d) => {
            return `  [${d.index}]:\t${String(OpCode[d.opcode] ?? d.opcode).padEnd(8)}${d.wide ? 'W' : ''} \t${d.body}`;
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
    private readRecord(): void {
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
                case OpCode.Field: {
                    const field = read();
                    const field_name = this.constants[field];
                    const value = read();
                    body = `Field ${field} ${this.reg(value)} ; ${field_name}`;
                    code = `${field_name}: ${this.rv(value)},`;
                    break;
                }
                case OpCode.FieldDyn: {
                    const field = read();
                    const value = read();
                    body = `FieldDyn ${field} ${this.reg(value)}`;
                    code = `[$ToString(${this.rv(field)})]: ${this.rv(value)},`;
                    break;
                }
                case OpCode.FieldIndex: {
                    const field = read();
                    const value = read();
                    body = `FieldIndex ${field} ${this.reg(value)}`;
                    code = `${field}: ${this.rv(value)},`;
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
                    code = `});`;
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
    private readArray(): void {
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
                    code = `]);`;
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
        switch (opcode) {
            case OpCode.Func: {
                const f = read();
                const argn = read();
                const regn = read();
                body = `${this.reg(f)} = fn (${Array.from({ length: argn }, (_, i) => this.reg(i + 1)).join(', ')}) { maxreg ${regn}`;
                code = `${this.wv(f)} = (${Array.from({ length: argn }, (_, i) => this.wv(i + 1, -1)).join(', ')}) => { $Cp(); let ${Array.from({ length: regn - argn + 1 }, (_, i) => (i ? this.wv(i + argn, -1) : this.wv(0, -1))).join(', ')};`;
                break;
            }
            case OpCode.Constant: {
                const reg = read();
                const i = read();
                const c = this.constants[i];
                body = `${this.reg(reg)} = Const ${i} ; ${c}`;
                code = `${this.wv(reg)} = ${c};`;
                break;
            }
            case OpCode.Uninit: {
                const reg = read();
                body = `${this.reg(reg)} = Uninit`;
                code = `${this.wv(reg)} = undefined;`;
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
            case OpCode.Pow:
            case OpCode.Gt:
            case OpCode.Geq:
            case OpCode.Lt:
            case OpCode.Leq:
            case OpCode.Eq:
            case OpCode.Neq:
            case OpCode.Aeq:
            case OpCode.Naeq: {
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
            case OpCode.Type:
            case OpCode.ToBool:
            case OpCode.ToNumber:
            case OpCode.ToString: {
                const ret = read();
                const value = read();
                body = `${this.reg(ret)} = ${OpCode[opcode]} ${this.reg(value)};`;
                code = `${this.wv(ret)} = $${OpCode[opcode]}(${this.rv(value)});`;
                break;
            }
            case OpCode.NonNil: {
                const value = read();
                body = `NonNil ${this.reg(value)}`;
                code = `$NonNil(${this.rv(value)})`;
                break;
            }
            case OpCode.Get: {
                const ret = read();
                const obj = read();
                const prop = this.constants[read()];
                body = `${this.reg(ret)} = ${OpCode[opcode]} ${this.reg(obj)} ${prop}`;
                code = `${this.wv(ret)} = $Get(${this.rv(obj)}, ${prop});`;
                break;
            }
            case OpCode.GetIndex: {
                const ret = read();
                const obj = read();
                const index = read();
                body = `${this.reg(ret)} = ${OpCode[opcode]} ${this.reg(obj)} ${index}`;
                code = `${this.wv(ret)} = $Get(${this.rv(obj)}, ${index});`;
                break;
            }
            case OpCode.GetDyn: {
                const ret = read();
                const obj = read();
                const index = read();
                body = `${this.reg(ret)} = ${OpCode[opcode]} ${this.reg(obj)} ${this.reg(index)}`;
                code = `${this.wv(ret)} = $Get(${this.rv(obj)}, ${this.rv(index)});`;
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
            case OpCode.Record: {
                const ret = read();
                body = `${this.reg(ret)} = Record`;
                code = `${this.wv(ret)} = ({`;
                break;
            }
            case OpCode.Array: {
                const ret = read();
                body = `${this.reg(ret)} = Array`;
                code = `${this.wv(ret)} = ([`;
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
                this.readRecord();
                break;
            }
            case OpCode.Array: {
                this.readArray();
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
