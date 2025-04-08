#![allow(dead_code)]

use std::ops::Deref;

mod analyzer;
mod ansi;
mod emitter;
mod lexer;
mod parser;
mod utils;

struct X(usize);

fn main() {
    let text = r##"{
    let mut name = "world"; //
    print <| "Hello ${let mut a = 1; "${a}name"}";
    
    [1,2,3] |> filter(fn {it % 2 == 0}) |> String
    
    for i in 1..<3 {
        print <| i;
    }

    let w = while a > 0 {
        a = a - 1;
    } else {
        a;
        10
    }
    
    let mut fr= for i in [1,2,3] |> map(fn {it * 2}) {
        print <| i;
    } else if a > 0 {
        a = a - 1;
    } 

    fn (a,b,c) {
        print <| {
            let mut sum = a + b + c;
            sum
        };
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
    let mut named_record = (hello: "world", foo: 1, bar: 2, 3 1 45.3 );
    let mut simple_record = ("hello", 1, 2 ,3);
    let mut spread_record = (..named_record, hello: "world", foo: 1, bar: 2,  ..simple_record);

    

    let a = (1 2){}
    x.y = 12
    z = 1
    {{{
     if a b else c
match {a}{}
    @@"你好${ if  } 世界 $$ 再见@" $$if "@世界"@@

    let t = typeof "x";
    "##;

    let mut input = lexer::to_input(text);
    let result = lexer::lex(&mut input, true).unwrap();
    println!("{:?}", result);

    let mut input = parser::to_input(&result);
    let exp = parser::parse(&mut input);

    println!("{:?}", input.deref());
    println!("{:#?}", exp.as_ref().unwrap());
    println!("{:#}", exp.as_ref().unwrap());
}
