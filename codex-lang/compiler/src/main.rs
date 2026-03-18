use std::collections::HashSet;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq)]
enum TypeName {
    I64,
    Void,
}

#[derive(Clone, Debug)]
struct Parameter {
    name: String,
    ty: TypeName,
}

#[derive(Clone, Debug)]
struct Function {
    name: String,
    params: Vec<Parameter>,
    return_type: TypeName,
    body: Vec<Statement>,
}

#[derive(Clone, Debug)]
enum Statement {
    Let { name: String, expr: Expr },
    Assign { name: String, expr: Expr },
    Emit(Expr),
    Return(Option<Expr>),
    If {
        condition: Expr,
        then_body: Vec<Statement>,
        else_body: Vec<Statement>,
    },
    While { condition: Expr, body: Vec<Statement> },
    Expr(Expr),
}

#[derive(Clone, Debug)]
enum Expr {
    Int(i64),
    Name(String),
    Unary { op: UnaryOp, expr: Box<Expr> },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Call { name: String, args: Vec<Expr> },
}

#[derive(Clone, Debug)]
enum UnaryOp {
    Neg,
}

#[derive(Clone, Debug)]
enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Clone, Debug)]
struct Line {
    number: usize,
    indent: usize,
    text: String,
}

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Int(i64),
    Name(String),
    LParen,
    RParen,
    Comma,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    EqEq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    End,
}

struct ExprParser {
    tokens: Vec<Token>,
    index: usize,
}

fn main() {
    if let Err(message) = run() {
        eprintln!("noema: {message}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        return Err(format!(
            "usage: {} <input.noe> <output-binary>",
            args.first().map(String::as_str).unwrap_or("noema")
        ));
    }

    let input_path = PathBuf::from(&args[1]);
    let output_path = PathBuf::from(&args[2]);
    let source = fs::read_to_string(&input_path)
        .map_err(|err| format!("failed to read {}: {err}", input_path.display()))?;

    let program = parse_program(&source)?;
    let c_source = lower_to_c(&program)?;
    let generated_dir = output_path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let c_output = if output_path.extension().and_then(|ext| ext.to_str()) == Some("c") {
        output_path.clone()
    } else {
        let generated_name = output_path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| format!("{name}.generated.c"))
            .unwrap_or_else(|| "generated.noema.c".to_string());
        generated_dir.join(generated_name)
    };

    fs::create_dir_all(&generated_dir)
        .map_err(|err| format!("failed to create {}: {err}", generated_dir.display()))?;
    fs::write(&c_output, c_source)
        .map_err(|err| format!("failed to write {}: {err}", c_output.display()))?;

    Ok(())
}

fn parse_program(source: &str) -> Result<Vec<Function>, String> {
    let lines = preprocess_lines(source);
    let mut index = 0;
    let mut functions = Vec::new();

    while index < lines.len() {
        let line = &lines[index];
        if line.indent != 0 {
            return Err(format!(
                "line {}: top-level items must not be indented",
                line.number
            ));
        }
        functions.push(parse_function(&lines, &mut index)?);
    }

    if functions.is_empty() {
        return Err("program defines no functions".to_string());
    }

    Ok(functions)
}

fn preprocess_lines(source: &str) -> Vec<Line> {
    let mut result = Vec::new();

    for (index, raw) in source.lines().enumerate() {
        let line_no = index + 1;
        let no_comment = raw.split('#').next().unwrap_or("");
        if no_comment.trim().is_empty() {
            continue;
        }

        let indent = no_comment.chars().take_while(|ch| *ch == ' ').count();
        let text = no_comment.trim().to_string();
        result.push(Line {
            number: line_no,
            indent,
            text,
        });
    }

    result
}

