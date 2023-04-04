#![allow(dead_code)]

use crate::ir::{DefId, DefIdData, Visit, Visitor};
use salsa::debug::DebugWithDb;

use crate::ir::{
    Diagnostic, Diagnostics, Expression, ExpressionData, Function, FunctionId, Op, Program,
    SourceProgram, Span, Statement, StatementData, VariableId,
};

lalrpop_mod!(grammar);

struct RewriteSpans<'a> {
    db: &'a dyn crate::Db,
    start_offset: usize,
    def_id: DefId,
}

impl<'a> Visitor for RewriteSpans<'a> {
    fn visit_span(&mut self, span: &mut Span) {
        span.id = self.def_id;
        span.start -= self.start_offset;
        span.end -= self.start_offset;
    }
}

// ANCHOR: parse_statements
#[salsa::tracked]
pub fn parse_statements(db: &dyn crate::Db, source: SourceProgram) -> Program {
    // Get the source text from the database
    let source_text = source.text(db);

    match grammar::ProgramParser::new().parse(db, &source_text) {
        Ok(stmts) => Program::new(
            db,
            stmts
                .into_iter()
                .flat_map(|x| match x.data {
                    StatementData::Function { name, mut data } => {
                        data.traverse(
                            db,
                            &mut RewriteSpans {
                                db,
                                start_offset: x.span.start,
                                def_id: DefId::new(db, DefIdData::Function(name)),
                            },
                        );

                        eprintln!("{} {:#?}", name.text(db), data);

                        Some(Function::new(db, name, data))
                    }
                    _ => None,
                })
                .collect::<Vec<_>>(),
        ),
        Err(err) => {
            Diagnostics::push(
                db,
                Diagnostic {
                    start: 0,
                    end: 0,
                    message: format!("{err}"),
                },
            );
            Program::new(db, vec![])
        }
    }
}
// ANCHOR_END: parse_statements

// ANCHOR: parse_string
/// Create a new database with the given source text and parse the result.
/// Returns the statements and the diagnostics generated.
#[cfg(test)]
fn parse_string(source_text: &str) -> String {
    // Create the database
    let db = crate::db::Database::default();

    // Create the source program
    let source_program = SourceProgram::new(&db, source_text.to_string());

    // Invoke the parser
    let statements = parse_statements(&db, source_program);

    // Read out any diagnostics
    let accumulated = parse_statements::accumulated::<Diagnostics>(&db, source_program);

    // Format the result as a string and return it
    format!("{:#?}", (statements.debug_all(&db), accumulated))
}
// ANCHOR_END: parse_string

