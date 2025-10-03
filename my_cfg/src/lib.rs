use std::{fmt::Display, ops::Index, usize};

use serde::{Deserialize, Serialize};

use crate::loopified::{Relooper, StructuredNode};

mod loopified;

#[derive(Debug, Clone)]
pub struct Arena<T>(la_arena::Arena<T>);

impl<T> Arena<T> {
    pub fn alloc(&mut self, value: T) -> Idx<T> {
        Idx(self.0.alloc(value))
    }

    pub fn iter(&self) -> impl Iterator<Item = (Idx<T>, &T)> {
        self.0.iter().map(|x| (Idx(x.0), x.1))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Idx<T>, &mut T)> {
        self.0.iter_mut().map(|x| (Idx(x.0), x.1))
    }
}

impl<T> Index<Idx<T>> for Arena<T> {
    type Output = T;

    fn index(&self, index: Idx<T>) -> &Self::Output {
        &self.0[index.0]
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Serialize> Serialize for Arena<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.values().collect::<Vec<&T>>().serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Arena<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let v = Vec::<T>::deserialize(deserializer)?;
        Ok(Self(la_arena::Arena::from_iter(v)))
    }
}

#[derive(Debug)]
pub struct Idx<T>(la_arena::Idx<T>);

impl<T> std::hash::Hash for Idx<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> Eq for Idx<T> {}

impl<T> PartialEq for Idx<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Copy for Idx<T> {}

impl<T> Clone for Idx<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Idx<T> {
    pub fn from_usize(x: usize) -> Self {
        Self(la_arena::Idx::from_raw(la_arena::RawIdx::from_u32(
            x as u32,
        )))
    }

    pub fn to_usize(self) -> usize {
        self.0.into_raw().into_u32() as usize
    }
}

impl<T> Serialize for Idx<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.into_raw().into_u32().serialize(serializer)
    }
}

impl<'de, T> Deserialize<'de> for Idx<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Self(la_arena::Idx::from_raw(la_arena::RawIdx::from_u32(
            u32::deserialize(deserializer)?,
        ))))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CType {
    Void,
    Float(u8),
    Int(u8),
    UInt(u8),
    Bool,
    Ptr(Box<CType>),
}

impl Display for CType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CType::Void => write!(f, "void"),
            CType::Float(bytes) => write!(f, "f{}", bytes * 8),
            CType::Int(bytes) => write!(f, "i{}", bytes * 8),
            CType::UInt(bytes) => write!(f, "u{}", bytes * 8),
            CType::Bool => write!(f, "bool"),
            CType::Ptr(ctype) => write!(f, "*mut {ctype}"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Local {
    pub name: Option<String>,
    pub ty: CType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Place {
    Local(Idx<Local>),
    Deref(Box<Place>),
    Offset(Box<Place>, Box<Value>),
}

impl Place {
    pub fn as_local(&self) -> Option<Idx<Local>> {
        let Place::Local(p) = self else { return None };
        Some(*p)
    }

    pub fn replace_local(&self, l: Idx<Local>, my_value: Value) -> Value {
        match self {
            Place::Local(idx) => {
                if l == *idx {
                    return my_value;
                }
                return Value::Place(self.clone());
            }
            Place::Deref(place) => Value::Place(Place::Deref(Box::new(
                place.replace_local(l, my_value).as_place().unwrap().clone(),
            ))),
            Place::Offset(place, value) => Value::Place(Place::Offset(
                Box::new(
                    place
                        .replace_local(l, my_value.clone())
                        .as_place()
                        .unwrap()
                        .clone(),
                ),
                Box::new(value.replace_local(l, my_value)),
            )),
        }
    }
}

impl Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Place::Local(idx) => write!(f, "_{}", idx.to_usize()),
            Place::Deref(place) => write!(f, "*{place}"),
            Place::Offset(place, value) => {
                write!(f, "{place}.offset({value})")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Binop {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    Le,
    
}

impl Display for Binop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Binop::Add => write!(f, "+"),
            Binop::Sub => write!(f, "-"),
            Binop::Mul => write!(f, "*"),
            Binop::Div => write!(f, "/"),
            Binop::Lt => write!(f, "<"),
            Binop::Le => write!(f, "<="),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Value {
    Place(Place),
    Literal(i32),
    Binop(Box<Value>, Binop, Box<Value>),
}

impl Value {
    pub fn as_place(&self) -> Option<&Place> {
        let Value::Place(p) = self else { return None };
        Some(p)
    }

    pub fn as_literal(&self) -> Option<i32> {
        let Value::Literal(p) = self else { return None };
        Some(*p)
    }

    pub fn from_local(l: Idx<Local>) -> Value {
        Value::Place(Place::Local(l))
    }

    pub fn replace_local(&self, l: Idx<Local>, my_value: Value) -> Value {
        // println!("old {self} replace {l:?} {my_value}");
        let result = match self {
            Value::Place(place) => place.replace_local(l, my_value),
            Value::Literal(l) => Value::Literal(*l),
            Value::Binop(value1, binop, value2) => Value::Binop(
                Box::new(value1.replace_local(l, my_value.clone())),
                *binop,
                Box::new(value2.replace_local(l, my_value)),
            ),
        };
        // println!("result {result}");
        result
    }

    pub fn has_local(&self, l: Idx<Local>) -> bool {
        let invalid_local = Idx::from_usize(usize::MAX);
        self.replace_local(l, Value::Place(Place::Local(invalid_local))) != *self
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Place(place) => write!(f, "{place}"),
            Value::Literal(i) => write!(f, "{i}"),
            Value::Binop(value1, binop, value2) => {
                write!(f, "{value1} {binop} {value2}")
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Stmt {
    Assign { place: Place, value: Value },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Terminator {
    Return,
    Goto { bb: Idx<BasicBlock> },
    If { cond: Value, then: Idx<BasicBlock>, else_: Idx<BasicBlock> },
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BasicBlock {
    pub stmts: Vec<Stmt>,
    pub terminator: Option<Terminator>,
}
impl BasicBlock {
    pub fn terminator(&self) -> &Terminator {
        match &self.terminator {
            Some(t) => t,
            None => panic!("Terminator is not constructed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Cfg {
    pub locals: Arena<Local>,
    pub bb: Arena<BasicBlock>,
}

impl Cfg {
    pub fn from_json(x: &str) -> Cfg {
        serde_json::from_str(x).unwrap()
    }

    pub fn loopify(&self) -> StructuredNode {
        let relooper = Relooper::new(self);
        relooper.reloop(Idx::from_usize(0))
    }

    pub fn print(&self) {
        println!("fn sub {{");
        for (idx, local) in self.locals.iter() {
            println!("    let _{}: {}", idx.to_usize(), local.ty)
        }
        for (idx, bb) in self.bb.iter() {
            println!("    bb{}: {{", idx.to_usize());
            for stmt in &bb.stmts {
                match stmt {
                    Stmt::Assign { place, value } => {
                        println!("        {place} = {value};");
                    }
                }
            }
            match &bb.terminator {
                Some(terminator) => {
                    match terminator {
                        Terminator::Return => println!("        return;"),
                        Terminator::Goto { bb } => {
                            println!("        goto bb{};", bb.to_usize());
                        },
                        Terminator::If { cond, then, else_ } => {
                            println!("        if {cond} {{ goto bb{} }} else {{ goto bb{} }}", then.to_usize(), else_.to_usize());
                        },
                    }
                },
                None => println!("        <incomplete mir terminator>"),
            }
            println!("    }}");
        }
        println!("}}");
    }
}
