import { program } from '@commander-js/extra-typings';
import pkg from '#package.json' with { type: 'json' };

import './commands/run.js';
import './commands/format.js';
import './commands/repl.js';

let p = program;
const binName = Object.keys(pkg.bin)[0];
if (binName) {
    p = program.name(binName);
}
export default p.version(pkg.version).description(pkg.description);
