use std::env;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::str;

/// A brainfuck token
#[derive(PartialEq)]
pub enum Token {
    Incr,      // "+"
    Decr,      // "-"
    MoveLeft,  // "<"
    MoveRight, // ">"
    Write,     // "."
    LoopBegin, // "["
    LoopEnd,   // "]"
}

/// Parse a source string and extract tokens
pub fn parse_source(source: &str) -> impl Iterator<Item = Token> + '_ {
    source.chars().filter_map(|c| match c {
        '+' => Some(Token::Incr),
        '-' => Some(Token::Decr),
        '<' => Some(Token::MoveLeft),
        '>' => Some(Token::MoveRight),
        '.' => Some(Token::Write),
        '[' => Some(Token::LoopBegin),
        ']' => Some(Token::LoopEnd),
        _ => None,
    })
}

/// A node of an Abstract Syntax Tree
#[derive(Clone, Debug)]
pub enum Node {
    Incr(isize),      // Increment instruction
    Move(isize),      // Move instruction
    Write,            // Write instruction
    Loop(Box<Node>),  // Loop instruction
    Block(Vec<Node>), // A container for nodes
}

pub fn build_ast(tokens: impl IntoIterator<Item = Token>) -> Node {
    let mut operations = vec![];
    let mut stack = vec![];
    for token in tokens {
        match token {
            Token::Decr => {
                operations.push(Node::Incr(-1));
            }
            Token::Incr => {
                operations.push(Node::Incr(1));
            }
            Token::MoveLeft => {
                operations.push(Node::Move(-1));
            }
            Token::MoveRight => {
                operations.push(Node::Move(1));
            }
            Token::Write => {
                operations.push(Node::Write);
            }
            Token::LoopBegin => {
                stack.push(operations);
                operations = vec![];
            }
            Token::LoopEnd => {
                let instruction = Node::Loop(Box::new(Node::Block(operations)));
                operations = stack.pop().unwrap();
                operations.push(instruction);
            }
        }
    }

    // Optimize output
    if operations.len() == 1 {
        operations[0].clone()
    } else {
        Node::Block(operations)
    }
}

fn optimize_ast(ast: &Node) -> Node {
    match ast {
        Node::Incr(val) => {
            if *val == 0 {
                Node::Block(vec![])
            } else {
                ast.clone()
            }
        }
        Node::Move(val) => {
            if *val == 0 {
                Node::Block(vec![])
            } else {
                ast.clone()
            }
        }
        Node::Write => ast.clone(),
        Node::Loop(node) => Node::Loop(Box::new(optimize_ast(&node))),
        Node::Block(nodes) => {
            // Optimize each nodes individually
            let mut new_nodes = vec![];
            for node in nodes.iter() {
                let opt_node = optimize_ast(&node);

                // Try to merge incr nodes
                if let Node::Incr(val) = opt_node {
                    if let Some(Node::Incr(last_val)) = new_nodes.last_mut() {
                        *last_val += val;
                    } else {
                        new_nodes.push(opt_node);
                    }
                }
                // Try to merge move nodes
                else if let Node::Move(val) = opt_node {
                    if let Some(Node::Move(last_val)) = new_nodes.last_mut() {
                        *last_val += val;
                    } else {
                        new_nodes.push(opt_node);
                    }
                } else {
                    new_nodes.push(opt_node);
                }
            }
            let nodes = new_nodes;

            if nodes.len() == 1 {
                nodes[0].clone()
            } else {
                Node::Block(nodes)
            }
        }
    }
}

pub fn compile_source(source: &str) -> Node {
    let ast = build_ast(parse_source(source));
    let ast = optimize_ast(&ast);

    ast
}

/// State of the brainfuck VM
pub struct State {
    pub memory: [u8; 30000],
    pub index: usize,
}

/// Run an AST in the brainfuck VM
pub fn run_ast(node: &Node, state: &mut State) {
    match node {
        Node::Incr(val) => {
            state.memory[state.index] = (state.memory[state.index] as isize + val) as u8;
        }
        Node::Move(val) => {
            state.index = (state.index as isize + val) as usize;
        }
        Node::Write => {
            print!("{}", state.memory[state.index] as char);
        }
        Node::Loop(sub_node) => {
            while state.memory[state.index] != 0 {
                run_ast(sub_node.as_ref(), state);
            }
        }
        Node::Block(sub_nodes) => {
            for sub_node in sub_nodes.iter() {
                run_ast(sub_node, state);
            }
        }
    }
}

fn write_bf(ast: &Node, write: &mut dyn Write) {
    match ast {
        Node::Incr(val) => {
            for _ in 0..val.abs() {
                if *val < 0 {
                    write.write(b"-").unwrap();
                } else {
                    write.write(b"+").unwrap();
                }
            }
        }
        Node::Move(val) => {
            for _ in 0..val.abs() {
                if *val < 0 {
                    write.write(b"<").unwrap();
                } else {
                    write.write(b">").unwrap();
                }
            }
        }
        Node::Write => {
            write.write(b".").unwrap();
        }
        Node::Loop(node) => {
            write.write(b"[").unwrap();
            write_bf(&node, write);

            write.write(b"]").unwrap();
        }
        Node::Block(nodes) => {
            for node in nodes.iter() {
                write_bf(&node, write);
            }
        }
    }
}

