use std::collections::{HashMap, HashSet};
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
struct Program {
    shapes: Vec<ShapeDecl>,
    functions: Vec<Function>,
}

#[derive(Clone, Debug)]
struct ShapeDecl {
    name: String,
    fields: Vec<FieldDecl>,
}

#[derive(Clone, Debug)]
struct FieldDecl {
    name: String,
    ty: TypeName,
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
    Let {
        name: String,
        annotation: Option<TypeName>,
        expr: Expr,
    },
    Assign {
        target: AssignTarget,
        expr: Expr,
    },
    Emit(Expr),
    Return(Option<Expr>),
    If {
        condition: Expr,
        then_body: Vec<Statement>,
        else_body: Vec<Statement>,
    },
    While {
        condition: Expr,
        body: Vec<Statement>,
    },
    Expr(Expr),
}

#[derive(Clone, Debug)]
enum AssignTarget {
    Name(String),
    Field {
        base: Box<AssignTarget>,
        field: String,
    },
}

#[derive(Clone, Debug)]
enum Expr {
    Int(i64),
    Bool(bool),
    Text(String),
    Name(String),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    Field {
        base: Box<Expr>,
        field: String,
    },
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
    },
    ListLiteral(Vec<Expr>),
    StructLiteral {
        name: String,
        fields: Vec<(String, Expr)>,
    },
}

