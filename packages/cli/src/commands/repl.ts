import { program } from '@commander-js/extra-typings';
import { startRepl } from '../utils/repl.js';

program
    .command('repl')
    .description('启动 MiraScript REPL（交互式命令行）')
    .action(async () => {
        await startRepl();
    });
