# @mirascript/wasm

`@mirascript/wasm` 提供 MiraScript 编译器的 WebAssembly 封装，适用于浏览器、Web Worker 和其他不便使用原生模块的环境。

## 能力概览

- 初始化 WebAssembly 编译模块
- 提供同步编译接口
- 返回编译产物与诊断信息

## 安装

```bash
pnpm add @mirascript/wasm
```

## 快速开始

```ts
import { init, compileSync } from '@mirascript/wasm';

await init();

const result = compileSync('1 + 2', {
  input_mode: 'Script',
});

console.log(result.diagnostics);
console.log(result.chunk);
```

## 使用建议

如果项目需要同时兼容 Node.js 与浏览器，优先使用 `@mirascript/bindings` 统一加载入口。

## 构建

```bash
pnpm --filter @mirascript/wasm build:dev
pnpm --filter @mirascript/wasm build
```

构建依赖 Rust 工具链与 `wasm-pack`。
