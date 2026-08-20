import { OpCode } from '@mirascript/constants';
import type { ScriptInput } from '@mirascript/constants';
import { BytecodeReader } from './bytecode-reader.js';
import type { IRange } from './diagnostic.js';
import { serialize } from '../helpers/serialize.js';

/** IL 操作数类型。 */
type OperandKind = 'register' | 'constant' | 'index' | 'unsigned';

const FIXED_OPERANDS: Partial<Record<OpCode, readonly OperandKind[]>> = {
    [OpCode.Noop]: [],
    [OpCode.Add]: ['register', 'register', 'register'],
    [OpCode.Sub]: ['register', 'register', 'register'],
    [OpCode.Mul]: ['register', 'register', 'register'],
    [OpCode.Div]: ['register', 'register', 'register'],
    [OpCode.Mod]: ['register', 'register', 'register'],
    [OpCode.Pow]: ['register', 'register', 'register'],
    [OpCode.Pos]: ['register', 'register'],
    [OpCode.Neg]: ['register', 'register'],
    [OpCode.Not]: ['register', 'register'],
    [OpCode.Plus]: ['register', 'register'],
    [OpCode.Eq]: ['register', 'register', 'register'],
    [OpCode.Neq]: ['register', 'register', 'register'],
    [OpCode.Lt]: ['register', 'register', 'register'],
    [OpCode.Lte]: ['register', 'register', 'register'],
    [OpCode.Gt]: ['register', 'register', 'register'],
    [OpCode.Gte]: ['register', 'register', 'register'],
    [OpCode.Aeq]: ['register', 'register', 'register'],
    [OpCode.Naeq]: ['register', 'register', 'register'],
    [OpCode.Same]: ['register', 'register', 'register'],
    [OpCode.Nsame]: ['register', 'register', 'register'],
    [OpCode.In]: ['register', 'register', 'register'],
    [OpCode.InGlobal]: ['register', 'register'],
    [OpCode.Format]: ['register', 'register', 'index'],
    [OpCode.And]: ['register', 'register', 'register'],
    [OpCode.Or]: ['register', 'register', 'register'],
    [OpCode.AssertInit]: ['register'],
    [OpCode.AssertNonNil]: ['register'],
    [OpCode.Type]: ['register', 'register'],
    [OpCode.ToBoolean]: ['register', 'register'],
    [OpCode.ToNumber]: ['register', 'register'],
    [OpCode.ToString]: ['register', 'register'],
    [OpCode.IsBoolean]: ['register', 'register'],
    [OpCode.IsNumber]: ['register', 'register'],
    [OpCode.IsString]: ['register', 'register'],
    [OpCode.IsRecord]: ['register', 'register'],
    [OpCode.IsArray]: ['register', 'register'],
    [OpCode.Constant]: ['register', 'constant'],
    [OpCode.Uninit]: ['register'],
    [OpCode.Assign]: ['register', 'register'],
    [OpCode.Swap]: ['register', 'register'],
    [OpCode.GetUpvalue]: ['register', 'unsigned', 'register'],
    [OpCode.SetUpvalue]: ['register', 'unsigned', 'register'],
    [OpCode.GetGlobal]: ['register', 'constant'],
    [OpCode.GetGlobalDyn]: ['register', 'register'],
    [OpCode.Record]: ['register'],
    [OpCode.Module]: ['register', 'constant'],
    [OpCode.Field]: ['constant', 'register'],
    [OpCode.FieldDyn]: ['register', 'register'],
    [OpCode.FieldIndex]: ['index', 'register'],
    [OpCode.FieldOpt]: ['constant', 'register'],
    [OpCode.FieldOptDyn]: ['register', 'register'],
    [OpCode.FieldOptIndex]: ['index', 'register'],
    [OpCode.Array]: ['register'],
    [OpCode.Item]: ['register'],
    [OpCode.ItemRange]: ['index', 'index'],
    [OpCode.ItemRangeDyn]: ['register', 'register'],
    [OpCode.ItemRangeExclusiveDyn]: ['register', 'register'],
    [OpCode.Spread]: ['register'],
    [OpCode.Freeze]: [],
    [OpCode.Has]: ['register', 'register', 'constant'],
    [OpCode.HasDyn]: ['register', 'register', 'register'],
    [OpCode.HasIndex]: ['register', 'register', 'index'],
    [OpCode.Get]: ['register', 'register', 'constant'],
    [OpCode.GetDyn]: ['register', 'register', 'register'],
    [OpCode.GetIndex]: ['register', 'register', 'index'],
    [OpCode.Set]: ['register', 'register', 'constant'],
    [OpCode.SetDyn]: ['register', 'register', 'register'],
    [OpCode.SetIndex]: ['register', 'register', 'index'],
    [OpCode.Slice]: ['register', 'register', 'index', 'index'],
    [OpCode.SliceStart]: ['register', 'register', 'index'],
    [OpCode.SliceEnd]: ['register', 'register', 'index'],
    [OpCode.SliceDyn]: ['register', 'register', 'register', 'register'],
    [OpCode.SliceExclusiveDyn]: ['register', 'register', 'register', 'register'],
    [OpCode.Length]: ['register', 'register'],
    [OpCode.Loop]: ['unsigned'],
    [OpCode.LoopFor]: ['unsigned', 'register'],
    [OpCode.LoopRange]: ['unsigned', 'register', 'register'],
    [OpCode.LoopRangeExclusive]: ['unsigned', 'register', 'register'],
    [OpCode.LoopEnd]: [],
    [OpCode.Break]: [],
    [OpCode.Continue]: [],
    [OpCode.If]: ['register'],
    [OpCode.IfNot]: ['register'],
    [OpCode.IfInit]: ['register'],
    [OpCode.IfNotInit]: ['register'],
    [OpCode.IfNil]: ['register'],
    [OpCode.IfNotNil]: ['register'],
    [OpCode.Else]: [],
    [OpCode.IfEnd]: [],
    [OpCode.Func]: ['register', 'unsigned', 'unsigned'],
    [OpCode.FuncVarg]: ['register', 'unsigned', 'unsigned'],
    [OpCode.FuncEnd]: [],
    [OpCode.Return]: ['register'],
};

