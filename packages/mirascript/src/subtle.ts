export * as constants from './helpers/constants.ts';
export { VmSharedContext, DefaultVmContext } from './vm/types/context.ts';
export * as operations from './vm/operations.ts';
export { serialize, serializeString, serializePropName, type SerializeOptions } from './helpers/serialize.ts';
export { lib } from './vm/lib/_loader.ts';
export * from './compiler/diagnostic.ts';
