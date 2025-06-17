import { createRequire } from 'node:module';
import type { NapiModule } from './type';

const require = createRequire(import.meta.url);
export const { compile, compileSync } = require('#lib') as NapiModule;

// import fs from 'node:fs/promises';
// const script = await fs.readFile('../../examples/fib.mira', 'utf8');

// const COUNT = 1_000_000;

// {
//     const start = performance.now();
//     for (let i = 1; i < COUNT; i++) {
//         await compile(script, {});
//     }
//     const end = performance.now();
//     console.log(`Async sequential: ${((end - start) / COUNT) * 1000} us`);
// }
// {
//     const start = performance.now();
//     await Promise.all(Array.from({ length: COUNT }, () => compile(script, {})));
//     const end = performance.now();
//     console.log(`Async parallel: ${((end - start) / COUNT) * 1000} us`);
// }

// {
//     const start = performance.now();
//     for (let i = 1; i < COUNT; i++) {
//         compileSync(script, {});
//     }
//     const end = performance.now();
//     console.log(`Sync sequential: ${((end - start) / COUNT) * 1000} us`);
// }
