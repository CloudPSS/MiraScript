import type { VmAny } from './types';

/**
 * VM 预期的错误
 */
export class VmError extends Error {
    constructor(
        message: string,
        readonly recovered: VmAny,
    ) {
        super(message);
        this.name = 'VmError';
    }

    /** 从其他错误构造 */
    static from(prefix: string, error: unknown, recovered: VmAny): VmError {
        if (prefix && !prefix.endsWith(': ')) prefix += ': ';
        const vmError = new VmError(`${prefix}${error instanceof Error ? error.message : String(error)}`, recovered);
        vmError.stack = error instanceof Error ? error.stack : undefined;
        return vmError;
    }
}
