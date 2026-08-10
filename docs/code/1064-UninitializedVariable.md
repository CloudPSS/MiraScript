# UninitializedVariable

等级：**错误**

变量无法在此之前访问...

## 如何修复

确保变量在当前执行路径上完成绑定后再读取。

### 修改前

```mira
value == 1 && 1 is value
```

### 修复后

```mira
1 is value && value == 1
```
