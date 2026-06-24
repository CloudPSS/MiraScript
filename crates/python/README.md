# MiraScript Python Bindings

MiraScript 的 Python 绑定允许在 Python 中使用 MiraScript 编译器。它提供了一个简单的接口来编译 MiraScript 脚本，并返回编译结果和诊断信息。

## 安装

```bash
pip install mirascript
```

## 基本示例

作为 CLI 工具使用：

```bash
mirascript -v x=1.3 -e "sin(x) + cos(PI - x)"
```

在代码中使用：

```python
from mirascript import compile

script, diagnostics = compile("sin(x) + cos(PI - x)")
result = script({"x": 1.3})
```

## 开发

1. 初始化环境

   ```bash
   # 安装依赖并安装 bindings 和模块
   uv sync
   uv run poe init
   ```

2. 黑盒测试

   ```bash
   uv run poe test
   ```

3. 生成代码并运行

   ```bash
   uv run poe debug <file.mira>
   ```

4. 格式化

   ```bash
   uv run poe format
   ```
