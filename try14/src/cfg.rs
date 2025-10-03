use la_arena::{Arena, Idx};

enum CType {
    Void,
    Float(u8),
    Ptr(Box<CType>),
}

struct Local {
    name: Option<String>,
    ty: CType,
}

enum Place {
    Local(Idx<Local>),
    Deref(Box<Place>),
    Offset(Box<Place>, Box<Value>),
}

enum Binop {
    Add,
    Sub,
}

enum Value {
    Place(Place),
    Literal(i32),
    Binop(Box<Value>, Binop, Box<Value>),
}

enum Stmt {
    Assign {
        place: Place,
        value: Value,
    }
}

enum Terminator {
    Return,
}

struct BasicBlock {
    stmts: Vec<Stmt>,
    terminator: Option<Terminator>,
}

struct Cfg {
    locals: Arena<Local>,
    bb: Arena<BasicBlock>,
}