#[derive(Clone, Debug)]
enum UnaryOp {
    Neg,
    Not,
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
    And,
    Or,
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
    Bool(bool),
    String(String),
    Name(String),
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Comma,
    Colon,
    Dot,
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
    And,
    Or,
    Not,
    End,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum TypeName {
    I64,
    Bool,
    Text,
    Socket,
    Void,
    Named(String),
    List(Box<TypeName>),
}

#[derive(Clone, Debug)]
struct FunctionSignature {
    params: Vec<TypeName>,
    return_type: TypeName,
}

#[derive(Clone, Debug)]
struct SemanticInfo {
    shapes: HashMap<String, ShapeDecl>,
    functions: HashMap<String, FunctionSignature>,
    list_types: HashSet<TypeName>,
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
    let semantic = analyze_program(&program)?;
    let c_source = lower_to_c(&program, &semantic)?;
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

fn parse_program(source: &str) -> Result<Program, String> {
    let lines = preprocess_lines(source);
    let mut index = 0;
    let mut shapes = Vec::new();
    let mut functions = Vec::new();

    while index < lines.len() {
        let line = &lines[index];
        if line.indent != 0 {
            return Err(format!(
                "line {}: top-level items must not be indented",
                line.number
            ));
        }

        if line.text.starts_with("shape ") {
            shapes.push(parse_shape(&lines, &mut index)?);
        } else if line.text.starts_with("loom ") {
            functions.push(parse_function(&lines, &mut index)?);
        } else {
            return Err(format!(
                "line {}: expected top-level 'shape' or 'loom'",
                line.number
            ));
        }
    }

    if functions.is_empty() {
        return Err("program defines no functions".to_string());
    }

    Ok(Program { shapes, functions })
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

fn parse_shape(lines: &[Line], index: &mut usize) -> Result<ShapeDecl, String> {
    let line = &lines[*index];
    let header = line
        .text
        .strip_prefix("shape ")
        .ok_or_else(|| format!("line {}: expected shape declaration", line.number))?;

    if !header.ends_with(':') {
        return Err(format!("line {}: shape header must end with ':'", line.number));
    }

    let name = header[..header.len() - 1].trim().to_string();
    if name.is_empty() {
        return Err(format!("line {}: shape name cannot be empty", line.number));
    }

    *index += 1;
    let mut fields = Vec::new();
    while *index < lines.len() {
        let field_line = &lines[*index];
        if field_line.indent < line.indent + 4 {
            break;
        }
        if field_line.indent > line.indent + 4 {
            return Err(format!(
                "line {}: unexpected indentation level {}",
                field_line.number, field_line.indent
            ));
        }

        let (field_name, field_type) = field_line
            .text
            .split_once(':')
            .ok_or_else(|| format!("line {}: invalid shape field", field_line.number))?;
        let field_name = field_name.trim().to_string();
        if field_name.is_empty() {
            return Err(format!("line {}: field name cannot be empty", field_line.number));
        }
        fields.push(FieldDecl {
            name: field_name,
            ty: parse_type_text(field_type.trim(), field_line.number)?,
        });
        *index += 1;
    }

    if fields.is_empty() {
        return Err(format!("line {}: shape body cannot be empty", line.number));
    }

    Ok(ShapeDecl { name, fields })
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
    let return_type = parse_type_text(return_part.trim(), line.number)?;

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
    for part in split_top_level(trimmed, ',') {
        let item = part.trim();
        let (name, ty) = item
            .split_once(':')
            .ok_or_else(|| format!("line {}: invalid parameter '{}'", line_no, item))?;
        params.push(Parameter {
            name: name.trim().to_string(),
            ty: parse_type_text(ty.trim(), line_no)?,
        });
    }

    Ok(params)
}

fn parse_type_text(input: &str, line_no: usize) -> Result<TypeName, String> {
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;
    let ty = parse_type_inner(&chars, &mut index)
        .ok_or_else(|| format!("line {}: invalid type '{}'", line_no, input))?;
    skip_spaces(&chars, &mut index);
    if index != chars.len() {
        return Err(format!("line {}: invalid type '{}'", line_no, input));
    }
    Ok(ty)
}

fn parse_type_inner(chars: &[char], index: &mut usize) -> Option<TypeName> {
    skip_spaces(chars, index);
    let ident = parse_identifier(chars, index)?;
    if ident == "list" {
        skip_spaces(chars, index);
        if *index >= chars.len() || chars[*index] != '<' {
            return None;
        }
        *index += 1;
        let inner = parse_type_inner(chars, index)?;
        skip_spaces(chars, index);
        if *index >= chars.len() || chars[*index] != '>' {
            return None;
        }
        *index += 1;
        return Some(TypeName::List(Box::new(inner)));
    }

    Some(match ident.as_str() {
        "i64" => TypeName::I64,
        "bool" => TypeName::Bool,
        "text" => TypeName::Text,
        "socket" => TypeName::Socket,
        "void" => TypeName::Void,
        _ => TypeName::Named(ident),
    })
}

fn skip_spaces(chars: &[char], index: &mut usize) {
    while *index < chars.len() && chars[*index].is_whitespace() {
        *index += 1;
    }
}

fn parse_identifier(chars: &[char], index: &mut usize) -> Option<String> {
    skip_spaces(chars, index);
    if *index >= chars.len() {
        return None;
    }
    let start = *index;
    let first = chars[*index];
    if !(first.is_ascii_alphabetic() || first == '_') {
        return None;
    }
    *index += 1;
    while *index < chars.len() && (chars[*index].is_ascii_alphanumeric() || chars[*index] == '_')
    {
        *index += 1;
    }
    Some(chars[start..*index].iter().collect())
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
        let (lhs, rhs) = split_assignment_expr(rest)
            .ok_or_else(|| format!("line {}: invalid let statement", line.number))?;
        let lhs = lhs.trim();
        let (name, annotation) = if let Some(colon_index) = find_top_level_char(lhs, ':') {
            let name = lhs[..colon_index].trim().to_string();
            let ty = parse_type_text(lhs[colon_index + 1..].trim(), line.number)?;
            (name, Some(ty))
        } else {
            (lhs.to_string(), None)
        };
        if name.is_empty() {
            return Err(format!("line {}: let binding name cannot be empty", line.number));
        }
        *index += 1;
        return Ok(Statement::Let {
            name,
            annotation,
            expr: parse_expression(rhs.trim(), line.number)?,
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
    if let Some((lhs, rhs)) = split_assignment_expr(content) {
        let target = parse_assign_target(lhs.trim(), line.number)?;
        *index += 1;
        return Ok(Statement::Assign {
            target,
            expr: parse_expression(rhs.trim(), line.number)?,
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

fn split_assignment_expr(input: &str) -> Option<(&str, &str)> {
    let chars: Vec<char> = input.chars().collect();
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut bracket_depth = 0usize;

    let mut index = 0usize;
    while index < chars.len() {
        match chars[index] {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '=' if paren_depth == 0 && brace_depth == 0 && bracket_depth == 0 => {
                let prev = if index > 0 { chars[index - 1] } else { '\0' };
                let next = if index + 1 < chars.len() {
                    chars[index + 1]
                } else {
                    '\0'
                };
                if prev != '=' && prev != '<' && prev != '>' && prev != '!' && next != '=' {
                    let lhs = chars[..index].iter().collect::<String>();
                    let rhs = chars[index + 1..].iter().collect::<String>();
                    return Some((Box::leak(lhs.into_boxed_str()), Box::leak(rhs.into_boxed_str())));
                }
            }
            _ => {}
        }
        index += 1;
    }

    None
}

fn split_top_level(input: &str, delimiter: char) -> Vec<String> {
    let chars: Vec<char> = input.chars().collect();
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut bracket_depth = 0usize;

    for (index, ch) in chars.iter().enumerate() {
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }

        if *ch == delimiter && paren_depth == 0 && brace_depth == 0 && bracket_depth == 0 {
            parts.push(chars[start..index].iter().collect::<String>());
            start = index + 1;
        }
    }

    parts.push(chars[start..].iter().collect::<String>());
    parts
}

fn find_top_level_char(input: &str, target: char) -> Option<usize> {
    let chars: Vec<char> = input.chars().collect();
    let mut paren_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut bracket_depth = 0usize;

    for (index, ch) in chars.iter().enumerate() {
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        if *ch == target && paren_depth == 0 && brace_depth == 0 && bracket_depth == 0 {
            return Some(index);
        }
    }

    None
}

fn parse_assign_target(input: &str, line_no: usize) -> Result<AssignTarget, String> {
    let expr = parse_expression(input, line_no)?;
    expr_to_assign_target(expr)
        .map_err(|_| format!("line {}: invalid assignment target '{}'", line_no, input))
}

fn expr_to_assign_target(expr: Expr) -> Result<AssignTarget, ()> {
    match expr {
        Expr::Name(name) => Ok(AssignTarget::Name(name)),
        Expr::Field { base, field } => Ok(AssignTarget::Field {
            base: Box::new(expr_to_assign_target(*base)?),
            field,
        }),
        _ => Err(()),
    }
}

fn parse_expression(input: &str, line_no: usize) -> Result<Expr, String> {
    let tokens = tokenize(input, line_no)?;
    let mut parser = ExprParser { tokens, index: 0 };
    let expr = parser.parse_bp(0)?;
    if parser.current() != &Token::End {
        return Err(format!(
            "line {}: unexpected trailing tokens in expression",
            line_no
        ));
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

        if ch == '"' {
            index += 1;
            let mut value = String::new();
            while index < chars.len() {
                let current = chars[index];
                if current == '"' {
                    index += 1;
                    break;
                }
                if current == '\\' {
                    index += 1;
                    if index >= chars.len() {
                        return Err(format!("line {}: unterminated string literal", line_no));
                    }
                    let escaped = match chars[index] {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        '"' => '"',
                        '\\' => '\\',
                        other => {
                            return Err(format!(
                                "line {}: unsupported string escape '\\{}'",
                                line_no, other
                            ))
                        }
                    };
                    value.push(escaped);
                    index += 1;
                    continue;
                }
                value.push(current);
                index += 1;
            }
            if index > chars.len() {
                return Err(format!("line {}: unterminated string literal", line_no));
            }
            tokens.push(Token::String(value));
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
            match value.as_str() {
                "true" => tokens.push(Token::Bool(true)),
                "false" => tokens.push(Token::Bool(false)),
                "and" => tokens.push(Token::And),
                "or" => tokens.push(Token::Or),
                "not" => tokens.push(Token::Not),
                _ => tokens.push(Token::Name(value)),
            }
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
            '{' => {
                tokens.push(Token::LBrace);
                index += 1;
            }
            '}' => {
                tokens.push(Token::RBrace);
                index += 1;
            }
            '[' => {
                tokens.push(Token::LBracket);
                index += 1;
            }
            ']' => {
                tokens.push(Token::RBracket);
                index += 1;
            }
            ',' => {
                tokens.push(Token::Comma);
                index += 1;
            }
            ':' => {
                tokens.push(Token::Colon);
                index += 1;
            }
            '.' => {
                tokens.push(Token::Dot);
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
            Token::Bool(value) => Expr::Bool(value),
            Token::String(value) => Expr::Text(value),
            Token::Name(name) => Expr::Name(name),
            Token::Minus => {
                let expr = self.parse_bp(100)?;
                Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(expr),
                }
            }
            Token::Not => {
                let expr = self.parse_bp(100)?;
                Expr::Unary {
                    op: UnaryOp::Not,
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
            Token::LBracket => {
                let mut items = Vec::new();
                if self.current() != &Token::RBracket {
                    loop {
                        items.push(self.parse_bp(0)?);
                        if self.current() == &Token::Comma {
                            self.bump();
                            continue;
                        }
                        break;
                    }
                }
                if self.current() != &Token::RBracket {
                    return Err("expected ']' after list literal".to_string());
                }
                self.bump();
                Expr::ListLiteral(items)
            }
            token => return Err(format!("unexpected token at start of expression: {:?}", token)),
        };

        loop {
            lhs = match self.current() {
                Token::LParen => self.parse_call(lhs)?,
                Token::LBrace => self.parse_struct_literal(lhs)?,
                Token::Dot => self.parse_field(lhs)?,
                Token::LBracket => self.parse_index(lhs)?,
                _ => lhs,
            };

            let (left_bp, right_bp, op) = match self.current() {
                Token::Or => (1, 2, BinaryOp::Or),
                Token::And => (3, 4, BinaryOp::And),
                Token::EqEq => (5, 6, BinaryOp::Eq),
                Token::NotEq => (5, 6, BinaryOp::Ne),
                Token::Lt => (5, 6, BinaryOp::Lt),
                Token::Le => (5, 6, BinaryOp::Le),
                Token::Gt => (5, 6, BinaryOp::Gt),
                Token::Ge => (5, 6, BinaryOp::Ge),
                Token::Plus => (10, 11, BinaryOp::Add),
                Token::Minus => (10, 11, BinaryOp::Sub),
                Token::Star => (20, 21, BinaryOp::Mul),
                Token::Slash => (20, 21, BinaryOp::Div),
                Token::Percent => (20, 21, BinaryOp::Mod),
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

    fn parse_call(&mut self, lhs: Expr) -> Result<Expr, String> {
        let name = match lhs {
            Expr::Name(name) => name,
            _ => return Err("only named functions can be called".to_string()),
        };
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
        Ok(Expr::Call { name, args })
    }

    fn parse_struct_literal(&mut self, lhs: Expr) -> Result<Expr, String> {
        let name = match lhs {
            Expr::Name(name) => name,
            _ => return Err("struct literals must start with a shape name".to_string()),
        };
        self.bump();
        let mut fields = Vec::new();
        if self.current() != &Token::RBrace {
            loop {
                let field_name = match self.bump() {
                    Token::Name(name) => name,
                    token => {
                        return Err(format!(
                            "expected field name in struct literal, found {:?}",
                            token
                        ))
                    }
                };
                if self.current() != &Token::Colon {
                    return Err("expected ':' in struct literal".to_string());
                }
                self.bump();
                let expr = self.parse_bp(0)?;
                fields.push((field_name, expr));
                if self.current() == &Token::Comma {
                    self.bump();
                    continue;
                }
                break;
            }
        }
        if self.current() != &Token::RBrace {
            return Err("expected '}' after struct literal".to_string());
        }
        self.bump();
        Ok(Expr::StructLiteral { name, fields })
    }

    fn parse_field(&mut self, lhs: Expr) -> Result<Expr, String> {
        self.bump();
        let field = match self.bump() {
            Token::Name(name) => name,
            token => return Err(format!("expected field name after '.', found {:?}", token)),
        };
        Ok(Expr::Field {
            base: Box::new(lhs),
            field,
        })
    }

    fn parse_index(&mut self, lhs: Expr) -> Result<Expr, String> {
        self.bump();
        let index = self.parse_bp(0)?;
        if self.current() != &Token::RBracket {
            return Err("expected ']' after index expression".to_string());
        }
        self.bump();
        Ok(Expr::Index {
            base: Box::new(lhs),
            index: Box::new(index),
        })
    }
}

fn analyze_program(program: &Program) -> Result<SemanticInfo, String> {
    let mut shapes = HashMap::new();
    let mut functions = HashMap::new();
    let mut list_types = HashSet::new();

    for shape in &program.shapes {
        if shapes.insert(shape.name.clone(), shape.clone()).is_some() {
            return Err(format!("duplicate shape '{}'", shape.name));
        }
        if shape.fields.is_empty() {
            return Err(format!("shape '{}' must declare at least one field", shape.name));
        }
        let mut field_names = HashSet::new();
        for field in &shape.fields {
            if !field_names.insert(field.name.clone()) {
                return Err(format!(
                    "shape '{}' has duplicate field '{}'",
                    shape.name, field.name
                ));
            }
        }
    }

    for shape in program.shapes.iter() {
        for field in &shape.fields {
            ensure_type_defined(&field.ty, &shapes)?;
            register_list_types(&field.ty, &mut list_types);
        }
    }

    for function in &program.functions {
        if functions
            .insert(
                function.name.clone(),
                FunctionSignature {
                    params: function.params.iter().map(|param| param.ty.clone()).collect(),
                    return_type: function.return_type.clone(),
                },
            )
            .is_some()
        {
            return Err(format!("duplicate function '{}'", function.name));
        }
    }

    let main_signature = functions
        .get("main")
        .ok_or_else(|| "program must define loom main()".to_string())?;
    if !main_signature.params.is_empty() {
        return Err("loom main() must not accept parameters".to_string());
    }
    if main_signature.return_type != TypeName::I64 {
        return Err("loom main() must return i64".to_string());
    }

    for function in &program.functions {
        let mut parameter_names = HashSet::new();
        for param in &function.params {
            if !parameter_names.insert(param.name.clone()) {
                return Err(format!(
                    "function '{}' has duplicate parameter '{}'",
                    function.name, param.name
                ));
            }
            ensure_type_defined(&param.ty, &shapes)?;
            register_list_types(&param.ty, &mut list_types);
        }
        ensure_type_defined(&function.return_type, &shapes)?;
        register_list_types(&function.return_type, &mut list_types);

        let mut env = HashMap::new();
        for param in &function.params {
            env.insert(param.name.clone(), param.ty.clone());
        }
        analyze_block(
            &function.body,
            &mut env,
            &shapes,
            &functions,
            &function.return_type,
            &mut list_types,
        )?;
    }

    Ok(SemanticInfo {
        shapes,
        functions,
        list_types,
    })
}

fn ensure_type_defined(
    ty: &TypeName,
    shapes: &HashMap<String, ShapeDecl>,
) -> Result<(), String> {
    match ty {
        TypeName::I64 | TypeName::Bool | TypeName::Text | TypeName::Socket | TypeName::Void => Ok(()),
        TypeName::Named(name) => {
            if shapes.contains_key(name) {
                Ok(())
            } else {
                Err(format!("unknown shape type '{}'", name))
            }
        }
        TypeName::List(inner) => {
            if **inner == TypeName::Void {
                return Err("list<void> is not supported".to_string());
            }
            ensure_type_defined(inner, shapes)
        }
    }
}

fn register_list_types(ty: &TypeName, list_types: &mut HashSet<TypeName>) {
    if let TypeName::List(inner) = ty {
        let list_ty = TypeName::List(inner.clone());
        list_types.insert(list_ty);
        register_list_types(inner, list_types);
    }
}

fn analyze_block(
    statements: &[Statement],
    env: &mut HashMap<String, TypeName>,
    shapes: &HashMap<String, ShapeDecl>,
    functions: &HashMap<String, FunctionSignature>,
    return_type: &TypeName,
    list_types: &mut HashSet<TypeName>,
) -> Result<(), String> {
    for statement in statements {
        analyze_statement(statement, env, shapes, functions, return_type, list_types)?;
    }
    Ok(())
}

fn analyze_statement(
    statement: &Statement,
    env: &mut HashMap<String, TypeName>,
    shapes: &HashMap<String, ShapeDecl>,
    functions: &HashMap<String, FunctionSignature>,
    return_type: &TypeName,
    list_types: &mut HashSet<TypeName>,
) -> Result<(), String> {
    match statement {
        Statement::Let {
            name,
            annotation,
            expr,
        } => {
            if env.contains_key(name) {
                return Err(format!("binding '{}' is already defined in this scope", name));
            }
            if let Some(ty) = annotation {
                ensure_type_defined(ty, shapes)?;
                register_list_types(ty, list_types);
            }
            let expr_type = infer_expr_type(expr, env, shapes, functions, annotation.as_ref(), list_types)?;
            if let Some(annotation) = annotation {
                if &expr_type != annotation {
                    return Err(format!(
                        "let binding '{}' expected type '{}', got '{}'",
                        name,
                        annotation.display(),
                        expr_type.display()
                    ));
                }
            }
            env.insert(name.clone(), expr_type);
        }
        Statement::Assign { target, expr } => {
            let target_type = infer_target_type(target, env, shapes)?;
            let expr_type = infer_expr_type(expr, env, shapes, functions, Some(&target_type), list_types)?;
            if expr_type != target_type {
                return Err(format!(
                    "assignment expected type '{}', got '{}'",
                    target_type.display(),
                    expr_type.display()
                ));
            }
        }
        Statement::Emit(expr) => {
            let ty = infer_expr_type(expr, env, shapes, functions, None, list_types)?;
            if ty != TypeName::I64 && ty != TypeName::Bool && ty != TypeName::Text {
                return Err(format!(
                    "emit only supports i64, bool, or text values, got '{}'",
                    ty.display()
                ));
            }
        }
        Statement::Return(None) => {
            if *return_type != TypeName::Void {
                return Err(format!(
                    "return; requires function return type void, got '{}'",
                    return_type.display()
                ));
            }
        }
        Statement::Return(Some(expr)) => {
            if *return_type == TypeName::Void {
                return Err("void functions cannot return a value".to_string());
            }
            let ty = infer_expr_type(expr, env, shapes, functions, Some(return_type), list_types)?;
            if &ty != return_type {
                return Err(format!(
                    "return expected type '{}', got '{}'",
                    return_type.display(),
                    ty.display()
                ));
            }
        }
        Statement::If {
            condition,
            then_body,
            else_body,
        } => {
            let ty = infer_expr_type(condition, env, shapes, functions, None, list_types)?;
            if ty != TypeName::Bool {
                return Err("if conditions must evaluate to bool".to_string());
            }
            let mut then_env = env.clone();
            analyze_block(
                then_body,
                &mut then_env,
                shapes,
                functions,
                return_type,
                list_types,
            )?;
            let mut else_env = env.clone();
            analyze_block(
                else_body,
                &mut else_env,
                shapes,
                functions,
                return_type,
                list_types,
            )?;
        }
        Statement::While { condition, body } => {
            let ty = infer_expr_type(condition, env, shapes, functions, None, list_types)?;
            if ty != TypeName::Bool {
                return Err("while conditions must evaluate to bool".to_string());
            }
            let mut loop_env = env.clone();
            analyze_block(
                body,
                &mut loop_env,
                shapes,
                functions,
                return_type,
                list_types,
            )?;
        }
        Statement::Expr(expr) => {
            infer_expr_type(expr, env, shapes, functions, None, list_types)?;
        }
    }

    Ok(())
}

fn infer_target_type(
    target: &AssignTarget,
    env: &HashMap<String, TypeName>,
    shapes: &HashMap<String, ShapeDecl>,
) -> Result<TypeName, String> {
    match target {
        AssignTarget::Name(name) => env
            .get(name)
            .cloned()
            .ok_or_else(|| format!("unknown variable '{}'", name)),
        AssignTarget::Field { base, field } => {
            let base_type = infer_target_type(base, env, shapes)?;
            match base_type {
                TypeName::Named(shape_name) => {
                    let shape = shapes
                        .get(&shape_name)
                        .ok_or_else(|| format!("unknown shape '{}'", shape_name))?;
                    let field_decl = shape
                        .fields
                        .iter()
                        .find(|item| item.name == *field)
                        .ok_or_else(|| {
                            format!("shape '{}' has no field '{}'", shape_name, field)
                        })?;
                    Ok(field_decl.ty.clone())
                }
                other => Err(format!(
                    "field assignment requires a shape value, got '{}'",
                    other.display()
                )),
            }
        }
    }
}

fn infer_expr_type(
    expr: &Expr,
    env: &HashMap<String, TypeName>,
    shapes: &HashMap<String, ShapeDecl>,
    functions: &HashMap<String, FunctionSignature>,
    expected: Option<&TypeName>,
    list_types: &mut HashSet<TypeName>,
) -> Result<TypeName, String> {
    let ty = match expr {
        Expr::Int(_) => TypeName::I64,
        Expr::Bool(_) => TypeName::Bool,
        Expr::Text(_) => TypeName::Text,
        Expr::Name(name) => env
            .get(name)
            .cloned()
            .ok_or_else(|| format!("unknown variable '{}'", name))?,
        Expr::Unary { op, expr } => {
            let inner = infer_expr_type(expr, env, shapes, functions, None, list_types)?;
            match op {
                UnaryOp::Neg => {
                    if inner != TypeName::I64 {
                        return Err(format!(
                            "unary '-' requires i64, got '{}'",
                            inner.display()
                        ));
                    }
                    TypeName::I64
                }
                UnaryOp::Not => {
                    if inner != TypeName::Bool {
                        return Err(format!(
                            "unary 'not' requires bool, got '{}'",
                            inner.display()
                        ));
                    }
                    TypeName::Bool
                }
            }
        }
        Expr::Binary { left, op, right } => {
            let left_type = infer_expr_type(left, env, shapes, functions, None, list_types)?;
            let right_type =
                infer_expr_type(right, env, shapes, functions, Some(&left_type), list_types)?;
            match op {
                BinaryOp::Add => {
                    if left_type == TypeName::I64 && right_type == TypeName::I64 {
                        TypeName::I64
                    } else if left_type == TypeName::Text && right_type == TypeName::Text {
                        TypeName::Text
                    } else {
                        return Err(format!(
                            "'+' requires matching i64 or text operands, got '{}' and '{}'",
                            left_type.display(),
                            right_type.display()
                        ));
                    }
                }
                BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                    if left_type != TypeName::I64 || right_type != TypeName::I64 {
                        return Err(format!(
                            "arithmetic requires i64 operands, got '{}' and '{}'",
                            left_type.display(),
                            right_type.display()
                        ));
                    }
                    TypeName::I64
                }
                BinaryOp::Eq | BinaryOp::Ne => {
                    if left_type != right_type {
                        return Err(format!(
                            "comparison requires matching operand types, got '{}' and '{}'",
                            left_type.display(),
                            right_type.display()
                        ));
                    }
                    if left_type != TypeName::I64
                        && left_type != TypeName::Bool
                        && left_type != TypeName::Text
                    {
                        return Err(format!(
                            "comparison is not supported for type '{}'",
                            left_type.display()
                        ));
                    }
                    TypeName::Bool
                }
                BinaryOp::Lt | BinaryOp::Le | BinaryOp::Gt | BinaryOp::Ge => {
                    if left_type != TypeName::I64 || right_type != TypeName::I64 {
                        return Err(format!(
                            "ordered comparison requires i64 operands, got '{}' and '{}'",
                            left_type.display(),
                            right_type.display()
                        ));
                    }
                    TypeName::Bool
                }
                BinaryOp::And | BinaryOp::Or => {
                    if left_type != TypeName::Bool || right_type != TypeName::Bool {
                        return Err(format!(
                            "logical operators require bool operands, got '{}' and '{}'",
                            left_type.display(),
                            right_type.display()
                        ));
                    }
                    TypeName::Bool
                }
            }
        }
        Expr::Call { name, args } => infer_call_type(
            name,
            args,
            env,
            shapes,
            functions,
            list_types,
        )?,
        Expr::Field { base, field } => {
            let base_type = infer_expr_type(base, env, shapes, functions, None, list_types)?;
            match base_type {
                TypeName::Named(shape_name) => {
                    let shape = shapes
                        .get(&shape_name)
                        .ok_or_else(|| format!("unknown shape '{}'", shape_name))?;
                    let field_decl = shape
                        .fields
                        .iter()
                        .find(|item| item.name == *field)
                        .ok_or_else(|| {
                            format!("shape '{}' has no field '{}'", shape_name, field)
                        })?;
                    field_decl.ty.clone()
                }
                other => {
                    return Err(format!(
                        "field access requires a shape value, got '{}'",
                        other.display()
                    ))
                }
            }
        }
        Expr::Index { base, index } => {
            let base_type = infer_expr_type(base, env, shapes, functions, None, list_types)?;
            let index_type = infer_expr_type(index, env, shapes, functions, None, list_types)?;
            if index_type != TypeName::I64 {
                return Err(format!(
                    "list indexing requires i64 indices, got '{}'",
                    index_type.display()
                ));
            }
            match base_type {
                TypeName::List(inner) => *inner,
                other => {
                    return Err(format!(
                        "indexing requires a list value, got '{}'",
                        other.display()
                    ))
                }
            }
        }
        Expr::ListLiteral(items) => {
            let hinted = expected.and_then(|ty| match ty {
                TypeName::List(inner) => Some(inner.as_ref()),
                _ => None,
            });
            let element_type = if items.is_empty() {
                hinted
                    .cloned()
                    .ok_or_else(|| "empty list literals require an expected list<T> type".to_string())?
            } else {
                let first = infer_expr_type(
                    &items[0],
                    env,
                    shapes,
                    functions,
                    hinted,
                    list_types,
                )?;
                for item in items.iter().skip(1) {
                    let item_type = infer_expr_type(
                        item,
                        env,
                        shapes,
                        functions,
                        Some(&first),
                        list_types,
                    )?;
                    if item_type != first {
                        return Err(format!(
                            "list literal items must share one type, got '{}' and '{}'",
                            first.display(),
                            item_type.display()
                        ));
                    }
                }
                first
            };
            let list_type = TypeName::List(Box::new(element_type));
            list_types.insert(list_type.clone());
            list_type
        }
        Expr::StructLiteral { name, fields } => {
            let shape = shapes
                .get(name)
                .ok_or_else(|| format!("unknown shape '{}'", name))?;
            if fields.len() != shape.fields.len() {
                return Err(format!(
                    "shape '{}' expects {} fields, got {}",
                    name,
                    shape.fields.len(),
                    fields.len()
                ));
            }
            let mut seen = HashSet::new();
            for (field_name, field_expr) in fields {
                if !seen.insert(field_name.clone()) {
                    return Err(format!(
                        "shape literal '{}' repeats field '{}'",
                        name, field_name
                    ));
                }
                let decl = shape
                    .fields
                    .iter()
                    .find(|field| field.name == *field_name)
                    .ok_or_else(|| {
                        format!("shape '{}' has no field '{}'", name, field_name)
                    })?;
                let field_type = infer_expr_type(
                    field_expr,
                    env,
                    shapes,
                    functions,
                    Some(&decl.ty),
                    list_types,
                )?;
                if field_type != decl.ty {
                    return Err(format!(
                        "shape field '{}.{}' expected '{}', got '{}'",
                        name,
                        field_name,
                        decl.ty.display(),
                        field_type.display()
                    ));
                }
            }
            TypeName::Named(name.clone())
        }
    };

    register_list_types(&ty, list_types);
    Ok(ty)
}

fn infer_call_type(
    name: &str,
    args: &[Expr],
    env: &HashMap<String, TypeName>,
    shapes: &HashMap<String, ShapeDecl>,
    functions: &HashMap<String, FunctionSignature>,
    list_types: &mut HashSet<TypeName>,
) -> Result<TypeName, String> {
    match name {
        "count" => {
            if args.len() != 1 {
                return Err("count(...) expects exactly one argument".to_string());
            }
            let arg_type = infer_expr_type(&args[0], env, shapes, functions, None, list_types)?;
            match arg_type {
                TypeName::Text | TypeName::List(_) => Ok(TypeName::I64),
                other => Err(format!(
                    "count(...) only supports text and list values, got '{}'",
                    other.display()
                )),
            }
        }
        "append" => {
            if args.len() != 2 {
                return Err("append(...) expects exactly two arguments".to_string());
            }
            let list_type = infer_expr_type(&args[0], env, shapes, functions, None, list_types)?;
            match list_type {
                TypeName::List(inner) => {
                    let item_type =
                        infer_expr_type(&args[1], env, shapes, functions, Some(&inner), list_types)?;
                    if item_type != *inner {
                        return Err(format!(
                            "append(...) expected list item type '{}', got '{}'",
                            inner.display(),
                            item_type.display()
                        ));
                    }
                    let result = TypeName::List(inner);
                    register_list_types(&result, list_types);
                    Ok(result)
                }
                other => Err(format!(
                    "append(...) requires a list as its first argument, got '{}'",
                    other.display()
                )),
            }
        }
        "read_text" => {
            if args.len() != 1 {
                return Err("read_text(...) expects exactly one argument".to_string());
            }
            let arg_type = infer_expr_type(&args[0], env, shapes, functions, None, list_types)?;
            if arg_type != TypeName::Text {
                return Err(format!(
                    "read_text(...) expects a text path, got '{}'",
                    arg_type.display()
                ));
            }
            Ok(TypeName::Text)
        }
        "write_text" => {
            if args.len() != 2 {
                return Err("write_text(...) expects exactly two arguments".to_string());
            }
            let path_type = infer_expr_type(&args[0], env, shapes, functions, None, list_types)?;
            let text_type = infer_expr_type(&args[1], env, shapes, functions, None, list_types)?;
            if path_type != TypeName::Text || text_type != TypeName::Text {
                return Err("write_text(...) expects (text, text)".to_string());
            }
            Ok(TypeName::Bool)
        }
        "arg" => {
            if args.len() != 1 {
                return Err("arg(...) expects exactly one argument".to_string());
            }
            let index_type = infer_expr_type(&args[0], env, shapes, functions, None, list_types)?;
            if index_type != TypeName::I64 {
                return Err(format!(
                    "arg(...) expects an i64 index, got '{}'",
                    index_type.display()
                ));
            }
            Ok(TypeName::Text)
        }
        "arg_count" => {
            if !args.is_empty() {
                return Err("arg_count() does not take arguments".to_string());
            }
            Ok(TypeName::I64)
        }
        "text_of" => {
            if args.len() != 1 {
                return Err("text_of(...) expects exactly one argument".to_string());
            }
            let value_type = infer_expr_type(&args[0], env, shapes, functions, None, list_types)?;
            if value_type != TypeName::I64
                && value_type != TypeName::Bool
                && value_type != TypeName::Text
            {
                return Err(format!(
                    "text_of(...) only supports i64, bool, or text values, got '{}'",
                    value_type.display()
                ));
            }
            Ok(TypeName::Text)
        }
        "i64_of" => {
            if args.len() != 1 {
                return Err("i64_of(...) expects exactly one argument".to_string());
            }
            let value_type = infer_expr_type(&args[0], env, shapes, functions, None, list_types)?;
            if value_type != TypeName::Text {
                return Err(format!(
                    "i64_of(...) expects text input, got '{}'",
                    value_type.display()
                ));
            }
            Ok(TypeName::I64)
        }
        "socket_open" => {
            if args.len() != 2 {
                return Err("socket_open(...) expects exactly two arguments".to_string());
            }
            let host_type = infer_expr_type(&args[0], env, shapes, functions, None, list_types)?;
            let port_type = infer_expr_type(&args[1], env, shapes, functions, None, list_types)?;
            if host_type != TypeName::Text || port_type != TypeName::I64 {
                return Err("socket_open(...) expects (text, i64)".to_string());
            }
            Ok(TypeName::Socket)
        }
        "socket_send" => {
            if args.len() != 2 {
                return Err("socket_send(...) expects exactly two arguments".to_string());
            }
            let socket_type =
                infer_expr_type(&args[0], env, shapes, functions, None, list_types)?;
            let text_type = infer_expr_type(&args[1], env, shapes, functions, None, list_types)?;
            if socket_type != TypeName::Socket || text_type != TypeName::Text {
                return Err("socket_send(...) expects (socket, text)".to_string());
            }
            Ok(TypeName::I64)
        }
        "socket_recv" => {
            if args.len() != 2 {
                return Err("socket_recv(...) expects exactly two arguments".to_string());
            }
            let socket_type =
                infer_expr_type(&args[0], env, shapes, functions, None, list_types)?;
            let limit_type = infer_expr_type(&args[1], env, shapes, functions, None, list_types)?;
            if socket_type != TypeName::Socket || limit_type != TypeName::I64 {
                return Err("socket_recv(...) expects (socket, i64)".to_string());
            }
            Ok(TypeName::Text)
        }
        "socket_close" => {
            if args.len() != 1 {
                return Err("socket_close(...) expects exactly one argument".to_string());
            }
            let socket_type =
                infer_expr_type(&args[0], env, shapes, functions, None, list_types)?;
            if socket_type != TypeName::Socket {
                return Err(format!(
                    "socket_close(...) expects socket input, got '{}'",
                    socket_type.display()
                ));
            }
            Ok(TypeName::Bool)
        }
        _ => {
            let signature = functions
                .get(name)
                .ok_or_else(|| format!("unknown function '{}'", name))?;
            if signature.params.len() != args.len() {
                return Err(format!(
                    "function '{}' expects {} arguments, got {}",
                    name,
                    signature.params.len(),
                    args.len()
                ));
            }
            for (arg, expected_type) in args.iter().zip(signature.params.iter()) {
                let actual =
                    infer_expr_type(arg, env, shapes, functions, Some(expected_type), list_types)?;
                if &actual != expected_type {
                    return Err(format!(
                        "function '{}' expected argument type '{}', got '{}'",
                        name,
                        expected_type.display(),
                        actual.display()
                    ));
                }
            }
            Ok(signature.return_type.clone())
        }
    }
}

fn lower_to_c(program: &Program, semantic: &SemanticInfo) -> Result<String, String> {
    let mut out = String::new();

    out.push_str("/* generated by noema-compiler */\n");
    out.push_str("#include <stdbool.h>\n");
    out.push_str("#include <stdint.h>\n");
    out.push_str("#include <stdio.h>\n");
    out.push_str("#include <stdlib.h>\n");
    out.push_str("#include <string.h>\n");
    out.push_str("#include <errno.h>\n");
    out.push_str("#include <netdb.h>\n");
    out.push_str("#include <sys/socket.h>\n");
    out.push_str("#include <sys/types.h>\n");
    out.push_str("#include <unistd.h>\n\n");

    out.push_str("typedef struct {\n");
    out.push_str("    int64_t len;\n");
    out.push_str("    const char *data;\n");
    out.push_str("} NoemaText;\n\n");

    out.push_str("typedef struct {\n");
    out.push_str("    int fd;\n");
    out.push_str("} NoemaSocket;\n\n");

    out.push_str("static int noema_argc = 0;\n");
    out.push_str("static char **noema_argv = NULL;\n\n");

    out.push_str("static void *noema_alloc(size_t size) {\n");
    out.push_str("    void *ptr = malloc(size);\n");
    out.push_str("    if (ptr == NULL) {\n");
    out.push_str("        fprintf(stderr, \"noema runtime: allocation failed\\n\");\n");
    out.push_str("        exit(1);\n");
    out.push_str("    }\n");
    out.push_str("    return ptr;\n");
    out.push_str("}\n\n");

    out.push_str("static NoemaText noema_text_literal(const char *data, int64_t len) {\n");
    out.push_str("    NoemaText text;\n");
    out.push_str("    text.len = len;\n");
    out.push_str("    text.data = data;\n");
    out.push_str("    return text;\n");
    out.push_str("}\n\n");

    out.push_str("static char *noema_text_to_cstr(NoemaText text) {\n");
    out.push_str("    char *buffer = (char *)noema_alloc((size_t)text.len + 1);\n");
    out.push_str("    memcpy(buffer, text.data, (size_t)text.len);\n");
    out.push_str("    buffer[text.len] = '\\0';\n");
    out.push_str("    return buffer;\n");
    out.push_str("}\n\n");

    out.push_str("static int64_t noema_text_count(NoemaText text) {\n");
    out.push_str("    return text.len;\n");
    out.push_str("}\n\n");

    out.push_str("static bool noema_text_eq(NoemaText left, NoemaText right) {\n");
    out.push_str("    if (left.len != right.len) {\n");
    out.push_str("        return false;\n");
    out.push_str("    }\n");
    out.push_str("    return memcmp(left.data, right.data, (size_t)left.len) == 0;\n");
    out.push_str("}\n\n");

    out.push_str("static NoemaText noema_text_concat(NoemaText left, NoemaText right) {\n");
    out.push_str("    char *buffer = (char *)noema_alloc((size_t)(left.len + right.len + 1));\n");
    out.push_str("    memcpy(buffer, left.data, (size_t)left.len);\n");
    out.push_str("    memcpy(buffer + left.len, right.data, (size_t)right.len);\n");
    out.push_str("    buffer[left.len + right.len] = '\\0';\n");
    out.push_str("    return noema_text_literal(buffer, left.len + right.len);\n");
    out.push_str("}\n\n");

    out.push_str("static NoemaText noema_text_from_i64(int64_t value) {\n");
    out.push_str("    char stack_buffer[64];\n");
    out.push_str("    int length = snprintf(stack_buffer, sizeof(stack_buffer), \"%lld\", (long long)value);\n");
    out.push_str("    char *buffer = (char *)noema_alloc((size_t)length + 1);\n");
    out.push_str("    memcpy(buffer, stack_buffer, (size_t)length + 1);\n");
    out.push_str("    return noema_text_literal(buffer, (int64_t)length);\n");
    out.push_str("}\n\n");

    out.push_str("static int64_t noema_i64_of(NoemaText text) {\n");
    out.push_str("    char *buffer = noema_text_to_cstr(text);\n");
    out.push_str("    char *end = NULL;\n");
    out.push_str("    long long value = strtoll(buffer, &end, 10);\n");
    out.push_str("    if (end == buffer || *end != '\\0') {\n");
    out.push_str("        fprintf(stderr, \"noema runtime: invalid integer text '%s'\\n\", buffer);\n");
    out.push_str("        exit(1);\n");
    out.push_str("    }\n");
    out.push_str("    return (int64_t)value;\n");
    out.push_str("}\n\n");

    out.push_str("static NoemaText noema_text_from_bool(bool value) {\n");
    out.push_str("    return value ? noema_text_literal(\"true\", 4) : noema_text_literal(\"false\", 5);\n");
    out.push_str("}\n\n");

    out.push_str("static void noema_emit_i64(int64_t value) {\n");
    out.push_str("    printf(\"%lld\\n\", (long long)value);\n");
    out.push_str("}\n\n");

    out.push_str("static void noema_emit_bool(bool value) {\n");
    out.push_str("    puts(value ? \"true\" : \"false\");\n");
    out.push_str("}\n\n");

    out.push_str("static void noema_emit_text(NoemaText text) {\n");
    out.push_str("    fwrite(text.data, 1, (size_t)text.len, stdout);\n");
    out.push_str("    fputc('\\n', stdout);\n");
    out.push_str("}\n\n");

    out.push_str("static int64_t noema_arg_count(void) {\n");
    out.push_str("    return (int64_t)noema_argc;\n");
    out.push_str("}\n\n");

    out.push_str("static NoemaText noema_arg(int64_t index) {\n");
    out.push_str("    if (index < 0 || index >= noema_argc) {\n");
    out.push_str("        fprintf(stderr, \"noema runtime: argv index out of range\\n\");\n");
    out.push_str("        exit(1);\n");
    out.push_str("    }\n");
    out.push_str("    return noema_text_literal(noema_argv[index], (int64_t)strlen(noema_argv[index]));\n");
    out.push_str("}\n\n");

    out.push_str("static NoemaText noema_read_text(NoemaText path) {\n");
    out.push_str("    char *path_c = noema_text_to_cstr(path);\n");
    out.push_str("    FILE *file = fopen(path_c, \"rb\");\n");
    out.push_str("    if (file == NULL) {\n");
    out.push_str("        fprintf(stderr, \"noema runtime: failed to open input file %s\\n\", path_c);\n");
    out.push_str("        exit(1);\n");
    out.push_str("    }\n");
    out.push_str("    if (fseek(file, 0, SEEK_END) != 0) {\n");
    out.push_str("        fprintf(stderr, \"noema runtime: failed to seek input file %s\\n\", path_c);\n");
    out.push_str("        exit(1);\n");
    out.push_str("    }\n");
    out.push_str("    long size = ftell(file);\n");
    out.push_str("    if (size < 0) {\n");
    out.push_str("        fprintf(stderr, \"noema runtime: failed to read size for %s\\n\", path_c);\n");
    out.push_str("        exit(1);\n");
    out.push_str("    }\n");
    out.push_str("    rewind(file);\n");
    out.push_str("    char *buffer = (char *)noema_alloc((size_t)size + 1);\n");
    out.push_str("    size_t read = fread(buffer, 1, (size_t)size, file);\n");
    out.push_str("    fclose(file);\n");
    out.push_str("    if (read != (size_t)size) {\n");
    out.push_str("        fprintf(stderr, \"noema runtime: failed to read all bytes from %s\\n\", path_c);\n");
    out.push_str("        exit(1);\n");
    out.push_str("    }\n");
    out.push_str("    buffer[size] = '\\0';\n");
    out.push_str("    return noema_text_literal(buffer, (int64_t)size);\n");
    out.push_str("}\n\n");

    out.push_str("static bool noema_write_text(NoemaText path, NoemaText text) {\n");
    out.push_str("    char *path_c = noema_text_to_cstr(path);\n");
    out.push_str("    FILE *file = fopen(path_c, \"wb\");\n");
    out.push_str("    if (file == NULL) {\n");
    out.push_str("        return false;\n");
    out.push_str("    }\n");
    out.push_str("    size_t written = fwrite(text.data, 1, (size_t)text.len, file);\n");
    out.push_str("    fclose(file);\n");
    out.push_str("    return written == (size_t)text.len;\n");
    out.push_str("}\n\n");

    out.push_str("static NoemaSocket noema_socket_open(NoemaText host, int64_t port) {\n");
    out.push_str("    char *host_c = noema_text_to_cstr(host);\n");
    out.push_str("    char port_buffer[32];\n");
    out.push_str("    struct addrinfo hints;\n");
    out.push_str("    struct addrinfo *result = NULL;\n");
    out.push_str("    struct addrinfo *cursor = NULL;\n");
    out.push_str("    int fd = -1;\n");
    out.push_str("    NoemaSocket socket_value;\n");
    out.push_str("    memset(&hints, 0, sizeof(hints));\n");
    out.push_str("    hints.ai_socktype = SOCK_STREAM;\n");
    out.push_str("    hints.ai_family = AF_UNSPEC;\n");
    out.push_str("    snprintf(port_buffer, sizeof(port_buffer), \"%lld\", (long long)port);\n");
    out.push_str("    if (getaddrinfo(host_c, port_buffer, &hints, &result) != 0) {\n");
    out.push_str("        fprintf(stderr, \"noema runtime: getaddrinfo failed for %s:%s\\n\", host_c, port_buffer);\n");
    out.push_str("        exit(1);\n");
    out.push_str("    }\n");
    out.push_str("    for (cursor = result; cursor != NULL; cursor = cursor->ai_next) {\n");
    out.push_str("        fd = socket(cursor->ai_family, cursor->ai_socktype, cursor->ai_protocol);\n");
    out.push_str("        if (fd < 0) {\n");
    out.push_str("            continue;\n");
    out.push_str("        }\n");
    out.push_str("        if (connect(fd, cursor->ai_addr, cursor->ai_addrlen) == 0) {\n");
    out.push_str("            break;\n");
    out.push_str("        }\n");
    out.push_str("        close(fd);\n");
    out.push_str("        fd = -1;\n");
    out.push_str("    }\n");
    out.push_str("    freeaddrinfo(result);\n");
    out.push_str("    if (fd < 0) {\n");
    out.push_str("        fprintf(stderr, \"noema runtime: failed to connect to %s:%s\\n\", host_c, port_buffer);\n");
    out.push_str("        exit(1);\n");
    out.push_str("    }\n");
    out.push_str("    socket_value.fd = fd;\n");
    out.push_str("    return socket_value;\n");
    out.push_str("}\n\n");

    out.push_str("static int64_t noema_socket_send(NoemaSocket socket_value, NoemaText text) {\n");
    out.push_str("    int64_t total = 0;\n");
    out.push_str("    while (total < text.len) {\n");
    out.push_str("        ssize_t wrote = send(socket_value.fd, text.data + total, (size_t)(text.len - total), 0);\n");
    out.push_str("        if (wrote <= 0) {\n");
    out.push_str("            fprintf(stderr, \"noema runtime: socket send failed\\n\");\n");
    out.push_str("            exit(1);\n");
    out.push_str("        }\n");
    out.push_str("        total += (int64_t)wrote;\n");
    out.push_str("    }\n");
    out.push_str("    return total;\n");
    out.push_str("}\n\n");

    out.push_str("static NoemaText noema_socket_recv(NoemaSocket socket_value, int64_t limit) {\n");
    out.push_str("    if (limit < 0) {\n");
    out.push_str("        fprintf(stderr, \"noema runtime: socket_recv limit must be non-negative\\n\");\n");
    out.push_str("        exit(1);\n");
    out.push_str("    }\n");
    out.push_str("    char *buffer = (char *)noema_alloc((size_t)limit + 1);\n");
    out.push_str("    ssize_t got = recv(socket_value.fd, buffer, (size_t)limit, 0);\n");
    out.push_str("    if (got < 0) {\n");
    out.push_str("        fprintf(stderr, \"noema runtime: socket recv failed\\n\");\n");
    out.push_str("        exit(1);\n");
    out.push_str("    }\n");
    out.push_str("    buffer[got] = '\\0';\n");
    out.push_str("    return noema_text_literal(buffer, (int64_t)got);\n");
    out.push_str("}\n\n");

    out.push_str("static bool noema_socket_close(NoemaSocket socket_value) {\n");
    out.push_str("    return close(socket_value.fd) == 0;\n");
    out.push_str("}\n\n");

    for shape in &program.shapes {
        writeln!(&mut out, "typedef struct {} {{", shape.name).unwrap();
        for field in &shape.fields {
            writeln!(
                &mut out,
                "    {} {};",
                c_type_name(&field.ty),
                field.name
            )
            .unwrap();
        }
        writeln!(&mut out, "}} {};\n", shape.name).unwrap();
    }

    let mut ordered_lists: Vec<TypeName> = semantic.list_types.iter().cloned().collect();
    ordered_lists.sort_by_key(TypeName::mangle);
    for list_type in &ordered_lists {
        emit_list_helpers(list_type, &mut out)?;
    }

    for function in &program.functions {
        write!(
            &mut out,
            "static {} {}(",
            c_type_name(&function.return_type),
            c_function_name(&function.name)
        )
        .unwrap();
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
        out.push_str(");\n");
    }
    out.push('\n');

    for function in &program.functions {
        lower_function(function, semantic, &mut out)?;
    }

    out.push_str("int main(int argc, char **argv) {\n");
    out.push_str("    noema_argc = argc;\n");
    out.push_str("    noema_argv = argv;\n");
    out.push_str("    return (int)codex_main();\n");
    out.push_str("}\n");

    Ok(out)
}

fn emit_list_helpers(list_type: &TypeName, out: &mut String) -> Result<(), String> {
    let TypeName::List(inner) = list_type else {
        return Ok(());
    };

    let struct_name = list_struct_name(inner);
    let item_type = c_type_name(inner);
    let append_name = list_append_name(inner);
    let count_name = list_count_name(inner);
    let at_name = list_at_name(inner);
    let make_name = list_make_name(inner);
    let from_items_name = list_from_items_name(inner);

    writeln!(out, "typedef struct {} {{", struct_name).unwrap();
    writeln!(out, "    {} *items;", item_type).unwrap();
    out.push_str("    int64_t len;\n");
    out.push_str("} ");
    out.push_str(&struct_name);
    out.push_str(";\n\n");

    writeln!(out, "static {} {}(void) {{", struct_name, make_name).unwrap();
    writeln!(out, "    {} list;", struct_name).unwrap();
    out.push_str("    list.items = NULL;\n");
    out.push_str("    list.len = 0;\n");
    out.push_str("    return list;\n");
    out.push_str("}\n\n");

    writeln!(
        out,
        "static {} {}(const {} *items, int64_t len) {{",
        struct_name,
        from_items_name,
        item_type
    )
    .unwrap();
    writeln!(out, "    {} list = {}();", struct_name, make_name).unwrap();
    out.push_str("    if (len <= 0) {\n");
    out.push_str("        return list;\n");
    out.push_str("    }\n");
    writeln!(
        out,
        "    list.items = ({} *)noema_alloc(sizeof({}) * (size_t)len);",
        item_type,
        item_type
    )
    .unwrap();
    writeln!(
        out,
        "    memcpy(list.items, items, sizeof({}) * (size_t)len);",
        item_type
    )
    .unwrap();
    out.push_str("    list.len = len;\n");
    out.push_str("    return list;\n");
    out.push_str("}\n\n");

    writeln!(
        out,
        "static {} {}({} list, {} value) {{",
        struct_name,
        append_name,
        struct_name,
        item_type
    )
    .unwrap();
    writeln!(out, "    {} result;", struct_name).unwrap();
    writeln!(
        out,
        "    result.items = ({} *)noema_alloc(sizeof({}) * (size_t)(list.len + 1));",
        item_type,
        item_type
    )
    .unwrap();
    out.push_str("    if (list.len > 0) {\n");
    writeln!(
        out,
        "        memcpy(result.items, list.items, sizeof({}) * (size_t)list.len);",
        item_type
    )
    .unwrap();
    out.push_str("    }\n");
    out.push_str("    result.items[list.len] = value;\n");
    out.push_str("    result.len = list.len + 1;\n");
    out.push_str("    return result;\n");
    out.push_str("}\n\n");

    writeln!(out, "static int64_t {}({} list) {{", count_name, struct_name).unwrap();
    out.push_str("    return list.len;\n");
    out.push_str("}\n\n");

    writeln!(
        out,
        "static {} {}({} list, int64_t index) {{",
        item_type,
        at_name,
        struct_name
    )
    .unwrap();
    out.push_str("    if (index < 0 || index >= list.len) {\n");
    out.push_str("        fprintf(stderr, \"noema runtime: list index out of range\\n\");\n");
    out.push_str("        exit(1);\n");
    out.push_str("    }\n");
    out.push_str("    return list.items[index];\n");
    out.push_str("}\n\n");

    Ok(())
}

fn lower_function(
    function: &Function,
    semantic: &SemanticInfo,
    out: &mut String,
) -> Result<(), String> {
    write!(
        out,
        "static {} {}(",
        c_type_name(&function.return_type),
        c_function_name(&function.name)
    )
    .unwrap();
    if function.params.is_empty() {
        out.push_str("void");
    } else {
        for (index, param) in function.params.iter().enumerate() {
            if index > 0 {
                out.push_str(", ");
            }
            write!(
                out,
                "{} {}",
                c_type_name(&param.ty),
                param.name
            )
            .unwrap();
        }
    }
    out.push_str(") {\n");

    let mut env = HashMap::new();
    for param in &function.params {
        env.insert(param.name.clone(), param.ty.clone());
    }

    lower_block(
        &function.body,
        &mut env,
        semantic,
        out,
        1,
        &function.return_type,
    )?;
    out.push_str("}\n\n");
    Ok(())
}

fn lower_block(
    statements: &[Statement],
    env: &mut HashMap<String, TypeName>,
    semantic: &SemanticInfo,
    out: &mut String,
    indent: usize,
    return_type: &TypeName,
) -> Result<(), String> {
    for statement in statements {
        lower_statement(statement, env, semantic, out, indent, return_type)?;
    }
    Ok(())
}

fn lower_statement(
    statement: &Statement,
    env: &mut HashMap<String, TypeName>,
    semantic: &SemanticInfo,
    out: &mut String,
    indent: usize,
    return_type: &TypeName,
) -> Result<(), String> {
    let prefix = "    ".repeat(indent);

    match statement {
        Statement::Let {
            name,
            annotation,
            expr,
        } => {
            let expr_type = infer_expr_type(
                expr,
                env,
                &semantic.shapes,
                &semantic.functions,
                annotation.as_ref(),
                &mut HashSet::new(),
            )?;
            writeln!(
                out,
                "{}{} {} = {};",
                prefix,
                c_type_name(&expr_type),
                name,
                lower_expr(expr, env, semantic, annotation.as_ref())?
            )
            .unwrap();
            env.insert(name.clone(), expr_type);
        }
        Statement::Assign { target, expr } => {
            let target_type = infer_target_type(target, env, &semantic.shapes)?;
            let lowered_target = lower_target(target)?;
            let lowered_expr = lower_expr(expr, env, semantic, Some(&target_type))?;
            writeln!(out, "{}{} = {};", prefix, lowered_target, lowered_expr).unwrap();
        }
        Statement::Emit(expr) => {
            let expr_type = infer_expr_type(
                expr,
                env,
                &semantic.shapes,
                &semantic.functions,
                None,
                &mut HashSet::new(),
            )?;
            let lowered = lower_expr(expr, env, semantic, None)?;
            match expr_type {
                TypeName::I64 => writeln!(out, "{}noema_emit_i64({});", prefix, lowered).unwrap(),
                TypeName::Bool => writeln!(out, "{}noema_emit_bool({});", prefix, lowered).unwrap(),
                TypeName::Text => writeln!(out, "{}noema_emit_text({});", prefix, lowered).unwrap(),
                _ => unreachable!(),
            }
        }
        Statement::Return(Some(expr)) => {
            writeln!(
                out,
                "{}return {};",
                prefix,
                lower_expr(expr, env, semantic, Some(return_type))?
            )
            .unwrap();
        }
        Statement::Return(None) => {
            writeln!(out, "{}return;", prefix).unwrap();
        }
        Statement::Expr(expr) => {
            writeln!(out, "{}{};", prefix, lower_expr(expr, env, semantic, None)?).unwrap();
        }
        Statement::If {
            condition,
            then_body,
            else_body,
        } => {
            writeln!(
                out,
                "{}if ({}) {{",
                prefix,
                lower_expr(condition, env, semantic, None)?
            )
            .unwrap();
            let mut then_env = env.clone();
            lower_block(
                then_body,
                &mut then_env,
                semantic,
                out,
                indent + 1,
                return_type,
            )?;
            if else_body.is_empty() {
                writeln!(out, "{}}}", prefix).unwrap();
            } else {
                writeln!(out, "{}}} else {{", prefix).unwrap();
                let mut else_env = env.clone();
                lower_block(
                    else_body,
                    &mut else_env,
                    semantic,
                    out,
                    indent + 1,
                    return_type,
                )?;
                writeln!(out, "{}}}", prefix).unwrap();
            }
        }
        Statement::While { condition, body } => {
            writeln!(
                out,
                "{}while ({}) {{",
                prefix,
                lower_expr(condition, env, semantic, None)?
            )
            .unwrap();
            let mut loop_env = env.clone();
            lower_block(body, &mut loop_env, semantic, out, indent + 1, return_type)?;
            writeln!(out, "{}}}", prefix).unwrap();
        }
    }

    Ok(())
}

fn lower_target(target: &AssignTarget) -> Result<String, String> {
    Ok(match target {
        AssignTarget::Name(name) => name.clone(),
        AssignTarget::Field { base, field } => format!("({}).{}", lower_target(base)?, field),
    })
}

fn lower_expr(
    expr: &Expr,
    env: &HashMap<String, TypeName>,
    semantic: &SemanticInfo,
    expected: Option<&TypeName>,
) -> Result<String, String> {
    Ok(match expr {
        Expr::Int(value) => value.to_string(),
        Expr::Bool(value) => {
            if *value {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Expr::Text(value) => format!(
            "noema_text_literal(\"{}\", {})",
            escape_c_string(value),
            value.len()
        ),
        Expr::Name(name) => name.clone(),
        Expr::Unary { op, expr } => match op {
            UnaryOp::Neg => format!("(-{})", lower_expr(expr, env, semantic, None)?),
            UnaryOp::Not => format!("(!{})", lower_expr(expr, env, semantic, None)?),
        },
        Expr::Binary { left, op, right } => {
            let left_type = infer_expr_type(
                left,
                env,
                &semantic.shapes,
                &semantic.functions,
                None,
                &mut HashSet::new(),
            )?;
            let left_expr = lower_expr(left, env, semantic, None)?;
            let right_expr = lower_expr(right, env, semantic, Some(&left_type))?;
            match op {
                BinaryOp::Add if left_type == TypeName::Text => {
                    format!("noema_text_concat({}, {})", left_expr, right_expr)
                }
                BinaryOp::Eq if left_type == TypeName::Text => {
                    format!("noema_text_eq({}, {})", left_expr, right_expr)
                }
                BinaryOp::Ne if left_type == TypeName::Text => {
                    format!("(!noema_text_eq({}, {}))", left_expr, right_expr)
                }
                BinaryOp::Add => format!("({} + {})", left_expr, right_expr),
                BinaryOp::Sub => format!("({} - {})", left_expr, right_expr),
                BinaryOp::Mul => format!("({} * {})", left_expr, right_expr),
                BinaryOp::Div => format!("({} / {})", left_expr, right_expr),
                BinaryOp::Mod => format!("({} % {})", left_expr, right_expr),
                BinaryOp::Eq => format!("({} == {})", left_expr, right_expr),
                BinaryOp::Ne => format!("({} != {})", left_expr, right_expr),
                BinaryOp::Lt => format!("({} < {})", left_expr, right_expr),
                BinaryOp::Le => format!("({} <= {})", left_expr, right_expr),
                BinaryOp::Gt => format!("({} > {})", left_expr, right_expr),
                BinaryOp::Ge => format!("({} >= {})", left_expr, right_expr),
                BinaryOp::And => format!("({} && {})", left_expr, right_expr),
                BinaryOp::Or => format!("({} || {})", left_expr, right_expr),
            }
        }
        Expr::Call { name, args } => lower_call(name, args, env, semantic)?,
        Expr::Field { base, field } => format!("({}).{}", lower_expr(base, env, semantic, None)?, field),
        Expr::Index { base, index } => {
            let base_type = infer_expr_type(
                base,
                env,
                &semantic.shapes,
                &semantic.functions,
                None,
                &mut HashSet::new(),
            )?;
            let TypeName::List(inner) = base_type else {
                return Err("index lowering requires list type".to_string());
            };
            format!(
                "{}({}, {})",
                list_at_name(&inner),
                lower_expr(base, env, semantic, None)?,
                lower_expr(index, env, semantic, Some(&TypeName::I64))?
            )
        }
        Expr::ListLiteral(items) => {
            let list_type = infer_expr_type(
                expr,
                env,
                &semantic.shapes,
                &semantic.functions,
                expected,
                &mut HashSet::new(),
            )?;
            let TypeName::List(inner) = list_type else {
                return Err("list literal lowering requires list type".to_string());
            };
            if items.is_empty() {
                format!("{}()", list_make_name(&inner))
            } else {
                let lowered_items = items
                    .iter()
                    .map(|item| lower_expr(item, env, semantic, Some(&inner)))
                    .collect::<Result<Vec<_>, _>>()?;
                format!(
                    "{}(({}[]){{{}}}, {})",
                    list_from_items_name(&inner),
                    c_type_name(&inner),
                    lowered_items.join(", "),
                    items.len()
                )
            }
        }
        Expr::StructLiteral { name, fields } => {
            let lowered_fields = fields
                .iter()
                .map(|(field, value)| {
                    Ok(format!(
                        ".{} = {}",
                        field,
                        lower_expr(value, env, semantic, None)?
                    ))
                })
                .collect::<Result<Vec<_>, String>>()?;
            format!("({}){{{}}}", name, lowered_fields.join(", "))
        }
    })
}

fn lower_call(
    name: &str,
    args: &[Expr],
    env: &HashMap<String, TypeName>,
    semantic: &SemanticInfo,
) -> Result<String, String> {
    match name {
        "count" => {
            let arg_type = infer_expr_type(
                &args[0],
                env,
                &semantic.shapes,
                &semantic.functions,
                None,
                &mut HashSet::new(),
            )?;
            let arg_expr = lower_expr(&args[0], env, semantic, None)?;
            match arg_type {
                TypeName::Text => Ok(format!("noema_text_count({})", arg_expr)),
                TypeName::List(inner) => Ok(format!("{}({})", list_count_name(&inner), arg_expr)),
                _ => Err("count lowering requires text or list".to_string()),
            }
        }
        "append" => {
            let list_type = infer_expr_type(
                &args[0],
                env,
                &semantic.shapes,
                &semantic.functions,
                None,
                &mut HashSet::new(),
            )?;
            let TypeName::List(inner) = list_type else {
                return Err("append lowering requires a list".to_string());
            };
            Ok(format!(
                "{}({}, {})",
                list_append_name(&inner),
                lower_expr(&args[0], env, semantic, None)?,
                lower_expr(&args[1], env, semantic, Some(&inner))?
            ))
        }
        "read_text" => Ok(format!(
            "noema_read_text({})",
            lower_expr(&args[0], env, semantic, Some(&TypeName::Text))?
        )),
        "write_text" => Ok(format!(
            "noema_write_text({}, {})",
            lower_expr(&args[0], env, semantic, Some(&TypeName::Text))?,
            lower_expr(&args[1], env, semantic, Some(&TypeName::Text))?
        )),
        "arg" => Ok(format!(
            "noema_arg({})",
            lower_expr(&args[0], env, semantic, Some(&TypeName::I64))?
        )),
        "arg_count" => Ok("noema_arg_count()".to_string()),
        "text_of" => {
            let ty = infer_expr_type(
                &args[0],
                env,
                &semantic.shapes,
                &semantic.functions,
                None,
                &mut HashSet::new(),
            )?;
            let arg_expr = lower_expr(&args[0], env, semantic, None)?;
            match ty {
                TypeName::I64 => Ok(format!("noema_text_from_i64({})", arg_expr)),
                TypeName::Bool => Ok(format!("noema_text_from_bool({})", arg_expr)),
                TypeName::Text => Ok(arg_expr),
                _ => Err("text_of lowering only supports i64, bool, or text".to_string()),
            }
        }
        "i64_of" => Ok(format!(
            "noema_i64_of({})",
            lower_expr(&args[0], env, semantic, Some(&TypeName::Text))?
        )),
        "socket_open" => Ok(format!(
            "noema_socket_open({}, {})",
            lower_expr(&args[0], env, semantic, Some(&TypeName::Text))?,
            lower_expr(&args[1], env, semantic, Some(&TypeName::I64))?
        )),
        "socket_send" => Ok(format!(
            "noema_socket_send({}, {})",
            lower_expr(&args[0], env, semantic, Some(&TypeName::Socket))?,
            lower_expr(&args[1], env, semantic, Some(&TypeName::Text))?
        )),
        "socket_recv" => Ok(format!(
            "noema_socket_recv({}, {})",
            lower_expr(&args[0], env, semantic, Some(&TypeName::Socket))?,
            lower_expr(&args[1], env, semantic, Some(&TypeName::I64))?
        )),
        "socket_close" => Ok(format!(
            "noema_socket_close({})",
            lower_expr(&args[0], env, semantic, Some(&TypeName::Socket))?
        )),
        _ => {
            let signature = semantic
                .functions
                .get(name)
                .ok_or_else(|| format!("unknown function '{}'", name))?;
            let lowered_args = args
                .iter()
                .zip(signature.params.iter())
                .map(|(arg, ty)| lower_expr(arg, env, semantic, Some(ty)))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(format!(
                "{}({})",
                c_function_name(name),
                lowered_args.join(", ")
            ))
        }
    }
}

fn c_function_name(name: &str) -> String {
    if name == "main" {
        "codex_main".to_string()
    } else {
        name.to_string()
    }
}

fn c_type_name(ty: &TypeName) -> String {
    match ty {
        TypeName::I64 => "int64_t".to_string(),
        TypeName::Bool => "bool".to_string(),
        TypeName::Text => "NoemaText".to_string(),
        TypeName::Socket => "NoemaSocket".to_string(),
        TypeName::Void => "void".to_string(),
        TypeName::Named(name) => name.clone(),
        TypeName::List(inner) => list_struct_name(inner),
    }
}

fn list_struct_name(inner: &TypeName) -> String {
    format!("NoemaList_{}", inner.mangle())
}

fn list_make_name(inner: &TypeName) -> String {
    format!("noema_list_make_{}", inner.mangle())
}

fn list_from_items_name(inner: &TypeName) -> String {
    format!("noema_list_from_items_{}", inner.mangle())
}

fn list_append_name(inner: &TypeName) -> String {
    format!("noema_list_append_{}", inner.mangle())
}

fn list_count_name(inner: &TypeName) -> String {
    format!("noema_list_count_{}", inner.mangle())
}

fn list_at_name(inner: &TypeName) -> String {
    format!("noema_list_at_{}", inner.mangle())
}

impl TypeName {
    fn display(&self) -> String {
        match self {
            TypeName::I64 => "i64".to_string(),
            TypeName::Bool => "bool".to_string(),
            TypeName::Text => "text".to_string(),
            TypeName::Socket => "socket".to_string(),
            TypeName::Void => "void".to_string(),
            TypeName::Named(name) => name.clone(),
            TypeName::List(inner) => format!("list<{}>", inner.display()),
        }
    }

    fn mangle(&self) -> String {
        match self {
            TypeName::I64 => "i64".to_string(),
            TypeName::Bool => "bool".to_string(),
            TypeName::Text => "text".to_string(),
            TypeName::Socket => "socket".to_string(),
            TypeName::Void => "void".to_string(),
            TypeName::Named(name) => name.to_lowercase(),
            TypeName::List(inner) => format!("list_{}", inner.mangle()),
        }
    }
}

fn escape_c_string(input: &str) -> String {
    let mut output = String::new();
    for ch in input.chars() {
        match ch {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            other => output.push(other),
        }
    }
    output
}
