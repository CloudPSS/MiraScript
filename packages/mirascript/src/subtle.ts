import { getModule } from '@mirascript/bindings';

export * as constants from './helpers/constants.js';
export { VmSharedContext, DefaultVmContext } from './vm/types/context.js';
export * as operations from './vm/operations.js';
export {
    serialize,
    serializeNil,
    serializeBoolean,
    serializeNumber,
    serializeString,
    serializePropName,
    serializeArray,
    serializeRecord,
    type SerializeOptions,
} from './helpers/serialize.js';
export { lib } from './vm/lib/_loader.js';
export * from './compiler/diagnostic.js';
export { generateBytecode, generateBytecodeSync, emitScript } from './compiler/index.js';

/** 所有 MiraScript 关键字 */
export let keywords: () => readonly string[] = () => {
    const kw = Object.freeze(getModule().keywords());
    keywords = () => kw;
    return kw;
};