fn write_c_ast(ast: &Node, write: &mut dyn Write) {
    match ast {
        Node::Incr(val) => {
            write
                .write(format!("    memory[index] += {};\n", val).as_bytes())
                .unwrap();
        }

        Node::Move(val) => {
            write
                .write(format!("    index += {};\n", val).as_bytes())
                .unwrap();
        }
        Node::Write => {
            write
                .write(b"    printf(\"%c\", memory[index]);\n")
                .unwrap();
        }
        Node::Loop(node) => {
            write.write(b"    while (memory[index] != 0) {\n").unwrap();
            write_c_ast(node, write);
            write.write(b"    }").unwrap();
        }
        Node::Block(nodes) => {
            for node in nodes.into_iter() {
                write_c_ast(node, write);
            }
        }
    }
}

fn write_c(ast: &Node, write: &mut dyn Write) {
    write.write(b"#include <stdint.h>\n").unwrap();
    write.write(b"#include <stdio.h>\n").unwrap();
    write.write(b"#include <stdlib.h>\n").unwrap();
    write.write(b"\n").unwrap();
    write
        .write(b"int main(int argc, char ** argv) {\n")
        .unwrap();
    write.write(b"    uint8_t memory[30000] = {0};\n").unwrap();
    write.write(b"    size_t index = 0;\n").unwrap();
    write.write(b"\n").unwrap();
    write.write(b"    // bf source code\n").unwrap();
    write_c_ast(ast, write);
    write.write(b"\n").unwrap();
    write.write(b"\n").unwrap();
    write.write(b"    return EXIT_SUCCESS;\n").unwrap();
    write.write(b"}\n").unwrap();
}

fn write_rust_ast(ast: &Node, write: &mut dyn Write) {
    match ast {
        Node::Incr(val) => {
            write
                .write(
                    format!(
                        "    memory[index] = (memory[index] as isize + {}) as u8;\n",
                        val
                    )
                    .as_bytes(),
                )
                .unwrap();
        }

        Node::Move(val) => {
            write
                .write(format!("    index = (index as isize + {}) as usize;\n", val).as_bytes())
                .unwrap();
        }
        Node::Write => {
            write
                .write(b"    print!(\"{}\", memory[index] as char);\n")
                .unwrap();
        }
        Node::Loop(node) => {
            write.write(b"    while memory[index] != 0 {\n").unwrap();
            write_rust_ast(node, write);
            write.write(b"    }").unwrap();
        }
        Node::Block(nodes) => {
            for node in nodes.into_iter() {
                write_rust_ast(node, write);
            }
        }
    }
}

fn write_rust(ast: &Node, write: &mut dyn Write) {
    write.write(b"fn main() {\n").unwrap();
    write
        .write(b"    let mut memory: [u8; 30000] = [0; 30000];\n")
        .unwrap();
    write.write(b"    let mut index: usize = 0;\n").unwrap();
    write.write(b"\n").unwrap();
    write.write(b"    // bf source code\n").unwrap();
    write_rust_ast(ast, write);
    write.write(b"}\n").unwrap();
}

fn usage() {
    println!("brainfuck - A brainfuck compiler");
    println!("");
    println!("usage: brainfuck options... input_source [output_file]");
    println!("");
    println!("    -e, --eval      evaluate the source code");
    println!("    input_source    path to the input source");
    println!("    output_file     path to the output file, if needed");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    let mut source_path = None;
    let mut output_path = None;
    let mut evaluate = false;
    while i < args.len() {
        if args[i] == "-h" || args[i] == "--help" {
            usage();

            return;
        }

        if args[i] == "-e" || args[i] == "--eval" {
            evaluate = true;
            i += 1;
            continue;
        }

        if source_path.is_none() {
            source_path = Some(&args[i]);
            i += 1;
            continue;
        }

        if output_path.is_none() {
            output_path = Some(&args[i]);
            i += 1;
            continue;
        }

        i += 1;
    }

    // Read the input source
    let source_data = fs::read(source_path.unwrap()).unwrap();
    let source = str::from_utf8(&source_data).unwrap();

    // Compile the source
    let ast = compile_source(source);

    // Run the program, if needed
    if evaluate {
        run_ast(
            &ast,
            &mut State {
                index: 0,
                memory: [0; 30000],
            },
        );
    }

    // Output the program
    if let Some(path) = output_path {
        let path = PathBuf::from(path);
        let extension = path.extension().unwrap();
        match extension.to_str().unwrap() {
            "bf" => {
                let mut file = File::create(path).unwrap();
                write_bf(&ast, &mut file);
            }
            "c" => {
                let mut file = File::create(path).unwrap();
                write_c(&ast, &mut file);
            }
            "rs" => {
                let mut file = File::create(path).unwrap();
                write_rust(&ast, &mut file);
            }
            _ => panic!("unsupported extension {:?}", extension),
        };
    }
}
