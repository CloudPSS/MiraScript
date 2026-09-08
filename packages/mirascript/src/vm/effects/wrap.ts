import { apply } from '../../helpers/utils.js';
import {
    AsyncRequiredError,
    ContextReentrancyError,
    ReplayMismatchError,
    SuspensionError,
    currentContext,
    failContext,
    rethrowControl,
    throwIfFailed,
    type EffectKind,
    type EffectSlot,
} from './state.js';

/** 包装 effect 调用，也支持标准库内部使用的参数与返回值 */
export function wrapEffect<A extends readonly unknown[], R>(
    kind: EffectKind,
    fn: (...args: A) => R | PromiseLike<R>,
): (...args: A) => R {
    const token = Symbol();
    return function (this: unknown, ...args: A): R {
        const context = currentContext;
        if (!context) {
            if (kind === 'async') throw new AsyncRequiredError();
            return apply(fn, this, args) as R;
        }
        throwIfFailed(context);
        if (context.inHost) failContext(new ContextReentrancyError());
        if (context.suspension) throw context.suspension;
        const index = context.cursor++;
        let slot = context.slots[index];
        if (slot) {
            if (slot.token !== token || slot.kind !== kind) failContext(new ReplayMismatchError());
        } else {
            slot = { token, kind, state: 'pending', value: undefined };
            context.slots.push(slot);
            context.inHost = true;
            try {
                const value = apply(fn, this, args);
                if (kind === 'async') {
                    const pending: EffectSlot = slot;
                    pending.ready = Promise.resolve(value).then(
                        (result) => {
                            pending.state = 'resolved';
                            pending.value = result;
                        },
                        (error: unknown) => {
                            pending.state = 'rejected';
                            pending.value = error;
                        },
                    );
                } else {
                    slot.state = 'resolved';
                    slot.value = value;
                }
            } catch (error) {
                rethrowControl(error);
                slot.state = 'rejected';
                slot.value = error;
            } finally {
                context.inHost = false;
            }
        }
        throwIfFailed(context);
        // 保留原始异常值，包括 undefined 和 null
        if (slot.state === 'rejected') throw slot.value;
        if (slot.state === 'resolved') return slot.value as R;
        context.suspension = new SuspensionError(slot);
        throw context.suspension;
    };
}
