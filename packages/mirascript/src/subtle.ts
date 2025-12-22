import './compiler/load-module.js';

export { KEYWORDS, OpCode } from '@mirascript/constants';
export * as constants from './helpers/constants.js';
export * as convert from './helpers/convert/index.js';
export { DefaultVmContext } from './vm/types/context.js';
export * as operations from './vm/operations/index.js';
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
export { emitScript } from './compiler/emit-script.js';
export { createScript, wrapScript } from './compiler/create-script.js';
export { generateBytecode, generateBytecodeSync, type VmBytecodeResult } from './compiler/generate-bytecode.js';
export { type GlobalReferenceChain, analyzeGlobalReferences } from './helpers/analyze.js';
