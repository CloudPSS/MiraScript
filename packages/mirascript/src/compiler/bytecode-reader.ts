import type { OpCode } from '@mirascript/constants';
import type { VmPrimitive } from '../vm/index.js';
import { readConsts } from './emit/consts.js';

/** MiraScript 字节码读取器。 */
export class BytecodeReader {
    constructor(readonly chunk: Uint8Array) {
        const reader = new DataView(chunk.buffer, chunk.byteOffset, chunk.byteLength);
        this.chunkSize = reader.getUint32(0, true);
        this.codeSize = reader.getUint32(4, true);
        this.constSize = reader.getUint32(8 + this.codeSize, true);
        this.codeReader = new DataView(chunk.buffer, chunk.byteOffset + 8, this.codeSize);
        this.constVals = readConsts(
            new Uint8Array(chunk.buffer, chunk.byteOffset + 12 + this.codeSize, this.constSize),
        );
    }

    readonly chunkSize: number;
    readonly codeSize: number;
    readonly constSize: number;
    readonly constVals: VmPrimitive[];
    protected readonly codeReader: DataView;
    protected codeOffset = 0;

    /** 是否仍有指令可读。 */
    protected get hasCode(): boolean {
        return this.codeOffset < this.codeSize;
    }

    /** 读取操作码。 */
    protected readOpcode(): { opcode: OpCode; wide: boolean; offset: number } {
        const offset = this.codeOffset;
        const raw = this.codeReader.getUint8(this.codeOffset++);
        return { opcode: raw & 0x7f, wide: raw >= 0x80, offset };
    }

    /** 读取无符号参数。 */
    protected readParam(wide: boolean): number {
        const value = wide
            ? this.codeReader.getUint32(this.codeOffset, true)
            : this.codeReader.getUint8(this.codeOffset);
        this.codeOffset += wide ? 4 : 1;
        return value;
    }

    /** 读取有符号索引。 */
    protected readIndex(wide: boolean): number {
        const value = wide ? this.codeReader.getInt32(this.codeOffset, true) : this.codeReader.getInt8(this.codeOffset);
        this.codeOffset += wide ? 4 : 1;
        return value;
    }
}
