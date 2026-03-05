import { ModuleLoader } from './loader.js';
import type * as BcModule from '@mirascript/wasm';

/** 字节码模块 */
export type { BcModule };
/** 字节码模块 */
export type BcModuleType = typeof BcModule;

const loader = new ModuleLoader<BcModuleType>(async () => {
    const wasm = await import('@mirascript/wasm');
    await wasm.init();
    return wasm;
});

/** 加载模块 */
export const loadModule: () => Promise<BcModuleType> = loader.load;
/** 获取已加载的模块 */
export const getModule: () => BcModuleType = loader.get;
