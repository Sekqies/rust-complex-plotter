use std::thread::current;
use std::{collections::HashMap, collections::HashSet};
use std::sync::OnceLock;
use std::rc::Rc;
static SYMBOL_MAPPER: OnceLock<HashMap<&'static str, Symbol>> = OnceLock::new();
#[derive(PartialEq, Eq,Clone,Debug)]
pub enum Symbol{
    NULLSYMBOL,
    ADD,
    SUBTRACT,
    NEGATE,
    MULTIPLY,
    DIVIDE,
    SIN,
    COS,
    CONSTANT(String),
    VARIABLE(String),
    DERIVATIVE,
    NUMERICAL_DERIVATIVE,
    UNKNOWN
}
#[derive(Debug,Clone)]
pub struct IndexNode{
    children:Vec<usize>,
    symbol:Symbol,
}
#[derive(Debug)]
pub struct SyntaxNode{
    symbol: Symbol,
    children: Vec<Rc<SyntaxNode>>,
}

#[derive(Debug)]
pub struct SyntaxTree{
    head: usize,
    arena: Vec<IndexNode>
}



impl IndexNode{
    fn add_child(&mut self, child:usize){
        self.children.push(child);
    }
}





fn break_into_symbols(expression:&String)->Vec<String>{
    let mut out:Vec<String> = Vec::new();
    let separators:Vec<char> = [',','(',')',';'].into();
    let mut last_separator_index = 0;

    for (i,c) in expression.char_indices(){
        if separators.contains(&c){
            let token_slice = &expression[last_separator_index..i];

            if !token_slice.trim().is_empty(){
                out.push(token_slice.to_string());
            }
            if c=='(' || c== ')' {
                out.push(c.to_string());
            }
                last_separator_index = i + c.len_utf8();
            
        }
    }

    let final_token = &expression[last_separator_index..];
    if !final_token.trim().is_empty(){
        out.push(final_token.to_string());
    }
    
    return out;
}


fn get_symbol_mapper() -> &'static HashMap<&'static str, Symbol> {
    SYMBOL_MAPPER.get_or_init(|| {
        HashMap::from([
            ("add", Symbol::ADD),
            ("sin", Symbol::SIN),
            ("cos", Symbol::COS),

        ])
    })
}

fn parse_symbol(symbol: &String) -> Symbol {
    let possible_symbol = build_value(symbol);
    if possible_symbol != Symbol::UNKNOWN{
        return possible_symbol;
    }
    get_symbol_mapper()
        .get(symbol.as_str())
        .cloned() 
        .unwrap_or(Symbol::UNKNOWN) 
}

fn build_value(symbol:&String) -> Symbol{
    let constant_list = HashSet::from([
        "e","pi"
    ]);
    let variable_list = HashSet::from([
        "x","y","z"
    ]);
    if(symbol.parse::<f64>().is_ok()|| constant_list.contains(symbol.as_str())){
        return Symbol::CONSTANT(symbol.clone());
    }
    if(variable_list.contains(symbol.as_str())){
        return Symbol::VARIABLE(symbol.clone());    
    }
    return Symbol::UNKNOWN;
}

fn is_value(symbol:&Symbol) -> bool{
    matches!(symbol,Symbol::CONSTANT(_)) || matches!(symbol,Symbol::VARIABLE(_))
}


fn hierarchize(symbols: &Vec<String>) -> SyntaxTree {
    let mut arena: Vec<IndexNode> = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    let mut head_index:usize = 0;
    for symbol in symbols{
        if symbol == "(" {continue}
        if symbol == ")" {
            stack.pop();
            continue;
        }
        let mut function = parse_symbol(symbol);
        let current_node = IndexNode{symbol:function.clone(), children: Vec::new()};
        arena.push(current_node);
        let current_index = arena.len() - 1;
        match stack.last(){
            Some(&parent_index) => {
                arena[parent_index].children.push(current_index);
                if !is_value(&function) {
                    stack.push(current_index);
                }
            }
            None => {
                head_index = current_index;
                if !is_value(&function) {
                    stack.push(current_index);
                }
            }
        }
    }
    SyntaxTree{head:head_index, arena: arena}
}

