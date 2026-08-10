# ImmutableVariableAssignment

等级：**错误**

不能对不可变变量赋值...

## 如何修复

需要重新赋值时，在声明中加入 `mut`；否则创建新的绑定。

### 修改前

```mira
let count = 0;
count = 1;
```

### 修复后

```mira
let mut count = 0;
count = 1;
```
