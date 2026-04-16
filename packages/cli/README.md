# @mirascript/cli

`@mirascript/cli` 是 MiraScript 的命令行工具，提供脚本执行和代码格式化能力，对外暴露 `mirascript` 可执行命令。

## 安装

```bash
pnpm add -D @mirascript/cli
```

或直接在工作区中调用：

```bash
pnpm --filter @mirascript/cli build
```

## 命令概览

### 运行脚本

```bash
mirascript run examples/01_hello_world.mira
```

支持的常用选项：

- `-e, --eval <script>`：直接执行内联脚本
- `-t, --template`：以模板模式执行
- `-v, --variable <key=value>`：注入全局变量，可重复使用
- `--timeout <ms>`：设置执行超时，`0` 表示不限制

示例：

```bash
mirascript run -e "1 + 2"
mirascript run --template template.miratpl
mirascript run -v name=world examples/11_interpolation.mira
```

### 格式化脚本

```bash
mirascript format examples/**/*.mira
mirascript format --write examples/**/*.mira
```

支持从标准输入读取：

```bash
cat script.mira | mirascript format -
```

## 开发

```bash
pnpm --filter @mirascript/cli build
pnpm --filter @mirascript/cli watch
```

该包依赖 `@mirascript/mirascript` 与 `@mirascript/bindings`，因此本地开发前应先完成工作区依赖安装。
