# MiraScript Python Bindings

MiraScript 的 Python 绑定允许在 Python 中使用 MiraScript 编译器。它提供了一个简单的接口来编译 MiraScript 脚本，并返回编译结果和诊断信息。

## 开发

1. 初始化虚拟环境

   ```bash
   python -m venv .venv
   source .venv/bin/activate # Windows: .venv\Scripts\activate
   ```

2. 安装依赖

   ```bash
   pip install -r requirements.txt
   ```

3. 编译并安装 bindings

   ```bash
   maturin develop
   ```
