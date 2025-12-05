/* eslint-disable unicorn/prefer-math-trunc */
import { isNaN } from '../helpers/utils.js';
// 不使用 performance.now()，精度够即可，且性能开销更小
const { now } = Date;
const TIME_ORIGIN = now() - 1000 * 3600 * 12; // 防止系统时间被调整到过去时出问题
const timestamp = () => (now() - TIME_ORIGIN) | 0;

const CP_DEFAULT_INTERVAL = 100; // 每 100 次调用检查一次
const MAX_DEPTH = 128;
const CP_UNSET = -1;
/** Default timeout in milliseconds */
const CP_DEFAULT_TIMEOUT = 100;

let cpDepth = 0;
let cp = CP_UNSET | 0;
let cpTimeout = CP_DEFAULT_TIMEOUT | 0;
let cpInterval = CP_DEFAULT_INTERVAL | 0;
let cpCounter = 0;
/** 检查点 */
export function Cp(): void {
    if (cp === CP_UNSET) {
        cpCounter = 0;
        cp = timestamp() | 0;
        return;
    }
    // 不是每次都查时间
    if ((cpCounter = (cpCounter + 1) | 0) % cpInterval !== 0) {
        return;
    }
    cpCounter = 0;
    if ((timestamp() | 0) - (cp | 0) >= (cpTimeout | 0)) {
        throw new RangeError('Execution timed out');
    }
}
/** 检查点 */
export function CpEnter(): void {
    cpDepth = (cpDepth | 0) + 1;
    if ((cpDepth | 0) <= 1) {
        cp = timestamp() | 0;
        cpDepth = 1;
    } else if ((cpDepth | 0) > MAX_DEPTH) {
        throw new RangeError('Maximum call depth exceeded');
    } else {
        Cp();
    }
}
/** 检查点 */
export function CpExit(): void {
    cpDepth = (cpDepth | 0) - 1;
    if ((cpDepth | 0) < 1) {
        cp = CP_UNSET | 0;
        cpDepth = 0;
    } else {
        Cp();
    }
}

const INT_MAX = 0x7fff_ffff;
/** 设置检查点超时时间 */
export function configCheckpoint(
    timeout: number = CP_DEFAULT_TIMEOUT,
    checkInterval: number = CP_DEFAULT_INTERVAL,
): void {
    if (typeof timeout !== 'number' || timeout <= 0 || timeout >= INT_MAX || isNaN(timeout)) {
        throw new RangeError('Invalid timeout value');
    }
    if (typeof checkInterval !== 'number' || checkInterval <= 0 || checkInterval >= INT_MAX || isNaN(checkInterval)) {
        throw new RangeError('Invalid check interval value');
    }
    cpTimeout = timeout | 0;
    cpInterval = checkInterval | 0;
}
