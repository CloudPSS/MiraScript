# @mirascript/cli

`@mirascript/cli` 是 MiraScript 的命令行工具，提供脚本执行、REPL 与格式化能力，对外暴露 `mirascript` 命令。

## 安装

```bash
pnpm add -D @mirascript/cli
```

## 命令概览

### 运行脚本

```bash
mirascript run examples/01_hello_world.mira
```

常用选项：

- `-e, --eval <script>`：直接执行内联脚本
- `-t, --template`：以模板模式执行
- `--no-template`：强制使用脚本模式
- `-v, --variable <key=value>`：注入全局变量，可重复使用
- `--timeout <ms>`：执行超时（`0` 表示不超时）

如果未提供脚本路径和 `-e`，会进入 REPL。

### 格式化脚本

```bash
mirascript format examples/**/*.mira
mirascript format --write examples/**/*.mira
```

也支持从标准输入读取：

```bash
mirascript format - < script.mira
```

## 开发

```bash
pnpm --filter @mirascript/cli build
pnpm --filter @mirascript/cli watch
```

本包依赖 `@mirascript/mirascript` 与 `@mirascript/bindings`，建议先在仓库根目录完成依赖安装。
