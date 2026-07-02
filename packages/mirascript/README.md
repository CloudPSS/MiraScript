# @mirascript/mirascript

`@mirascript/mirascript` 是 MiraScript 的核心 TypeScript API，负责脚本编译、运行时桥接与上层能力导出。

## 能力概览

- 将 MiraScript 源码编译为可执行函数
- 提供同步与异步编译接口
- 导出编译器、运行时和序列化相关 API
- 在需要时自动加载底层编译器模块

## 安装

```bash
pnpm add @mirascript/mirascript
```

## 快速开始

```ts
import { compile, compileSync } from '@mirascript/mirascript';

const script = await compile('1 + 2');
console.log(script());

const syncScript = compileSync('let x = 3; x * 7');
console.log(syncScript());
```

## 常用导出

- `compile()` / `compileSync()`：编译源码
- `serialize()`：序列化运行时值
- `./subtle`：较底层接口与工具

## 开发

```bash
pnpm --filter @mirascript/mirascript build
pnpm --filter @mirascript/mirascript watch
pnpm --filter @mirascript/mirascript test
pnpm --filter @mirascript/mirascript bench
```

如需直接访问具体编译后端，可参考 `@mirascript/bindings`、`@mirascript/napi` 与 `@mirascript/wasm`。
