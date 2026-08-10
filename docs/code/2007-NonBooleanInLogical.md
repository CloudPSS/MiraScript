# NonBooleanInLogical

等级：**警告**

逻辑表达式中不能使用非布尔字面量

## 如何修复

逻辑运算符两侧应为布尔表达式。

### 修改前

```mira
let allowed = 1 && true;
```

### 修复后

```mira
let allowed = true && true;
```
