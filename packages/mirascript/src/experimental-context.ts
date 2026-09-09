import { VmFunction, type VmFunctionLike, type VmFunctionOptionLike } from './vm/types/function.js';
import { wrapEffect } from './vm/effects/wrap.js';
import type { VmValue } from './vm/types/index.js';
import { defineProperty } from './helpers/utils.js';

/**
 * ## 实验性执行上下文
 * JS backend 可通过 `experimental-context` 显式启用可挂起的 Host Function。
 * 生成的脚本仍是同步函数，运行时在异步操作完成后从入口重新执行，并复用已记录的 effect。
 *
 * ```ts
 * import { compileSync, createVmContext, VmFunction } from '@mirascript/mirascript';
 * import { runInContext } from '@mirascript/mirascript/experimental-context';
 *
 * const globals = createVmContext({
 *   load: VmFunction.async(
 *     async (id) => {
 *       if (typeof id !== 'string') throw new TypeError('id must be a string');
 *       // Host 实现显式转换外部结果，返回合法 VM 值。
 *       return { id, value: 42 };
 *     },
 *     { name: 'load', summary: '加载数据' },
 *   ),
 * });
 *
 * runInContext(
 *   compileSync('load("example").value'),
 *   globals,
 *   (value) => console.log(value),
 *   (error) => console.error(error),
 * );
 * ```
 *
 * 实际导入实验入口后，现有 `VmFunction` 对象获得以下方法；也可使用
 * `import '@mirascript/mirascript/experimental-context'` 仅安装扩展。
 * `import type` 不会安装运行时方法。模块声明增强在包含该入口的整个 TypeScript 编译项目中生效。
 *
 * | 方法                            | 用途                          | 在执行上下文中的行 * 为                             |
 * | ------------------------------- | ----------------------------- |  * ------------------------------------------------ |
 * | `VmFunction.memo(fn, option?)`  | 随机数、时间、外部状态读取    | 每个逻辑调用缓存结果或异 * 常                       |
 * | `VmFunction.once(fn, option?)`  | 输出等立即发生的副作用        | 每个逻辑调用只执行一次，缓存结果或异 * 常，不回滚   |
 * | `VmFunction.async(fn, option?)` | 返回 PromiseLike 的 Host 操作 | 启动一次，等待完成，在原逻辑调用处返 * 回结果或抛错 |
 *
 * 三个方法都直接返回已包装的 `VmFunction`，`option` 与普通构造器一致。
 * 输入参数遵循 `VmFunctionLike`，包含缺省参数的 `undefined`；返回值或异步完成值必须为
 * `VmAny | void`。三个方法均为泛型函数，像普通 `VmFunction` 一样保留输入函数的类型；
 * `async` 保留参数与 `this` 类型，将 PromiseLike 的完成值作为同步返回值。
 * 包装器不转换普通 JS 对象，调用方自行检查或断言 VM 参数与脚本结果。
 *
 * `runInContext(script, globals, onDone, onError)` 接受 `VmScript` 和
 * `VmContext | null | undefined`，返回 `void`。结果固定为 `VmValue`，错误为 `unknown`。
 * 无挂起时，完成或错误回调在当前调用栈中执行；挂起后，通过微任务恢复执行。
 * 回调只通知一次，并在退出执行上下文后调用。回调自身抛错不会转交另一个回调；
 * 同步时直接传播，异步时作为微任务异常抛出。
 *
 * 普通 `script(globals)` 保持同步，不创建 effect journal。
 * `memo/once` 在上下文外直接执行，`async` 则在启动 Host 实现之前抛出 `AsyncRequiredError`。
 * 实验入口同时导出 `ReplayMismatchError` 和 `ContextReentrancyError`。
 *
 * 内置 `random`、时间转换函数未传日期或传入 `nil` 时的当前时间读取，以及
 * `debug_print`、`panic` 的输出部分已接入 effect 管理。日志沿用原有异步输出时序，
 * 完成回调不等待日志输出。Checkpoint 超时仅约束每轮同步执行，不包括异步等待。
 *
 * ### Replay 条件与限制
 *
 * - 包装函数应在执行入口之外创建，确保每次 replay 使用相同的函数身份。
 * - 缓存按调用顺序记录，不按参数去重；循环中的同一函数可有多个独立逻辑调用。
 * - 未包装的函数、globals 查询和外部对象读取必须满足 replay 条件。运行时不快照 globals、
 *   拦截 getter 或外部对象写入，也不管理未声明的副作用。
 * - 结果按引用缓存，不拷贝或冻结。不要在运行期间修改缓存对象或影响控制流的外部状态。
 * - 函数身份或调用序列长度变化会报错，但不会比较参数，不能检测所有 replay 分歧。
 * - 支持独立执行交错等待；拒绝执行中的嵌套 `runInContext`，以及被管理 Host 实现同步调用
 *   其他 effect。Host 实现在 `await` 后回调脚本执行 effect 不受支持。
 * - 不要捕获并吞掉运行时控制信号后继续产生副作用。已记录的挂起或致命契约错误不会因此消失。
 * - 不提供取消、事务、deferred commit 或跨安装副本的 context 互操作；Rust/Python backend 不变。
 */

/**
 * 异步 Mirascript 函数签名
 * @see {@link VmFunctionLike}
 */
type VmAsyncFunctionLike = (
    this: void,
    ...args: ReadonlyArray<VmValue | undefined>
) => PromiseLike<ReturnType<VmFunctionLike>> | ReturnType<VmFunctionLike>;

/** 保留异步函数的参数类型，将完成值作为同步返回值 */
type ResumedFunction<T extends VmAsyncFunctionLike> = T extends (...args: infer A extends readonly unknown[]) => unknown
    ? (this: void, ...args: A) => Awaited<ReturnType<T>>
    : never;

declare module './vm/types/function.js' {
    namespace VmFunction {
        /** 创建读取函数，每次逻辑调用缓存返回值或异常 */
        function memo<T extends VmFunctionLike>(fn: T, option?: VmFunctionOptionLike<T>): VmFunction<T>;
        /** 创建副作用函数，每次逻辑调用最多执行一次，不支持回滚 */
        function once<T extends VmFunctionLike>(fn: T, option?: VmFunctionOptionLike<T>): VmFunction<T>;
        /** 创建可挂起的异步函数，需要通过 runInContext 执行 */
        function async<T extends VmAsyncFunctionLike>(
            fn: T,
            option?: VmFunctionOptionLike<ResumedFunction<T>>,
        ): VmFunction<ResumedFunction<T>>;
    }
}

/** 创建 effect 函数，已标记的 Mirascript 函数也需要重新包装 */
function createEffectWrapper(kind: 'memo' | 'once' | 'async') {
    return (fn: VmFunctionLike | VmAsyncFunctionLike, option?: VmFunctionOptionLike) => {
        if (typeof fn != 'function') throw new TypeError('Invalid function');
        const wrapped = wrapEffect<ReadonlyArray<VmValue | undefined>, ReturnType<VmFunctionLike>>(kind, fn);
        defineProperty(wrapped, 'name', { value: fn.name, configurable: true });
        return VmFunction(wrapped, option);
    };
}

defineProperty(VmFunction, 'memo', { value: createEffectWrapper('memo') });
defineProperty(VmFunction, 'once', { value: createEffectWrapper('once') });
defineProperty(VmFunction, 'async', { value: createEffectWrapper('async') });

export { runInContext } from './vm/effects/run.js';
export { AsyncRequiredError, ReplayMismatchError, ContextReentrancyError } from './vm/effects/state.js';
