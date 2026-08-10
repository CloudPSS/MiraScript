# BadOmitKeyRecordExpression

等级：**错误**

无法从表达式推断键名

## 如何修复

只有变量访问可以省略记录键名；复杂表达式需要显式写出键名。

### 修改前

```mira
let user = (:get_name());
```

### 修复后

```mira
let user = (name: get_name());
```
