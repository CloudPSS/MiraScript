# UnexpectedGlobal

等级：**错误**

意外的 `global`；它是用于全局变量的保留关键字

## 如何修复

`global` 不是普通标识符；通过字段或索引访问具体的全局变量。

### 修改前

```mira
let value = global;
```

### 修复后

```mira
let value = global.answer;
```
