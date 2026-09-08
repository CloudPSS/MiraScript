/** 异步函数缺少执行上下文 */
export class AsyncRequiredError extends Error {
    override readonly name = 'AsyncRequiredError';
    constructor() {
        super('An async VM function requires runInContext');
    }
}

/** 重新执行时的 effect 调用顺序与记录不一致 */
export class ReplayMismatchError extends Error {
    override readonly name = 'ReplayMismatchError';
    constructor() {
        super('Host effect sequence changed during replay');
    }
}

/** 不支持嵌套的执行上下文或 effect 调用 */
export class ContextReentrancyError extends Error {
    override readonly name = 'ContextReentrancyError';
    constructor() {
        super('Nested execution contexts and host effects are not supported');
    }
}

/** 函数重新执行时的处理方式 */
export type EffectKind = 'memo' | 'once' | 'async';
/** 一次逻辑调用的执行状态与结果 */
export type EffectSlot = {
    token: symbol;
    kind: EffectKind;
    state: 'pending' | 'resolved' | 'rejected';
    value: unknown;
    ready?: Promise<void>;
};

/** 内部挂起信号，不作为脚本结果或公开 API */
export class SuspensionError extends Error {
    override readonly name = 'SuspensionError';
    constructor(readonly slot: EffectSlot) {
        super('Host effect suspended');
    }
}

const kEffectContext = Symbol('EffectContext');
/** 单次 runInContext 调用的执行状态 */
export interface EffectContext {
    /** 内部标记，用于区分 EffectContext 类型 */
    readonly __marker__: typeof kEffectContext;
    /** 下一次 effect 调用的位置 */
    cursor: number;
    /** 按执行顺序保存的调用记录 */
    slots: EffectSlot[];
    /** 是否正在执行宿主实现，用于阻止嵌套 effect */
    inHost: boolean;
    /** 待处理的挂起信号，用户代码捕获后仍保留 */
    suspension?: SuspensionError;
    /** 首次不可恢复的运行时错误，用户代码捕获后仍保留 */
    failure?: ReplayMismatchError | ContextReentrancyError;
}

/** 创建上下文 */
export function createContext(): EffectContext {
    return {
        __proto__: null,
        __marker__: kEffectContext,
        cursor: 0,
        slots: [],
        inHost: false,
    } as EffectContext;
}

export let currentContext: EffectContext | undefined;

/** 设置当前上下文，仅在脚本同步执行期间生效，返回之前的上下文 */
export function setCurrentContext(context: EffectContext | undefined): EffectContext | undefined {
    const previous = currentContext;
    currentContext = context;
    return previous;
}

/** 重置执行位置与挂起信号，保留已有调用记录 */
export function resetContext(context: EffectContext): void {
    context.cursor = 0;
    context.suspension = undefined;
}

/** 记录并抛出不可恢复的运行时错误 */
export function failContext(error: ReplayMismatchError | ContextReentrancyError): never {
    if (currentContext) currentContext.failure ??= error;
    throw error;
}

/** 重新抛出已记录的运行时错误，防止宿主代码捕获后继续执行 */
export function throwIfFailed(context: EffectContext): void {
    if (context.failure) throw context.failure;
}

/** 透传运行时控制信号，在包装异常或使用后备值前调用 */
export function rethrowControl(error: unknown): void {
    if (
        error instanceof SuspensionError ||
        error instanceof AsyncRequiredError ||
        error instanceof ReplayMismatchError ||
        error instanceof ContextReentrancyError
    ) {
        throw error;
    }
}
