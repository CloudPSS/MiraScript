# NonNumberOrStringInComparison

等级：**警告**

比较表达式中不能使用非数字或字符串字面量

## 如何修复

关系模式右侧使用数字或字符串字面量。

### 修改前

```mira
value is > true
```

### 修复后

```mira
value is > 0
```
