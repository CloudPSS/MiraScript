# InvalidKeyword

等级：**错误**

`$0` 是关键字，不能用作标识符

## 如何修复

语言关键字不能作为变量名；将变量重命名。

### 修改前

```mira
fn true() { 1 }
```

### 修复后

```mira
fn is_enabled() { true }
```
