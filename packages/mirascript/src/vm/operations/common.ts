import { VmError } from '../../helpers/error.js';
import type { VmAny, VmValue } from '../types/index.js';

export const $AssertInit: (value: VmAny) => asserts value is VmValue = (value) => {
    if (value === undefined) throw new VmError(`Uninitialized value`, null);
};
