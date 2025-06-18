/* eslint-disable no-console */
import './index.css';
import * as monaco from '@private/monaco-editor';
import { KeyCode, KeyMod } from '@private/monaco-editor';
import { registerMiraScript } from '@mirascript/monaco';
import {
    type VmAny,
    VmExtern,
    createVmGlobal,
    isVmExtern,
    isVmModule,
    compile,
    serialize,
    type InputMode,
} from 'mirascript';

const globals = createVmGlobal(
    {
        e: new VmExtern([1, 2, [1, 2], { x: 0 }]),
        o: { a: [], b: 1, c: '2', d: { e: 3 } },
        x: [1, 2, 3],
        s: 'sdaf\ndsaf\\n'.repeat(10000),
    },
    {
        globalThis,
    },
);
registerMiraScript(monaco, () => globals);

let mode: InputMode = (localStorage.getItem('mode') as InputMode) || 'Script';
const createModel = (value: string) =>
    monaco.editor.createModel(
        value,
        mode === 'Template' ? 'mirascript-template' : 'mirascript',
        monaco.Uri.parse(`file:///${mode}.mira`),
    );

const value =
    localStorage.getItem('source') ||
    `{#
"\\u{aaa}"
let mut name = "world"; // comment
print("Hello \${let mut /** comment in string */a = 1; // comment in string
"\${a}name"}");

[1,2,3]::filter(fn {it % 2 == 0})::String()

for i in 1..<3 {
print(i);
} else {
}

let w = while a > 0 {
a = a - 1;
} else {
a;
10
}

let mut fr= for i in [1,2,3]::map(fn {it * 2}) {
print(i);
} else if a > 0 {
a = a - 1;
} 

fn (a,b,c) {
print({
    let mut sum = a + b + c;
    sum
});
};

1

call(1, if x {1}else{2;3}, {x;y})
call(a: 1, ..x)

match x {
case 1  x 
case 2 { y;3 }
 _  4 
}

let mut simple_array = [1, 2, {1;2;3}, "4", [5], []];
let mut spread_array = [1, 2, 3, "4", ..[5]];
let mut range_array = [1..2, 1..<3,]
let mut array = [1,2,3, 7..8, ..[9,10,11..<20], ..7, ..()];

let mut parenthesis = (1);
let mut single_record = (1,);
let mut single_record2 = (a: 1);
let mut ordinal_record = (1: 1);
let mut named_record = (hello: "world", foo: 1, bar: 2, 3 1 45.3 );
let mut simple_record = ("hello", 1, 2 ,3);
let mut spread_record = (..named_record, hello: "world", foo: 1, bar: 2,  ..simple_record);



let a = (1 2){}
x.y = 12
mut z 
{{{
if a b else c
match {a}{}
@@"你好\${ if  } 世界 $$ 再见@" $$if "@世界"@@

let t = type "x";

if x is y {
print("x is 1");
}
if "1" in ["1", "2", "3"] {
print("1 in [1, 2, 3]");
}
if "key" in (key: "value") {
print("key in (key: \\"value\\")");
}
if a {} else for global in (1,2e7, 1.53e-12, : s)
let (x, mut y) = 1;}}}

for (key, mut v,:mut e, _,:12,) in entries((1,2,3)) {
a += 12;
print(a);
}
let (a) = (1);
a.1 = 2;
a = (0: "x", 1:112, "a":u);
(00, 0, 0x0, 0o0, 0b0, 0e0, 0.0)

let (.._, _, ..\`ht\`, ..x, y, ..(1,2,3)) = x;

a!==1;
a!~=1;
a! ~=1;
a!.b!.c!['x']!(12)! 
a!.b!['x']!.c!(12);

a ?? 1;
a ??= 1;
2147483647 + 2147483648 
(z?:a, ?:b) = (:a, :b);

if a is not x {
print("a is not nan");
}

/*1*/ [/*2*/x/*3*/,/*4*/ /*5*/.., y, ..[], (+5..8), ..(not(> 1 or +nan),)] = [1, ..[], 5..8];}}}}
(if a {1} else {2}) - 3;
if a {1} else {2} - 3;
3- if a {1} else {2};
x;
(+1,_,mut a) = (1,2,3);
((fn { print(it) }))("END");


/* EOF 
dd   */ 
(  a, b, [1, ..], ..) = x;
a::map.1.xx()::(fn{})()::a['x']::b(); type(e); e::type();\

@$_123;
`;
const elEditor = document.querySelector<HTMLDivElement>('#editor')!;
const overlay = monaco.utils.createOverflowWidgetsDomNode(elEditor);
const editor = monaco.editor.create(elEditor, {
    fontFamily: 'Sarasa Mono SC',
    useShadowDOM: true,
    overflowWidgetsDomNode: overlay,
    formatOnType: true,
    formatOnPaste: true,
    fontLigatures: true,
    automaticLayout: true,
    theme: 'vs-dark',
    'semanticHighlighting.enabled': true,
    model: createModel(value),
});
editor.onDidDispose(() => overlay.dispose());
setTimeout(() => {
    editor.addAction({
        id: 'SwitchMode',
        label: 'Switch Mode',
        keybindings: [KeyMod.CtrlCmd | KeyCode.KeyM],
        run: () => {
            mode = mode === 'Script' ? 'Template' : 'Script';
            localStorage.setItem('mode', mode);
            const oldModel = editor.getModel();
            editor.setModel(createModel(editor.getValue()));
            oldModel?.dispose();
        },
    });
    editor.onDidChangeModelContent(() => {
        localStorage.setItem('source', editor.getValue());
    });
}, 1);

/** 将值转为显示 */
function print(value: VmAny | Error): string {
    if (value === null) return 'nil';
    if (value === undefined) return '<uninitialized>';
    if (value instanceof Error) return value.toString();
    if (isVmExtern(value) || isVmModule(value) || typeof value == 'function') {
        return String(value);
    }
    return serialize(value);
}

const elDisassembly = document.querySelector<HTMLDivElement>('#disassembly')!;
elDisassembly.addEventListener('contextmenu', (e) => {
    e.preventDefault();
    void run();
});

/** 编译运行 */
async function run() {
    const value = editor.getValue();

    console.time('transpile');
    const result = await compile(value, {
        pretty: true,
        input_mode: mode,
        sourceMap: true,
        fileName: 'xx/yy.mira',
    }).finally(() => {
        console.timeEnd('transpile');
    });
    let content = result.toString();

    // const COUNT = 1000;
    // const start = performance.now();
    // // await Promise.all(Array.from({ length: COUNT }, () => compile(value, { pretty: true, mode })));
    // for (let i = 0; i < COUNT; i++) {
    //     await compile(value, { pretty: true, input_mode: mode });
    // }
    // console.log(`Compile benchmark: ${((performance.now() - start) / COUNT) * 1000}us`);

    console.time('execute');
    try {
        const ret = result(globals);
        content += `\nResult:\n  ${print(ret)}`;
    } catch (ex) {
        content += `\n${String(ex)}`;
    }
    console.timeEnd('execute');
    elDisassembly.textContent = content;
}
