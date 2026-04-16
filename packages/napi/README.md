# @mirascript/napi

`@mirascript/napi` 提供 MiraScript 编译器的 Node.js 原生绑定，是面向服务端和本地 CLI 场景的高性能实现。该包通常作为底层依赖被 `@mirascript/bindings` 自动加载，而不是由业务代码直接依赖。

## 特点

- 基于 `napi-rs` 构建
- 面向 Node.js 环境
- 支持多平台预构建二进制发布
- 适合 CLI、构建工具和本地开发服务场景

## 安装

```bash
pnpm add @mirascript/napi
```

## 典型关系

大多数情况下建议直接使用 `@mirascript/bindings`：

```ts
import { loadModule } from '@mirascript/bindings';

const mod = await loadModule();
```

只有在你明确需要直接依赖原生实现时，才需要单独引入本包。

## 构建

```bash
pnpm --filter @mirascript/napi build:debug
pnpm --filter @mirascript/napi build
```

- `build:debug`：本机构建当前平台二进制
- `build`：优先尝试通过 Docker Buildx 生成多平台产物，失败时回退到本地构建

构建本包需要 Rust 工具链；多平台构建还需要 Docker Buildx。
