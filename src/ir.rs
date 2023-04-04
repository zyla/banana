#![allow(clippy::needless_borrow)]

use derive_new::new;
use ordered_float::OrderedFloat;

// ANCHOR: input
#[salsa::input]
pub struct SourceProgram {
    #[return_ref]
    pub text: String,
}
// ANCHOR_END: input

// ANCHOR: interned_ids
#[salsa::interned]
pub struct VariableId {
    #[return_ref]
    pub text: String,
}

#[salsa::interned]
pub struct FunctionId {
    #[return_ref]
    pub text: String,
}

#[salsa::interned]
pub struct DefId {
    pub data: DefIdData,
}

impl DefId {
    pub fn unknown(db: &dyn crate::Db) -> Self {
        Self::new(db, DefIdData::Unknown)
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug)]
pub enum DefIdData {
    Unknown,
    Function(FunctionId),
}
// ANCHOR_END: interned_ids

// ANCHOR: program
#[salsa::tracked]
pub struct Program {
    #[return_ref]
    pub functions: Vec<Function>,
}
// ANCHOR_END: program

// ANCHOR: statements_and_expressions
#[derive(Eq, PartialEq, Debug, Hash, new)]
pub struct Statement {
    pub span: Span,

    pub data: StatementData,
}

impl Visit for Statement {
    fn traverse<V: Visitor>(&mut self, db: &dyn crate::Db, v: &mut V) {
        v.visit_span(&mut self.span);
        self.data.traverse(db, v);
    }
}

#[derive(Eq, PartialEq, Debug, Hash)]
pub enum StatementData {
    /// Defines `fn <name>(<args>) = <body>`
    Function {
        name: FunctionId,
        data: FunctionData,
    },
    /// Defines `print <expr>`
    Print(Expression),
}

impl Visit for StatementData {
    fn traverse<V: Visitor>(&mut self, db: &dyn crate::Db, v: &mut V) {
        match self {
            Self::Function { data, .. } => data.traverse(db, v),
            Self::Print(x) => x.traverse(db, v),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Hash, new)]
pub struct Expression {
    pub span: Span,

    pub data: ExpressionData,
}

impl Visit for Expression {
    fn traverse<V: Visitor>(&mut self, db: &dyn crate::Db, v: &mut V) {
        v.visit_expr(self);
        v.visit_span(&mut self.span);
        self.data.traverse(db, v);
    }
}

#[derive(Eq, PartialEq, Debug, Hash)]
pub enum ExpressionData {
    Op(Box<Expression>, Op, Box<Expression>),
    Number(OrderedFloat<f64>),
    Variable(VariableId),
    Call(FunctionId, Vec<Expression>),
}

impl Visit for ExpressionData {
    fn traverse<V: Visitor>(&mut self, db: &dyn crate::Db, v: &mut V) {
        match self {
            Self::Op(l, _, r) => {
                l.traverse(db, v);
                r.traverse(db, v);
            }
            Self::Number(_) => {}
            Self::Variable(_) => {}
            Self::Call(_, args) => {
                args.traverse(db, v);
            }
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub enum Op {
    Add,
    Subtract,
    Multiply,
    Divide,
}
// ANCHOR_END: statements_and_expressions

// ANCHOR: functions
#[salsa::tracked]
pub struct Function {
    #[id]
    pub name: FunctionId,

    #[return_ref]
    pub data: FunctionData,
}

#[derive(Eq, PartialEq, Hash, Debug, new)]
pub struct FunctionData {
    pub name_span: Span,

    pub args: Vec<VariableId>,

    pub body: Expression,
}
// ANCHOR_END: functions

impl Visit for FunctionData {
    fn traverse<V: Visitor>(&mut self, db: &dyn crate::Db, v: &mut V) {
        self.name_span.traverse(db, v);
        self.body.traverse(db, v);
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug, new)]
pub struct Span {
    pub id: DefId,
    pub start: usize,
    pub end: usize,
}

impl Visit for Span {
    fn traverse<V: Visitor>(&mut self, _: &dyn crate::Db, v: &mut V) {
        v.visit_span(self);
    }
}

// ANCHOR: diagnostic
#[salsa::accumulator]
pub struct Diagnostics(Diagnostic);

#[derive(new, Clone, Debug)]
pub struct Diagnostic {
    pub start: usize,
    pub end: usize,
    pub message: String,
}
// ANCHOR_END: diagnostic

pub trait Visitor {
    fn visit_statement(&mut self, _: &mut Statement) {}
    fn visit_expr(&mut self, _: &mut Expression) {}
    fn visit_span(&mut self, _: &mut Span) {}
}

pub trait Visit {
    fn traverse<V: Visitor>(&mut self, db: &dyn crate::Db, v: &mut V);
}

impl<T: Visit> Visit for Vec<T> {
    fn traverse<V: Visitor>(&mut self, db: &dyn crate::Db, v: &mut V) {
        for x in self {
            x.traverse(db, v);
        }
    }
}
