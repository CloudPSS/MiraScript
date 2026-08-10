# MissingOpenParenAfterType

等级：**错误**

`type` 是类函数关键字；请在此处添加 `(`

## 如何修复

`type` 是类函数关键字，参数必须放在括号中。

### 修改前

```mira
let kind = type 1;
```

### 修复后

```mira
let kind = type(1);
```