fn derivative(node:&Rc<SyntaxNode>) -> Rc<SyntaxNode>{
    let mut out = SyntaxNode{
        children: vec![],
        symbol: Symbol::NULLSYMBOL
    };
    match &node.symbol{
        Symbol::ADD | Symbol::SUBTRACT => {
            out.symbol = node.symbol.clone();
            for child in node.children.iter(){
                out.children.push(derivative(child));
            }
        },
        Symbol::MULTIPLY => {
            out.symbol = Symbol::ADD;
            let u = node.children[0].clone();
            let v = node.children[1].clone();
            let l = SyntaxNode{
                symbol: Symbol::MULTIPLY,
                children: vec![derivative(&u),v.clone()]
            };
            let r = SyntaxNode{
                symbol: Symbol::MULTIPLY,
                children: vec![u.clone(),derivative(&v)]
            };
            out.children = Vec::from([Rc::new(l),Rc::new(r)]);
        },
        Symbol::DIVIDE => {
            let u = node.children[0].clone();
            let v = node.children[1].clone();        
            let l = SyntaxNode{
                symbol: Symbol::MULTIPLY,
                children: vec![derivative(&u),v.clone()]
            };
            let r = SyntaxNode{
                symbol: Symbol::MULTIPLY,
                children: vec![u.clone(),derivative(&v)]
            };       
            let numerator = SyntaxNode {
                symbol: Symbol::SUBTRACT,
                children: vec![Rc::new(l),Rc::new(r)]
            };
            let denominator = SyntaxNode{
                symbol: Symbol::MULTIPLY,
                children: vec![v.clone(),v.clone()]
            };
            out.symbol = Symbol::DIVIDE;
            out.children = vec![Rc::new(numerator),Rc::new(denominator)];
        },
        Symbol::SIN => {
            out.symbol = Symbol::MULTIPLY;
            let inner = node.children[0].clone();
            let l = SyntaxNode{
                symbol: Symbol::COS,
                children: vec![inner.clone()]
            };
            out.children = vec![Rc::new(l),derivative(&inner)];
        },
        Symbol::COS => {

        }
        Symbol::CONSTANT(_) => {
            out.symbol = Symbol::CONSTANT("0".to_string());
        },
        Symbol::VARIABLE(_) => {
            out.symbol = Symbol::CONSTANT("1".to_string());
        },
        tipo => {
            println!("fiz merda {:?}",tipo);
        }
    }
    println!("Derivada: {:?}",out.symbol.clone());
    Rc::new(out)
}



fn structure_tree(tree: &SyntaxTree) -> SyntaxNode {
    fn build_node(node_index: usize, arena: &Vec<IndexNode>) -> SyntaxNode {
        let arena_node = &arena[node_index];

        let children: Vec<Rc<SyntaxNode>> = arena_node
            .children
            .iter()
            .map(|child_index| Rc::new(build_node(*child_index, arena)))
            .collect();

        SyntaxNode {
            symbol: arena_node.symbol.clone(),
            children: children,
        }
    }

    if tree.arena.is_empty() {
        panic!("Árvore vazia");
    }

    build_node(tree.head, &tree.arena)
}

fn print_tree(node: &SyntaxNode) {
    print!("{:?}", node.symbol);

    if !is_value(&node.symbol) {
        print!("(");
        let mut children_iter = node.children.iter().peekable();
        while let Some(child) = children_iter.next() {
            print_tree(child);
            if children_iter.peek().is_some() {
                print!(", ");
            }
        }

        print!(")");
    }
}


fn main() {
    let function = "sin(add(sin(z),2))".into();
    let symbols = break_into_symbols(&function);
    let tree = hierarchize(&symbols);
    let head = structure_tree(&tree);
    let new_head = derivative(&Rc::new(head));
    print_tree(&structure_tree(&tree));
    println!();
    print_tree(&new_head);
}