import test, { type ExecutionContext } from 'ava';
import { VmError, type VmValue } from '../dist/index.js';

function checkVmError(
    t: ExecutionContext,
    vmError: VmError,
    message: string,
    originalError: Error,
    recoveredValue: VmValue,
) {
    t.is(vmError.message, message);
    t.is(vmError.recovered, recoveredValue);
    t.is(vmError.cause, originalError);
    t.is(vmError.stack, originalError.stack);
}

test('from error', (t) => {
    const originalError = new Error('Original error message');
    const recoveredValue = 42;
    const vmError = VmError.from('Prefix', originalError, recoveredValue);

    checkVmError(t, vmError, 'Prefix: Original error message', originalError, recoveredValue);
});

test('from non-error', (t) => {
    const originalError = 'Some error string';
    const recoveredValue = null;
    const vmError = VmError.from('Error occurred', originalError, recoveredValue);

    t.is(vmError.message, 'Error occurred: Some error string');
    t.is(vmError.recovered, recoveredValue);
    t.is(vmError.cause, originalError);
    t.true(typeof vmError.stack == 'string' && vmError.stack.length > 0);
});

test('from empty prefix', (t) => {
    const originalError = new Error('No prefix error');
    const recoveredValue = 'recovered';
    const vmError = VmError.from('', originalError, recoveredValue);

    checkVmError(t, vmError, 'No prefix error', originalError, recoveredValue);
});

test('from prefix without colon', (t) => {
    const originalError = new Error('Missing colon');
    const recoveredValue = true;
    const vmError = VmError.from('Warning', originalError, recoveredValue);

    checkVmError(t, vmError, 'Warning: Missing colon', originalError, recoveredValue);
});

test('from prefix with colon', (t) => {
    const originalError = new Error('Has colon');
    const recoveredValue = false;
    const vmError = VmError.from('Error:', originalError, recoveredValue);

    checkVmError(t, vmError, 'Error: Has colon', originalError, recoveredValue);
});

test('recovered value types', (t) => {
    const error = new Error('Test error');

    const vmErrorNumber = VmError.from('Number recovered', error, 123);
    t.is(vmErrorNumber.recovered, 123);

    const vmErrorString = VmError.from('String recovered', error, 'recovered');
    t.is(vmErrorString.recovered, 'recovered');

    const vmErrorObject = VmError.from('Object recovered', error, { key: 'value' });
    t.deepEqual(vmErrorObject.recovered, { key: 'value' });
});

test('stack trace preservation', (t) => {
    const originalError = new Error('Stack trace test');
    const recoveredValue = undefined;
    const vmError = VmError.from('Stack Test', originalError, recoveredValue);

    t.is(vmError.stack, originalError.stack);
});

test('non-error cause with stack', (t) => {
    const originalError = { message: 'Custom error object', stack: 'Custom stack trace' };
    const recoveredValue = 0;
    const vmError = VmError.from('Custom', originalError, recoveredValue);

    t.is(vmError.message, 'Custom: [object Object]');
    t.is(vmError.recovered, recoveredValue);
    t.is(vmError.cause, originalError);
    t.not(vmError.stack, undefined);
    t.not(vmError.stack, originalError.stack);
});
