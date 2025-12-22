import * as bundle from '#bundle';
import * as wasm from './wasm.js';
import { ModuleLoader } from './loader.js';

/** 字节码模块 */
export type { BcModule } from '#bundle';
/** 字节码模块 */
export type BcModuleType = wasm.BcModuleType | bundle.BcModuleType;

const loader = new ModuleLoader<BcModuleType>(async () => {
    try {
        return await bundle.loadModule();
        /* c8 ignore next 5 */
    } catch (ex) {
        // eslint-disable-next-line no-console
        console.warn('Failed to load compiler bundle, falling back to @mirascript/wasm');
        return await wasm.loadModule();
    }
});

/** 加载模块 */
export const loadModule: () => Promise<BcModuleType> = loader.load;
/** 获取已加载的模块 */
export const getModule: () => BcModuleType = loader.get;
