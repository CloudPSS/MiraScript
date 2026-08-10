# InvalidNumberLiteralUnderscore

等级：**错误**

数字字面量不能以下划线开头或结尾

## 如何修复

下划线只能放在数字中间用于分组，不能放在开头或结尾。

### 修改前

```mira
let size = 100_;
```

### 修复后

```mira
let size = 1_000;
```
