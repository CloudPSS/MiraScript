#![allow(dead_code)]

use std::ops::Deref;

mod analyzer;
mod ansi;
mod emitter;
mod lexer;
mod parser;
mod utils;

fn main() {
    let text = r##"
    var name = "world"; //
    print <| "Hello ${name}";
    
    [1,2,3] |> filter(fn {it % 2 == 0}) |> String
    
    for i in 1..<3 {
        print <| i;
    }

    for i in [1,2,3] |> map(fn {it * 2}) {
        print <| i;
    }

    fn  { it = 1 ;};

    1

    call(1, 2, 3)
    call(a: 1, ..x)

    var simple_array = [1, 2, 3, "4", [5], []];
    var spread_array = [1, 2, 3, "4", ..[5]];
    var range_array = [1..2, 1..<3,]
    var array = [1,2,3, 7..8, ..[9,10,11..<20], ..7, ..()];

    var parenthesis = (1);
    var single_record = (1,);
    var single_record2 = (a: 1);
    var named_record = (hello: "world", foo: 1, bar: 2, 3 1 45.3 );
    var simple_record = ("hello", 1, 2 ,3);
    var spread_record = (..named_record, hello: "world", foo: 1, bar: 2,  ..simple_record);

    val a = (1 2)
    x.y = 12
    z = 1
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
