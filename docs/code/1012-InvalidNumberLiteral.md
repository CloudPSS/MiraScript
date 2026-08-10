# InvalidNumberLiteral

等级：**错误**

无效的数字字面量

## 如何修复

补全数字的整数、小数或指数部分。

### 修改前

```mira
let rate = 1e+;
```

### 修复后

```mira
let rate = 1e+2;
```
