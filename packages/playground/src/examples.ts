import type { InputMode } from '@mirascript/mirascript';

export const EXAMPLES: Array<{ order: number; name: string; mode: InputMode; code: () => Promise<string> }> = [];

const exampleModules = import.meta.glob('./*.{mira,miratpl}', {
    base: '../../../examples',
    query: '?raw',
    import: 'default',
});

for (const [path, content] of Object.entries(exampleModules)) {
    const filename = path.split('/').pop()!;
    const isTemplate = filename.endsWith('.miratpl');

    // 提取序号和名称
    const regex = /^(\d+)_(.+?)\.(mira|miratpl)$/;
    const match = regex.exec(filename);
    if (!match || match.length < 4) {
        // eslint-disable-next-line no-console
        console.warn(`文件名格式不正确: ${filename}`);
        continue;
    }

    const order = Number.parseInt(match[1]!, 10);
    const baseName = match[2]!;

    // 生成友好的显示名称
    const displayName = baseName
        .split('_')
        .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
        .join(' ');

    EXAMPLES.push({
        order,
        name: displayName,
        mode: isTemplate ? 'Template' : 'Script',
        code: content as () => Promise<string>,
    });
}

// 添加默认示例以防没有文件被加载
if (EXAMPLES.length === 0) {
    EXAMPLES.push({
        order: 1,
        name: 'Hello World',
        mode: 'Script',
        // eslint-disable-next-line @typescript-eslint/promise-function-async
        code: () => Promise.resolve(`debug_print("Hello, World!");`),
    });
}

// 将示例按序号排序
EXAMPLES.sort((a, b) => a.order - b.order);
