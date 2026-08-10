# InvalidTypeCall

等级：**错误**

`type` 调用必须恰好有一个参数

## 如何修复

`type` 恰好接收一个参数。

### 修改前

```mira
let kind = type(1, 2);
```

### 修复后

```mira
let kind = type(1);
```
