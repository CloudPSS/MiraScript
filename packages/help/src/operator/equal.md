---
token: '=='
---

使用 `==` 运算符可以比较两个值是否严格相等。

---

对于原始值，如果它们的类型和值都相同，则认为它们相等。

```mira
5 == 5;            // true
"hello" == "hello"; // true
true == true;      // true
5 == "5";         // false
```

特别地，对于 `number` 类型的值 `nan`，它与任何值都不相等，包括它自己。

```mira
nan == nan; // false
```

---

对于 `array` 和 `record`，只有他们的元素和属性都相等时，才认为它们相等。

```mira
[1, 2, 3] == [1, 2, 3]; // true
(a: 1, b: 2) == (a: 1, b: 2); // true
[1, 2, 3] == [1, 2, 4]; // false
```

---

对于 `function` `module` 和 `extern`，只有它们引用的是同一个对象时，才认为它们相等。

```mira
let fn1 = fn() { return 1; };
let fn2 = fn() { return 1; };
fn1 == fn1; // true
fn1 == fn2; // false
```
