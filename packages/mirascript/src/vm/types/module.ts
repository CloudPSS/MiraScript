import { kVmModule } from '../../helpers/constants.js';
import { hasOwnEnumerable, keys } from '../../helpers/utils.js';
import type { TypeName, VmAny, VmImmutable } from './index.js';
import { VmWrapper } from './wrapper.js';

/** Mirascript 模块 */
export class VmModule<const T extends Record<string, VmImmutable> = Record<string, VmImmutable>> extends VmWrapper<T> {
    constructor(
        /** 模块名称 */
        readonly name: string,
        /** 模块导出 */
        value: T,
    ) {
        super(value);
    }
    /** @inheritdoc */
    override has(key: string): boolean {
        return hasOwnEnumerable(this.value, key);
    }
    /** @inheritdoc */
    override get(key: string): VmAny {
        if (!this.has(key)) return undefined;
        return this.value[key as keyof T] ?? null;
    }
    /** @inheritdoc */
    override keys(): string[] {
        return keys(this.value);
    }
    /** @inheritdoc */
    override same(other: VmAny): boolean {
        return this === other;
    }
    /** @inheritdoc */
    override get type(): TypeName {
        return 'module';
    }
    /** @inheritdoc */
    override get tag(): string {
        return this.name;
    }
}

Object.defineProperty(VmModule.prototype, kVmModule, { value: true });
