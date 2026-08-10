# UndefinedVariableAssignment

等级：**错误**

不能对未声明的变量赋值

## 如何修复

先用 `let mut` 声明变量，再进行赋值。

### 修改前

```mira
count = 1;
```

### 修复后

```mira
let mut count = 0;
count = 1;
```
