`is` 用于模式匹配：`value is pattern`，结果为 `boolean`。

```mira
1 is 1;        // true
1 is > 0;      // true
(a: 1) is (a: x); // true（并绑定 x）
```
