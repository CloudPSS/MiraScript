export * as constants from './helpers/constants.js';
export { VmSharedContext, DefaultVmContext } from './vm/types/context.js';
export * as operations from './vm/operations.js';
export { serialize, serializeString, serializePropName } from './helpers/serialize.js';
export { lib } from './vm/lib/_loader.js';
export * from './compiler/diagnostic.js';
