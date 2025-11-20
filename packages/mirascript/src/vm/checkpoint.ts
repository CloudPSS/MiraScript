const MAX_DEPTH = 128;

let cpDepth = 0;
let cp = Number.NaN;
let cpTimeout = 100; // Default timeout in milliseconds
/** 检查点 */
export function Cp(): void {
    if (!cp) {
        cp = Date.now();
    } else if (Date.now() - cp > cpTimeout) {
        throw new RangeError('Execution timeout');
    }
}
/** 检查点 */
export function CpEnter(): void {
    cpDepth++;
    if (cpDepth <= 1) {
        cp = Date.now();
        cpDepth = 1;
    } else if (cpDepth > MAX_DEPTH) {
        throw new RangeError('Maximum call depth exceeded');
    } else {
        Cp();
    }
}
/** 检查点 */
export function CpExit(): void {
    cpDepth--;
    if (cpDepth < 1) {
        cp = Number.NaN;
        cpDepth = 0;
    } else {
        Cp();
    }
}
/** 设置检查点超时时间 */
export function configCheckpoint(timeout = 100): void {
    if (typeof timeout !== 'number' || timeout <= 0 || Number.isNaN(timeout)) {
        throw new RangeError('Invalid timeout value');
    }
    cpTimeout = timeout;
}
