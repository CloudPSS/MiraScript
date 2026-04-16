# @mirascript/wasm

`@mirascript/wasm` 提供 MiraScript 编译器的 WebAssembly 封装，适用于浏览器、Web Worker 以及不方便加载原生 Node 模块的环境。

## 功能

- 初始化 WebAssembly 编译器模块
- 创建底层编译配置对象
- 提供同步编译接口，输出字节码和诊断信息

## 安装

```bash
pnpm add @mirascript/wasm
```

## 基本用法

```ts
import { init, compileSync } from '@mirascript/wasm';

await init();

const result = compileSync('1 + 2', {
  input_mode: 'Script',
});

console.log(result.diagnostics);
console.log(result.chunk);
```

## 构建

```bash
pnpm --filter @mirascript/wasm build:dev
pnpm --filter @mirascript/wasm build
```

构建依赖 `wasm-pack` 与 Rust 工具链。

## 说明

如果你的代码既要在 Node.js 运行，也要在浏览器运行，优先使用 `@mirascript/bindings`，由它自动选择原生或 WebAssembly 实现。
