import { isNaN } from '../helpers/utils.js';
const { now } = Date;
const TIME_ORIGIN = now() - 1000 * 3600 * 24; // 减去一天，防止系统时间被调整到过去时出问题
const timestamp = () => now() - TIME_ORIGIN;

const MAX_DEPTH = 128;
const CP_UNSET = -1;
/** Default timeout in milliseconds */
const CP_DEFAULT_TIMEOUT = 100;

let cpDepth = 0;
let cp = CP_UNSET;
let cpTimeout = CP_DEFAULT_TIMEOUT;
/** 检查点 */
export function Cp(): void {
    if (cp === CP_UNSET) {
        cp = timestamp();
    } else if (timestamp() - cp > cpTimeout) {
        throw new RangeError('Execution timeout');
    }
}
/** 检查点 */
export function CpEnter(): void {
    cpDepth++;
    if (cpDepth <= 1) {
        cp = timestamp();
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
        cp = CP_UNSET;
        cpDepth = 0;
    } else {
        Cp();
    }
}
/** 设置检查点超时时间 */
export function configCheckpoint(timeout = CP_DEFAULT_TIMEOUT): void {
    if (typeof timeout !== 'number' || timeout <= 0 || isNaN(timeout)) {
        throw new RangeError('Invalid timeout value');
    }
    cpTimeout = timeout;
}
