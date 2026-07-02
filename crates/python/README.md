# MiraScript Python Bindings

Python 绑定用于在 Python 中调用 MiraScript 编译器能力，支持脚本编译与执行。

## 安装

```bash
pip install mirascript
```

## 快速开始

命令行执行：

```bash
mirascript -v x=1.3 -e "sin(x) + cos(PI - x)"
```

在 Python 代码中执行：

```python
from mirascript import compile

script, diagnostics = compile("sin(x) + cos(PI - x)")
result = script({"x": 1.3})
```

## 本地开发

初始化环境：

```bash
uv sync
uv run poe init
```

运行测试与检查：

```bash
uv run poe test
uv run poe check
```

格式化代码：

```bash
uv run poe format
```

调试入口（生成 `debug_output.py`）：

```bash
uv run poe debug
```
