# LiteralNotCallable

等级：**警告**

字面量不能作为函数调用

## 如何修复

只有函数或其他可调用值才能使用调用语法。

### 修改前

```mira
let value = 42();
```

### 修复后

```mira
fn answer() { 42 }
let value = answer();
```