/** 输出指令前结束一层缩进的操作码。 */
const CLOSE_BEFORE = new Set<OpCode>([OpCode.Freeze, OpCode.LoopEnd, OpCode.Else, OpCode.IfEnd, OpCode.FuncEnd]);
/** 输出指令后增加一层缩进的操作码。 */
const OPEN_AFTER = new Set<OpCode>([
    OpCode.Record,
    OpCode.Module,
    OpCode.Array,
    OpCode.Loop,
    OpCode.LoopFor,
    OpCode.LoopRange,
    OpCode.LoopRangeExclusive,
    OpCode.If,
    OpCode.IfNot,
    OpCode.IfInit,
    OpCode.IfNotInit,
    OpCode.IfNil,
    OpCode.IfNotNil,
    OpCode.Else,
    OpCode.Func,
    OpCode.FuncVarg,
]);
/** 行内注释分号允许出现的最远列。 */
const MAX_COMMENT_COLUMN = 80;

/** 将操作码名称转换为 IL 风格。 */
function opcodeName(opcode: OpCode): string {
    const name = OpCode[opcode];
    if (typeof name !== 'string') return `UNKNOWN_${opcode}`;
    return name.replaceAll(/(?<!^)([A-Z])/g, '_$1').toUpperCase();
}

/** IL 指令对应的 MiraScript 源码映射。 */
export interface ILSourceMap {
    /** 原始 MiraScript 源码。 */
    readonly source: ScriptInput;
    /** 与字节码指令顺序一一对应的源码范围。 */
    readonly ranges: readonly IRange[];
}

/** 读取源码映射所在的原始代码行。 */
function sourceLine(lines: readonly string[], range: IRange | undefined): string {
    if (!range || range.startLineNumber < 1) return '';
    return lines[range.startLineNumber - 1]?.trim() ?? '';
}

/** MiraScript IL 生成器。 */
class ILEmitter extends BytecodeReader {
    /** 当前结构缩进。 */
    private indent = 0;
    /** 当前字节码指令对应的源码映射序号。 */
    private instructionIndex = 0;
    /** 已生成的 IL 行。 */
    private readonly lines: string[] = [];
    /** 已输出注释的源码行号。 */
    private readonly annotatedSourceLines = new Set<number>();
    /** 原始源码行。 */
    private readonly sourceLines: readonly string[];

    constructor(
        chunk: Uint8Array,
        private readonly sourceMap?: ILSourceMap,
    ) {
        super(chunk);
        const source = sourceMap?.source;
        const sourceText = typeof source === 'string' ? source : source ? new TextDecoder().decode(source) : '';
        this.sourceLines = sourceText.split(/\r?\n/);
    }

