# GlobalVariableNotDeclared

等级：**警告**

全局变量 `$0` 未声明

## 如何修复

检查全局变量名，并在宿主环境中声明后再访问。

### 修改前

```mira
let user = global.uesr;
```

### 修复后

```mira
let user = global.user;
```
