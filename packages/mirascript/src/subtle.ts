import { getModule } from '@mirascript/bindings';

export * as constants from './helpers/constants.js';
export * as convert from './helpers/convert/index.js';
export { DefaultVmContext } from './vm/types/context.js';
export * as operations from './vm/operations.js';
export {
    display as serializeForDisplay,
    serialize,
    serializeNil,
    serializeBoolean,
    serializeNumber,
    serializeString,
    serializeRecordKey,
    serializeArray,
    serializeRecord,
    type SerializeOptions,
} from './helpers/serialize.js';
export { lib } from './vm/lib/index.js';
export * from './compiler/diagnostic.js';
export { generateBytecode, generateBytecodeSync, emitScript } from './compiler/index.js';

/** 所有 MiraScript 关键字 */
export let keywords: () => readonly string[] = () => {
    const kw = Object.freeze(getModule().keywords());
    keywords = () => kw;
    return kw;
};
