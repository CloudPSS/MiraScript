# @mirascript/bindings

`@mirascript/bindings` 提供统一的编译器加载入口。它在 Node.js 环境优先加载 `@mirascript/napi`，在浏览器或原生模块不可用时回退到 `@mirascript/wasm`。

## 适用场景

- 需要同时支持 Node.js 与浏览器
- 希望用同一套 API 访问编译器模块
- 作为上层包（如 `@mirascript/mirascript`、`@mirascript/cli`、`@mirascript/monaco`）的底层依赖

## 安装

```bash
pnpm add @mirascript/bindings
```

## 快速开始

```ts
import { loadModule, getModule } from '@mirascript/bindings';

const mod = await loadModule();
const sameModule = getModule();

console.log(mod === sameModule);
```

## 导出

- `loadModule()`：异步加载底层模块
- `getModule()`：获取已加载模块（未加载时抛错）
- `@mirascript/bindings/wasm`：强制使用 WebAssembly 实现
- `@mirascript/bindings/napi`：强制使用 Node 原生实现

## 开发

```bash
pnpm --filter @mirascript/bindings build
pnpm --filter @mirascript/bindings watch
```

该包只负责装配与分发，实际编译能力由 `@mirascript/napi` 和 `@mirascript/wasm` 提供。