    /** 读取并格式化一个操作数。 */
    private readOperand(kind: OperandKind, wide: boolean): string {
        switch (kind) {
            case 'register':
                return `%${this.readParam(wide)}`;
            case 'constant':
                return `#${this.readIndex(wide)}`;
            case 'index':
                return String(this.readIndex(wide));
            case 'unsigned':
                return String(this.readParam(wide));
        }
    }

    /** 读取并格式化一条指令的操作数。 */
    private readOperands(opcode: OpCode, wide: boolean): string[] {
        if (opcode === OpCode.Concat) {
            const result = this.readOperand('register', wide);
            const count = this.readParam(wide);
            return [result, String(count), ...Array.from({ length: count }, () => this.readOperand('register', wide))];
        }
        if (opcode === OpCode.Pick || opcode === OpCode.Omit) {
            const result = this.readOperand('register', wide);
            const value = this.readOperand('register', wide);
            const count = this.readParam(wide);
            return [
                result,
                value,
                String(count),
                ...Array.from({ length: count }, () => this.readOperand('constant', wide)),
            ];
        }
        if (opcode === OpCode.Call || opcode === OpCode.CallDyn) {
            const result = this.readOperand('register', wide);
            const callable = this.readOperand(opcode === OpCode.Call ? 'constant' : 'register', wide);
            const argumentCount = this.readParam(wide);
            const arguments_ = Array.from({ length: argumentCount }, () => this.readOperand('register', wide));
            const spreadCount = this.readParam(wide);
            const spreads = Array.from({ length: spreadCount }, () => this.readOperand('unsigned', wide));
            return [result, callable, String(argumentCount), ...arguments_, String(spreadCount), ...spreads];
        }
        const kinds = FIXED_OPERANDS[opcode];
        if (!kinds) throw new Error(`Unknown opcode: ${opcode}`);
        return kinds.map((kind) => this.readOperand(kind, wide));
    }

    /** 首次遇到源码行时读取其内容。 */
    private readSourceLine(code: OpCode, range: IRange | undefined): string {
        const lineNumber = range?.startLineNumber;
        if (!lineNumber || this.annotatedSourceLines.has(lineNumber)) return '';
        if (code === OpCode.Constant) return '';
        this.annotatedSourceLines.add(lineNumber);
        return sourceLine(this.sourceLines, range);
    }

    /** 生成完整 IL。 */
    emit(): string {
        this.lines.push('.constants');
        for (let i = 0; i < this.constVals.length; i++) {
            this.lines.push(`  #${i} = ${serialize(this.constVals[i])}`);
        }
        this.lines.push('', '.code');
        const instructions: Array<{ text: string; comment: string }> = [];
        while (this.hasCode) {
            const { opcode, wide, offset } = this.readOpcode();
            if (CLOSE_BEFORE.has(opcode)) this.indent = Math.max(0, this.indent - 1);
            const name = `${opcodeName(opcode)}${wide ? '.WIDE' : ''}`;
            const operands = this.readOperands(opcode, wide);
            const offsetText = offset.toString(16).padStart(8, '0');
            const operandText = operands.length ? ` ${operands.join(', ')}` : '';
            let range: IRange | undefined;
            if (opcode === OpCode.Noop) {
                // Noop 指令不对应任何源码行，因此不增加 instructionIndex。
            } else {
                const currentIndex = this.instructionIndex;
                if (currentIndex !== 0) {
                    // 不为第一条指令映射源码
                    range = this.sourceMap?.ranges[currentIndex];
                }
                this.instructionIndex++;
            }
            const text = `${offsetText}  ${'  '.repeat(this.indent)}${name}${operandText}`;
            instructions.push({ text, comment: this.readSourceLine(opcode, range) });
            if (OPEN_AFTER.has(opcode)) this.indent++;
        }
        const inlineCommentLengths = instructions
            .filter(({ text, comment }) => Boolean(comment) && text.length < MAX_COMMENT_COLUMN)
            .map(({ text }) => text.length);
        const commentColumn = inlineCommentLengths.length ? Math.max(...inlineCommentLengths) + 1 : 0;
        this.lines.push(
            ...instructions.map(({ text, comment }) => {
                if (!comment) return text;
                if (text.length >= MAX_COMMENT_COLUMN) {
                    return `${text}\n${''.padEnd(commentColumn)}; ${comment}`;
                }
                return `${text.padEnd(commentColumn)}; ${comment}`;
            }),
        );
        return this.lines.join('\n');
    }
}

/** 将 MiraScript 字节码转换为可读 IL。 */
export function emitIL(chunk: Uint8Array, sourceMap?: ILSourceMap): string {
    return new ILEmitter(chunk, sourceMap).emit();
}
