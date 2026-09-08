import type { VmScript } from '../../compiler/create-script.js';
import type { VmContext, VmValue } from '../types/index.js';
import {
    ContextReentrancyError,
    ReplayMismatchError,
    currentContext,
    failContext,
    resetContext,
    createContext,
    setCurrentContext,
} from './state.js';

/** 立即执行脚本，挂起后通过重新执行并复用 effect 结果恢复运行 */
export function runInContext(script: VmScript, globals?: VmContext | null): VmValue | Promise<VmValue>;
/** 立即执行脚本，挂起后通过重新执行并复用 effect 结果恢复运行 */
export function runInContext(
    script: VmScript,
    globals: VmContext | null | undefined,
    onDone: (result: VmValue) => void,
    onError: (error: unknown) => void,
): void;
export function runInContext(
    script: VmScript,
    globals: VmContext | null | undefined,
    onDone?: (result: VmValue) => void,
    onError?: (error: unknown) => void,
): void | VmValue | Promise<VmValue> {
    let ret: void | VmValue | Promise<VmValue> = undefined;
    if (onDone == null && onError == null) {
        ret = new Promise<VmValue>((resolve, reject) => {
            onDone = (result) => {
                ret = result;
                resolve(result);
            };
            onError = reject;
        });
    }
    if (onDone == null || onError == null) {
        throw new TypeError('Both onDone and onError must be provided if one is provided.');
    }
    if (currentContext) failContext(new ContextReentrancyError());
    const context = createContext();
    let finished = false;
    const attempt = (): void => {
        if (finished) return;
        resetContext(context);
        const previous = setCurrentContext(context);
        let result: VmValue = null;
        let failed = false;
        let failure: unknown;
        try {
            result = script(globals);
            if (!context.suspension && context.cursor !== context.slots.length) {
                failContext(new ReplayMismatchError());
            }
        } catch (error) {
            failed = true;
            failure = error;
        } finally {
            setCurrentContext(previous);
        }
        const { suspension } = context;
        if (context.failure) {
            failed = true;
            failure = context.failure;
        } else if (suspension) {
            void suspension.slot.ready!.then(() => {
                // 避免回调异常进入内部 Promise 链
                queueMicrotask(attempt);
            });
            return;
        }
        finished = true;
        context.slots.length = 0;
        if (failed) onError!(failure);
        else onDone!(result);
    };
    attempt();
    return ret;
}
