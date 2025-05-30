/**
 * VM 预期的错误
 */
export class VmError extends Error {
    constructor(message: string) {
        super(message);
        this.name = 'VmError';
    }
}
