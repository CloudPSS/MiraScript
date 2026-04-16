# @mirascript/bindings

`@mirascript/bindings` 提供 MiraScript 编译器的统一 JavaScript 绑定层。它会优先在 Node.js 环境加载 `@mirascript/napi` 原生实现，在浏览器或无法加载原生模块时回退到 `@mirascript/wasm`。

## 适用场景

- 需要同时支持 Node.js 和浏览器环境
- 希望按统一接口访问底层编译器模块
- 作为上层包的基础依赖，例如 `@mirascript/mirascript`、`@mirascript/cli`、`@mirascript/monaco`

## 安装

```bash
pnpm add @mirascript/bindings
```

## 基本用法

```ts
import { loadModule, getModule } from '@mirascript/bindings';

const mod = await loadModule();
const sameModule = getModule();

console.log(mod === sameModule);
```

## 导出内容

- `loadModule()`：异步加载底层编译器模块
- `getModule()`：获取已加载模块；若未加载会抛错
- `@mirascript/bindings/wasm`：显式使用 WebAssembly 实现
- `@mirascript/bindings/napi`：显式使用 Node 原生实现

## 开发

```bash
pnpm --filter @mirascript/bindings build
pnpm --filter @mirascript/bindings watch
```

该包本身只负责装配与分发，实际编译能力来自 `@mirascript/napi` 与 `@mirascript/wasm`。
