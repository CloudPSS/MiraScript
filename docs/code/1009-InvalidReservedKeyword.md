# InvalidReservedKeyword

等级：**错误**

`$0` 是保留关键字，不能用作标识符

## 如何修复

保留关键字不能作为变量名；换一个描述用途的标识符。

### 修改前

```mira
let op = "add";
```

### 修复后

```mira
let operation = "add";
```