// ANCHOR: parse_print
#[test]
fn parse_print() {
    let actual = parse_string("print 1 + 2;");
    let expected = expect_test::expect![[r#"
        (
            Program {
                [salsa id]: 0,
                statements: [
                    Statement {
                        span: Span(
                            Id {
                                value: 5,
                            },
                        ),
                        data: Print(
                            Expression {
                                span: Span(
                                    Id {
                                        value: 4,
                                    },
                                ),
                                data: Op(
                                    Expression {
                                        span: Span(
                                            Id {
                                                value: 1,
                                            },
                                        ),
                                        data: Number(
                                            OrderedFloat(
                                                1.0,
                                            ),
                                        ),
                                    },
                                    Add,
                                    Expression {
                                        span: Span(
                                            Id {
                                                value: 3,
                                            },
                                        ),
                                        data: Number(
                                            OrderedFloat(
                                                2.0,
                                            ),
                                        ),
                                    },
                                ),
                            },
                        ),
                    },
                ],
            },
            [],
        )"#]];
    expected.assert_eq(&actual);
}
// ANCHOR_END: parse_print

#[test]
fn parse_example() {
    let actual = parse_string(
        "
            fn area_rectangle(w, h) = w * h;
            fn area_circle(r) = 3.14 * r * r;
            print area_rectangle(3, 4);
            print area_circle(1);
            print 11 * 2;
        ",
    );
    let expected = expect_test::expect![[r#"
        (
            Program {
                [salsa id]: 0,
                statements: [
                    Statement {
                        span: Span(
                            Id {
                                value: 10,
                            },
                        ),
                        data: Function(
                            Function(
                                Id {
                                    value: 1,
                                },
                            ),
                        ),
                    },
                    Statement {
                        span: Span(
                            Id {
                                value: 22,
                            },
                        ),
                        data: Function(
                            Function(
                                Id {
                                    value: 2,
                                },
                            ),
                        ),
                    },
                    Statement {
                        span: Span(
                            Id {
                                value: 29,
                            },
                        ),
                        data: Print(
                            Expression {
                                span: Span(
                                    Id {
                                        value: 28,
                                    },
                                ),
                                data: Call(
                                    FunctionId(
                                        Id {
                                            value: 1,
                                        },
                                    ),
                                    [
                                        Expression {
                                            span: Span(
                                                Id {
                                                    value: 24,
                                                },
                                            ),
                                            data: Number(
                                                OrderedFloat(
                                                    3.0,
                                                ),
                                            ),
                                        },
                                        Expression {
                                            span: Span(
                                                Id {
                                                    value: 26,
                                                },
                                            ),
                                            data: Number(
                                                OrderedFloat(
                                                    4.0,
                                                ),
                                            ),
                                        },
                                    ],
                                ),
                            },
                        ),
                    },
                    Statement {
                        span: Span(
                            Id {
                                value: 34,
                            },
                        ),
                        data: Print(
                            Expression {
                                span: Span(
                                    Id {
                                        value: 33,
                                    },
                                ),
                                data: Call(
                                    FunctionId(
                                        Id {
                                            value: 2,
                                        },
                                    ),
                                    [
                                        Expression {
                                            span: Span(
                                                Id {
                                                    value: 31,
                                                },
                                            ),
                                            data: Number(
                                                OrderedFloat(
                                                    1.0,
                                                ),
                                            ),
                                        },
                                    ],
                                ),
                            },
                        ),
                    },
                    Statement {
                        span: Span(
                            Id {
                                value: 39,
                            },
                        ),
                        data: Print(
                            Expression {
                                span: Span(
                                    Id {
                                        value: 38,
                                    },
                                ),
                                data: Op(
                                    Expression {
                                        span: Span(
                                            Id {
                                                value: 35,
                                            },
                                        ),
                                        data: Number(
                                            OrderedFloat(
                                                11.0,
                                            ),
                                        ),
                                    },
                                    Multiply,
                                    Expression {
                                        span: Span(
                                            Id {
                                                value: 37,
                                            },
                                        ),
                                        data: Number(
                                            OrderedFloat(
                                                2.0,
                                            ),
                                        ),
                                    },
                                ),
                            },
                        ),
                    },
                ],
            },
            [],
        )"#]];
    expected.assert_eq(&actual);
}

#[test]
fn parse_error() {
    let source_text: &str = "print 1 + + 2";
    //                       0123456789^ <-- this is the position 10, where the error is reported
    let actual = parse_string(source_text);
    let expected = expect_test::expect![[r#"
        (
            Program {
                [salsa id]: 0,
                statements: [],
            },
            [
                Diagnostic {
                    start: 10,
                    end: 11,
                    message: "unexpected character",
                },
            ],
        )"#]];
    expected.assert_eq(&actual);
}

#[test]
fn parse_precedence() {
    // this parses as `(1 + (2 * 3)) + 4`
    let source_text: &str = "print 1 + 2 * 3 + 4;";
    let actual = parse_string(source_text);
    let expected = expect_test::expect![[r#"
        (
            Program {
                [salsa id]: 0,
                statements: [
                    Statement {
                        span: Span(
                            Id {
                                value: 11,
                            },
                        ),
                        data: Print(
                            Expression {
                                span: Span(
                                    Id {
                                        value: 10,
                                    },
                                ),
                                data: Op(
                                    Expression {
                                        span: Span(
                                            Id {
                                                value: 7,
                                            },
                                        ),
                                        data: Op(
                                            Expression {
                                                span: Span(
                                                    Id {
                                                        value: 1,
                                                    },
                                                ),
                                                data: Number(
                                                    OrderedFloat(
                                                        1.0,
                                                    ),
                                                ),
                                            },
                                            Add,
                                            Expression {
                                                span: Span(
                                                    Id {
                                                        value: 6,
                                                    },
                                                ),
                                                data: Op(
                                                    Expression {
                                                        span: Span(
                                                            Id {
                                                                value: 3,
                                                            },
                                                        ),
                                                        data: Number(
                                                            OrderedFloat(
                                                                2.0,
                                                            ),
                                                        ),
                                                    },
                                                    Multiply,
                                                    Expression {
                                                        span: Span(
                                                            Id {
                                                                value: 5,
                                                            },
                                                        ),
                                                        data: Number(
                                                            OrderedFloat(
                                                                3.0,
                                                            ),
                                                        ),
                                                    },
                                                ),
                                            },
                                        ),
                                    },
                                    Add,
                                    Expression {
                                        span: Span(
                                            Id {
                                                value: 9,
                                            },
                                        ),
                                        data: Number(
                                            OrderedFloat(
                                                4.0,
                                            ),
                                        ),
                                    },
                                ),
                            },
                        ),
                    },
                ],
            },
            [],
        )"#]];
    expected.assert_eq(&actual);
}
