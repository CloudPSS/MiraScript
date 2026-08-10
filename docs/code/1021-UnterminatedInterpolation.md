# UnterminatedInterpolation

等级：**错误**

插值表达式未终止

## 如何修复

补全插值表达式的右括号和字符串引号。

### 修改前

```mira
let text = "total: $(price";
```

### 修复后

```mira
let price = 10;
let text = "total: $(price)";
```
