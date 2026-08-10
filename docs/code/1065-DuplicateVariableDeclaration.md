# DuplicateVariableDeclaration

等级：**错误**

该变量已...

## 如何修复

同一作用域不要重复声明同名变量；改名或直接给可变变量赋值。

### 修改前

```mira
let value = 1;
let value = 2;
```

### 修复后

```mira
let first = 1;
let second = 2;
```