fn parse_function(lines: &[Line], index: &mut usize) -> Result<Function, String> {
    let line = &lines[*index];
    let header = line
        .text
        .strip_prefix("loom ")
        .ok_or_else(|| format!("line {}: expected function starting with 'loom'", line.number))?;

    if !header.ends_with(':') {
        return Err(format!("line {}: function header must end with ':'", line.number));
    }

    let header = &header[..header.len() - 1];
    let (signature, return_part) = header
        .split_once("->")
        .ok_or_else(|| format!("line {}: function header requires '->'", line.number))?;
    let return_type = parse_type(return_part.trim(), line.number)?;

    let open_paren = signature
        .find('(')
        .ok_or_else(|| format!("line {}: invalid function signature", line.number))?;
    let close_paren = signature
        .rfind(')')
        .ok_or_else(|| format!("line {}: invalid function signature", line.number))?;
    if close_paren < open_paren {
        return Err(format!("line {}: invalid function signature", line.number));
    }

    let name = signature[..open_paren].trim().to_string();
    if name.is_empty() {
        return Err(format!("line {}: function name cannot be empty", line.number));
    }
    let params_text = &signature[open_paren + 1..close_paren];
    let params = parse_parameters(params_text, line.number)?;

    *index += 1;
    let body = parse_block(lines, index, line.indent + 4)?;
    if body.is_empty() {
        return Err(format!("line {}: function body cannot be empty", line.number));
    }

    Ok(Function {
        name,
        params,
        return_type,
        body,
    })
}

fn parse_parameters(input: &str, line_no: usize) -> Result<Vec<Parameter>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let mut params = Vec::new();
    for part in trimmed.split(',') {
        let item = part.trim();
        let (name, ty) = item
            .split_once(':')
            .ok_or_else(|| format!("line {}: invalid parameter '{}'", line_no, item))?;
        params.push(Parameter {
            name: name.trim().to_string(),
            ty: parse_type(ty.trim(), line_no)?,
        });
    }
    Ok(params)
}

fn parse_type(input: &str, line_no: usize) -> Result<TypeName, String> {
    match input {
        "i64" => Ok(TypeName::I64),
        "void" => Ok(TypeName::Void),
        _ => Err(format!("line {}: unsupported type '{}'", line_no, input)),
    }
}

fn parse_block(lines: &[Line], index: &mut usize, indent: usize) -> Result<Vec<Statement>, String> {
    let mut statements = Vec::new();

    while *index < lines.len() {
        let line = &lines[*index];
        if line.indent < indent {
            break;
        }
        if line.indent > indent {
            return Err(format!(
                "line {}: unexpected indentation level {}",
                line.number, line.indent
            ));
        }
        statements.push(parse_statement(lines, index, indent)?);
    }

    Ok(statements)
}

fn parse_statement(lines: &[Line], index: &mut usize, indent: usize) -> Result<Statement, String> {
    let line = &lines[*index];
    let text = line.text.as_str();

    if let Some(rest) = text.strip_prefix("let ") {
        ensure_semicolon(rest, line.number)?;
        let rest = &rest[..rest.len() - 1];
        let (name, expr) = rest
            .split_once('=')
            .ok_or_else(|| format!("line {}: invalid let statement", line.number))?;
        *index += 1;
        return Ok(Statement::Let {
            name: name.trim().to_string(),
            expr: parse_expression(expr.trim(), line.number)?,
        });
    }

    if let Some(rest) = text.strip_prefix("emit ") {
        ensure_semicolon(rest, line.number)?;
        *index += 1;
        return Ok(Statement::Emit(parse_expression(
            rest[..rest.len() - 1].trim(),
            line.number,
        )?));
    }

    if let Some(rest) = text.strip_prefix("return") {
        if rest.trim().is_empty() {
            return Err(format!("line {}: return statements must end with ';'", line.number));
        }
        ensure_semicolon(rest.trim_start(), line.number)?;
        let payload = rest.trim_start();
        let payload = &payload[..payload.len() - 1];
        *index += 1;
        if payload.trim().is_empty() {
            return Ok(Statement::Return(None));
        }
        return Ok(Statement::Return(Some(parse_expression(
            payload.trim(),
            line.number,
        )?)));
    }

    if let Some(condition) = text.strip_prefix("if ") {
        if !condition.ends_with(':') {
            return Err(format!("line {}: if statement must end with ':'", line.number));
        }
        let condition = parse_expression(condition[..condition.len() - 1].trim(), line.number)?;
        *index += 1;
        let then_body = parse_block(lines, index, indent + 4)?;
        let mut else_body = Vec::new();
        if *index < lines.len()
            && lines[*index].indent == indent
            && lines[*index].text == "else:"
        {
            *index += 1;
            else_body = parse_block(lines, index, indent + 4)?;
        }
        return Ok(Statement::If {
            condition,
            then_body,
            else_body,
        });
    }

    if let Some(condition) = text.strip_prefix("while ") {
        if !condition.ends_with(':') {
            return Err(format!("line {}: while statement must end with ':'", line.number));
        }
        let condition = parse_expression(condition[..condition.len() - 1].trim(), line.number)?;
        *index += 1;
        let body = parse_block(lines, index, indent + 4)?;
        return Ok(Statement::While { condition, body });
    }

    ensure_semicolon(text, line.number)?;
    let content = &text[..text.len() - 1];
    if let Some((name, expr)) = split_assignment(content) {
        *index += 1;
        return Ok(Statement::Assign {
            name: name.trim().to_string(),
            expr: parse_expression(expr.trim(), line.number)?,
        });
    }

    *index += 1;
    Ok(Statement::Expr(parse_expression(content.trim(), line.number)?))
}

