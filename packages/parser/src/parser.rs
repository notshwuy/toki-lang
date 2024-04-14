use chumsky::prelude::*;

use chumsky::text::newline;
use chumsky::Parser;

#[derive(Debug)]
pub enum Expression {
    Import(String),
    Ident(String),
    String(String),
    Function(String, Box<Expression>),
    FunctionCall(Box<Expression>, Vec<Expression>),
    Scope(Vec<Expression>),
    Chain(Vec<Expression>),
    Number(i32),
    Empty,
    Dot,
}

pub fn parser() -> impl Parser<char, Vec<Expression>, Error = Simple<char>> {
    let ident = text::ident().padded();
    let dot = just(".").map(|_| Expression::Dot);

    let string = just('"')
        .ignore_then(none_of("\"").repeated())
        .then_ignore(just('"'))
        .collect::<String>()
        .map(Expression::String);

    let attribute_access = ident
        .map(Expression::Ident)
        .separated_by(dot)
        .collect::<Vec<_>>();

    let apply_arguments = string
        .clone()
        .padded()
        .separated_by(just(","))
        .delimited_by(just("("), just(")"));

    let apply = attribute_access
        .then(apply_arguments)
        .map(|(attribute_access, args)| {
            Expression::FunctionCall(Box::new(Expression::Chain(attribute_access)), args)
        });

    let expression = apply;

    let block = expression
        .separated_by(newline())
        .padded()
        .delimited_by(just("{"), just("}"))
        .padded()
        .map(|expressions| expressions);

    let import = recursive(|_| {
        let import = text::keyword("import")
            .then(ident)
            .then(just("/"))
            .then(ident)
            .map(|(((_, t), _), e)| Expression::Import(format!("{}/{}", t, e)));

        import
    });

    let function = recursive(|_| {
        let function = text::keyword("fun")
            .then(ident)
            // todo take args
            .then_ignore(just("("))
            .then_ignore(just(")"))
            .then(block)
            .map(|((_, name), body)| Expression::Function(name, Expression::Scope(body).into()));

        function.or(import.clone())
    });

    let program = recursive(|_| import.clone().or(function).repeated().collect().padded());

    program.then_ignore(end())
}
