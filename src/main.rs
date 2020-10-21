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
    println!("{:?}", ast);
    let ast = optimize_ast(&ast);
    println!("{:?}", ast);
    run_ast(
        &ast,
        &mut State {
            memory: [0; 30000],
            index: 0,
        },
    );
}

fn main() {
    let source = "
++++++++               Set Cell #0 to 8
[
    >++++               Add 4 to Cell #1; this will always set Cell #1 to 4
    [                   as the cell will be cleared by the loop
        >++             Add 2 to Cell #2
        >+++            Add 3 to Cell #3
        >+++            Add 3 to Cell #4
        >+              Add 1 to Cell #5
        <<<<-           Decrement the loop counter in Cell #1
    ]                   Loop till Cell #1 is zero; number of iterations is 4
    >+                  Add 1 to Cell #2
    >+                  Add 1 to Cell #3
    >-                  Subtract 1 from Cell #4
    >>+                 Add 1 to Cell #6
    [<]                 Move back to the first zero cell you find; this will
                        be Cell #1 which was cleared by the previous loop
    <-                  Decrement the loop Counter in Cell #0
]                       Loop till Cell #0 is zero; number of iterations is 8

The result of this is:
Cell No :   0   1   2   3   4   5   6
Contents:   0   0  72 104  88  32   8
Pointer :   ^

>>.                     Cell #2 has value 72 which is 'H'
>---.                   Subtract 3 from Cell #3 to get 101 which is 'e'
+++++++..+++.           Likewise for 'llo' from Cell #3
>>.                     Cell #5 is 32 for the space
<-.                     Subtract 1 from Cell #4 for 87 to give a 'W'
<.                      Cell #3 was set to 'o' from the end of 'Hello'
+++.------.--------.    Cell #3 for 'rl' and 'd'
>>+.                    Add 1 to Cell #5 gives us an exclamation point
>++.                    And finally a newline from Cell #6
";
    //println!("// {}", source);
    run_source(source);
}
