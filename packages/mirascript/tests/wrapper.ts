import test from 'ava';
import { VmExtern, VmModule } from 'mirascript';

test('toJSON', (t) => {
    t.deepEqual(JSON.stringify(new VmExtern({})), undefined);
    t.deepEqual(JSON.stringify(new VmModule('test', {})), undefined);
});
