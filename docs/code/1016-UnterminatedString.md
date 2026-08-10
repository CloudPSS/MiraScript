# UnterminatedString

等级：**错误**

字符串字面量未终止

## 如何修复

在字符串末尾补上与开头匹配的引号。

### 修改前

```mira
let greeting = "hello;
```

### 修复后

```mira
let greeting = "hello";
```
