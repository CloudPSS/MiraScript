# @mirascript/napi

`@mirascript/napi` 提供 MiraScript 编译器的 Node.js 原生绑定，适合服务端和本地工具链场景。

## 能力概览

- 基于 `napi-rs` 构建
- 面向 Node.js 运行时
- 支持多平台目标产物
- 常作为 `@mirascript/bindings` 的后端实现

## 安装

```bash
pnpm add @mirascript/napi
```

## 使用建议

大多数场景建议直接依赖 `@mirascript/bindings`，由其自动选择 NAPI 或 WASM 实现。仅在明确需要原生实现时再直接依赖本包。

## 构建

```bash
pnpm --filter @mirascript/napi build:debug
pnpm --filter @mirascript/napi build
```

- `build:debug`：构建当前平台调试产物
- `build`：构建发布产物

构建依赖 Rust 工具链。
