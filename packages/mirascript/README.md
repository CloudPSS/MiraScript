# @mirascript/mirascript

`@mirascript/mirascript` 是 MiraScript 的核心 TypeScript API，负责连接底层编译器、生成可执行脚本，并导出运行时与虚拟机相关能力。

## 能力概览

- 将 MiraScript 源码编译为可执行的 JavaScript 包装对象
- 提供同步与异步编译接口
- 导出编译器、VM、序列化等上层 API
- 在需要时自动加载底层编译器模块

## 安装

```bash
pnpm add @mirascript/mirascript
```

## 基本示例

```ts
import { compile, compileSync } from '@mirascript/mirascript';

const script = await compile('1 + 2');
console.log(script());

const syncScript = compileSync('let x = 3; x * 7');
console.log(syncScript());
```

## 常用导出

- `compile()` / `compileSync()`：编译源代码
- `serialize()`：序列化运行时值
- `./subtle`：较底层的接口与工具
- `compiler/*`、`vm/*`：编译器与运行时内部模块

## 开发

```bash
pnpm --filter @mirascript/mirascript build
pnpm --filter @mirascript/mirascript watch
pnpm --filter @mirascript/mirascript test
pnpm --filter @mirascript/mirascript bench
```

如果你需要直接访问底层 Rust/WASM 编译器能力，优先查看 `@mirascript/bindings`、`@mirascript/napi` 和 `@mirascript/wasm`。
