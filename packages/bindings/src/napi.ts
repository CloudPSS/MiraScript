import { ModuleLoader } from './loader.js';
import type * as BcModule from '@mirascript/napi';

/** 字节码模块 */
export type { BcModule };
/** 字节码模块 */
export type BcModuleType = typeof BcModule;

const loader = new ModuleLoader<BcModuleType>(async () => await import('@mirascript/napi'));

/** 加载模块 */
export const loadModule: () => Promise<BcModuleType> = loader.load;
/** 获取已加载的模块 */
export const getModule: () => BcModuleType = loader.get;
