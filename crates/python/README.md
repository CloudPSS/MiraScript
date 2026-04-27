# MiraScript Python Bindings

MiraScript 的 Python 绑定允许在 Python 中使用 MiraScript 编译器。它提供了一个简单的接口来编译 MiraScript 脚本，并返回编译结果和诊断信息。

## 开发

1. 初始化环境

   ```bash
   # 配置 Python 虚拟环境
   python -m venv .venv
   source .venv/bin/activate # Windows: .venv\Scripts\activate
   # 安装依赖
   pip install -r requirements.txt
   # 安装 bindings 和模块
   maturin develop
   ```

2. 黑盒测试

   ```bash
   poe test
   ```

3. 生成代码并运行

   ```bash
   poe debug <file.mira>
   ```

4. 格式化

   ```bash
   poe format
   ```
