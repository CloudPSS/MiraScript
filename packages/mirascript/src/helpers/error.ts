import type { VmAny } from '../vm/types/index.js';

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
        if (prefix) {
            if (prefix.endsWith(':')) {
                prefix += ' ';
            } else if (!prefix.endsWith(': ')) {
                prefix += ': ';
            }
        }
        let vmError: VmError;
        if (error instanceof Error) {
            vmError = new VmError(`${prefix}${error.message}`, recovered);
            vmError.stack = error.stack;
        } else {
            vmError = new VmError(`${prefix}${String(error)}`, recovered);
        }
        vmError.cause = error;
        return vmError;
    }
}
