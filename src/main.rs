use std::env;
use std::fs;
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

/// Run a brainfuck source
pub fn run_source(source: &str) {
    let ast = build_ast(parse_source(source));
    let ast = optimize_ast(&ast);
    run_ast(
        &ast,
        &mut State {
            memory: [0; 30000],
            index: 0,
        },
    );
}

fn usage() {
    println!("brainfuck - A brainfuck compiler");
    println!("");
    println!("usage: brainfuck input_source");
    println!("");
    println!("    input_source    path to the input source");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut i = 1;
    let mut source_path = None;
    while i < args.len() {
        if args[i] == "-h" || args[i] == "--help" {
            usage();

            return;
        }

        if source_path.is_none() {
            source_path = Some(&args[i]);
            i += 1;
            continue;
        }

        i += 1;
    }

    if source_path.is_none() {
        println!("Error: missing input source file");
        usage();
        return;
    }

    let source_data = fs::read(source_path.unwrap()).unwrap();
    let source = str::from_utf8(&source_data).unwrap();
    run_source(source);
}