fn ensure_semicolon(input: &str, line_no: usize) -> Result<(), String> {
    if !input.ends_with(';') {
        return Err(format!("line {}: statement must end with ';'", line_no));
    }
    Ok(())
}

fn split_assignment(input: &str) -> Option<(&str, &str)> {
    let bytes = input.as_bytes();
    let mut depth = 0usize;
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] as char {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            '=' if depth == 0 => {
                let prev = if index > 0 { bytes[index - 1] as char } else { '\0' };
                let next = if index + 1 < bytes.len() {
                    bytes[index + 1] as char
                } else {
                    '\0'
                };
                if prev != '=' && prev != '<' && prev != '>' && prev != '!' && next != '=' {
                    return Some((&input[..index], &input[index + 1..]));
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn parse_expression(input: &str, line_no: usize) -> Result<Expr, String> {
    let tokens = tokenize(input, line_no)?;
    let mut parser = ExprParser { tokens, index: 0 };
    let expr = parser.parse_bp(0)?;
    if parser.current() != &Token::End {
        return Err(format!("line {}: unexpected trailing tokens in expression", line_no));
    }
    Ok(expr)
}

fn tokenize(input: &str, line_no: usize) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0usize;
    let mut tokens = Vec::new();

    while index < chars.len() {
        let ch = chars[index];
        if ch.is_whitespace() {
            index += 1;
            continue;
        }

        if ch.is_ascii_digit() {
            let start = index;
            index += 1;
            while index < chars.len() && chars[index].is_ascii_digit() {
                index += 1;
            }
            let value: String = chars[start..index].iter().collect();
            let parsed = value
                .parse::<i64>()
                .map_err(|_| format!("line {}: integer literal is too large", line_no))?;
            tokens.push(Token::Int(parsed));
            continue;
        }

        if ch.is_ascii_alphabetic() || ch == '_' {
            let start = index;
            index += 1;
            while index < chars.len()
                && (chars[index].is_ascii_alphanumeric() || chars[index] == '_')
            {
                index += 1;
            }
            let value: String = chars[start..index].iter().collect();
            tokens.push(Token::Name(value));
            continue;
        }

        match ch {
            '(' => {
                tokens.push(Token::LParen);
                index += 1;
            }
            ')' => {
                tokens.push(Token::RParen);
                index += 1;
            }
            ',' => {
                tokens.push(Token::Comma);
                index += 1;
            }
            '+' => {
                tokens.push(Token::Plus);
                index += 1;
            }
            '-' => {
                tokens.push(Token::Minus);
                index += 1;
            }
            '*' => {
                tokens.push(Token::Star);
                index += 1;
            }
            '/' => {
                tokens.push(Token::Slash);
                index += 1;
            }
            '%' => {
                tokens.push(Token::Percent);
                index += 1;
            }
            '=' => {
                if index + 1 < chars.len() && chars[index + 1] == '=' {
                    tokens.push(Token::EqEq);
                    index += 2;
                } else {
                    return Err(format!("line {}: unexpected '=' in expression", line_no));
                }
            }
            '!' => {
                if index + 1 < chars.len() && chars[index + 1] == '=' {
                    tokens.push(Token::NotEq);
                    index += 2;
                } else {
                    return Err(format!("line {}: unexpected '!'", line_no));
                }
            }
            '<' => {
                if index + 1 < chars.len() && chars[index + 1] == '=' {
                    tokens.push(Token::Le);
                    index += 2;
                } else {
                    tokens.push(Token::Lt);
                    index += 1;
                }
            }
            '>' => {
                if index + 1 < chars.len() && chars[index + 1] == '=' {
                    tokens.push(Token::Ge);
                    index += 2;
                } else {
                    tokens.push(Token::Gt);
                    index += 1;
                }
            }
            _ => {
                return Err(format!("line {}: unexpected character '{}'", line_no, ch));
            }
        }
    }

    tokens.push(Token::End);
    Ok(tokens)
}

impl ExprParser {
    fn current(&self) -> &Token {
        &self.tokens[self.index]
    }

    fn bump(&mut self) -> Token {
        let token = self.tokens[self.index].clone();
        self.index += 1;
        token
    }

    fn parse_bp(&mut self, min_bp: u8) -> Result<Expr, String> {
        let mut lhs = match self.bump() {
            Token::Int(value) => Expr::Int(value),
            Token::Name(name) => {
                if self.current() == &Token::LParen {
                    self.bump();
                    let mut args = Vec::new();
                    if self.current() != &Token::RParen {
                        loop {
                            args.push(self.parse_bp(0)?);
                            if self.current() == &Token::Comma {
                                self.bump();
                                continue;
                            }
                            break;
                        }
                    }
                    if self.current() != &Token::RParen {
                        return Err("expected ')' after function arguments".to_string());
                    }
                    self.bump();
                    Expr::Call { name, args }
                } else {
                    Expr::Name(name)
                }
            }
            Token::Minus => {
                let expr = self.parse_bp(100)?;
                Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                }
            }
            Token::LParen => {
                let expr = self.parse_bp(0)?;
                if self.current() != &Token::RParen {
                    return Err("expected ')'".to_string());
                }
                self.bump();
                expr
            }
            token => return Err(format!("unexpected token at start of expression: {:?}", token)),
        };

        loop {
            let (left_bp, right_bp, op) = match self.current() {
                Token::Plus => (10, 11, BinaryOp::Add),
                Token::Minus => (10, 11, BinaryOp::Sub),
                Token::Star => (20, 21, BinaryOp::Mul),
                Token::Slash => (20, 21, BinaryOp::Div),
                Token::Percent => (20, 21, BinaryOp::Mod),
                Token::EqEq => (5, 6, BinaryOp::Eq),
                Token::NotEq => (5, 6, BinaryOp::Ne),
                Token::Lt => (5, 6, BinaryOp::Lt),
                Token::Le => (5, 6, BinaryOp::Le),
                Token::Gt => (5, 6, BinaryOp::Gt),
                Token::Ge => (5, 6, BinaryOp::Ge),
                _ => break,
            };

            if left_bp < min_bp {
                break;
            }

            self.bump();
            let rhs = self.parse_bp(right_bp)?;
            lhs = Expr::Binary {
                left: Box::new(lhs),
                op,
                right: Box::new(rhs),
            };
        }

        Ok(lhs)
    }
}

fn lower_to_c(functions: &[Function]) -> Result<String, String> {
    let mut out = String::new();
    let function_names: HashSet<String> = functions.iter().map(|f| f.name.clone()).collect();

    if !function_names.contains("main") {
        return Err("program must define loom main()".to_string());
    }
    if functions
        .iter()
        .find(|function| function.name == "main")
        .map(|function| function.return_type.clone())
        != Some(TypeName::I64)
    {
        return Err("loom main() must return i64".to_string());
    }

    out.push_str("/* generated by noema-compiler */\n");
    out.push_str("#include <stdint.h>\n");
    out.push_str("#include <stdio.h>\n\n");

    for function in functions {
        let c_name = c_function_name(&function.name);
        write!(&mut out, "{} {}(", c_type_name(&function.return_type), c_name).unwrap();
        if function.params.is_empty() {
            out.push_str("void");
        } else {
            for (index, param) in function.params.iter().enumerate() {
                if index > 0 {
                    out.push_str(", ");
                }
                write!(
                    &mut out,
                    "{} {}",
                    c_type_name(&param.ty),
                    param.name
                )
                .unwrap();
            }
        }
        writeln!(&mut out, ") {{").unwrap();
        for statement in &function.body {
            lower_statement(statement, 1, &function_names, &mut out)?;
        }
        out.push_str("}\n\n");
    }

    out.push_str("int main(void) {\n");
    out.push_str("    return (int)codex_main();\n");
    out.push_str("}\n");

    Ok(out)
}

fn lower_statement(
    statement: &Statement,
    indent: usize,
    function_names: &HashSet<String>,
    out: &mut String,
) -> Result<(), String> {
    let prefix = "    ".repeat(indent);
    match statement {
        Statement::Let { name, expr } => {
            writeln!(
                out,
                "{}int64_t {} = {};",
                prefix,
                name,
                lower_expr(expr, function_names)?
            )
            .unwrap();
        }
        Statement::Assign { name, expr } => {
            writeln!(
                out,
                "{}{} = {};",
                prefix,
                name,
                lower_expr(expr, function_names)?
            )
            .unwrap();
        }
        Statement::Emit(expr) => {
            writeln!(
                out,
                "{}printf(\"%lld\\n\", (long long)({}));",
                prefix,
                lower_expr(expr, function_names)?
            )
            .unwrap();
        }
        Statement::Return(Some(expr)) => {
            writeln!(
                out,
                "{}return {};",
                prefix,
                lower_expr(expr, function_names)?
            )
            .unwrap();
        }
        Statement::Return(None) => {
            writeln!(out, "{}return;", prefix).unwrap();
        }
        Statement::Expr(expr) => {
            writeln!(out, "{}{};", prefix, lower_expr(expr, function_names)?).unwrap();
        }
        Statement::While { condition, body } => {
            writeln!(
                out,
                "{}while {} {{",
                prefix,
                lower_condition(condition, function_names)?
            )
            .unwrap();
            for child in body {
                lower_statement(child, indent + 1, function_names, out)?;
            }
            writeln!(out, "{}}}", prefix).unwrap();
        }
        Statement::If {
            condition,
            then_body,
            else_body,
        } => {
            writeln!(
                out,
                "{}if {} {{",
                prefix,
                lower_condition(condition, function_names)?
            )
            .unwrap();
            for child in then_body {
                lower_statement(child, indent + 1, function_names, out)?;
            }
            if else_body.is_empty() {
                writeln!(out, "{}}}", prefix).unwrap();
            } else {
                writeln!(out, "{}}} else {{", prefix).unwrap();
                for child in else_body {
                    lower_statement(child, indent + 1, function_names, out)?;
                }
                writeln!(out, "{}}}", prefix).unwrap();
            }
        }
    }
    Ok(())
}

fn lower_expr(expr: &Expr, function_names: &HashSet<String>) -> Result<String, String> {
    Ok(match expr {
        Expr::Int(value) => value.to_string(),
        Expr::Name(name) => name.clone(),
        Expr::Unary { op, expr } => match op {
            UnaryOp::Neg => format!("(-{})", lower_expr(expr, function_names)?),
        },
        Expr::Binary { left, op, right } => {
            let left = lower_expr(left, function_names)?;
            let right = lower_expr(right, function_names)?;
            let op_text = match op {
                BinaryOp::Add => "+",
                BinaryOp::Sub => "-",
                BinaryOp::Mul => "*",
                BinaryOp::Div => "/",
                BinaryOp::Mod => "%",
                BinaryOp::Eq => "==",
                BinaryOp::Ne => "!=",
                BinaryOp::Lt => "<",
                BinaryOp::Le => "<=",
                BinaryOp::Gt => ">",
                BinaryOp::Ge => ">=",
            };
            format!("({left} {op_text} {right})")
        }
        Expr::Call { name, args } => {
            let c_name = if function_names.contains(name) {
                c_function_name(name)
            } else {
                name.clone()
            };
            let lowered_args = args
                .iter()
                .map(|arg| lower_expr(arg, function_names))
                .collect::<Result<Vec<_>, _>>()?;
            format!("{c_name}({})", lowered_args.join(", "))
        }
    })
}

fn lower_condition(expr: &Expr, function_names: &HashSet<String>) -> Result<String, String> {
    Ok(format!("({})", lower_expr(expr, function_names)?))
}

fn c_function_name(name: &str) -> String {
    if name == "main" {
        "codex_main".to_string()
    } else {
        name.to_string()
    }
}

fn c_type_name(ty: &TypeName) -> &'static str {
    match ty {
        TypeName::I64 => "int64_t",
        TypeName::Void => "void",
    }
}
