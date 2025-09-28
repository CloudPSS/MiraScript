/* eslint-disable no-console */
import { compile, serialize } from './index.js';

const script = process.argv[2];
if (!script) {
    console.error('请提供要执行的脚本');
    process.exit(1);
}
try {
    const f = await compile(script);
    const r = f();

    console.log(serialize(r));
} catch (ex) {
    console.error((ex as Error).message);
    process.exit(2);
}
