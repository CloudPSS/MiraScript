import { program } from '@commander-js/extra-typings';
import pkg from '#package.json' with { type: 'json' };

import './commands/run.js';
import './commands/format.js';

export default program.name(pkg.name.split('/').pop()!).version(pkg.version).description(pkg.description);
