//! Tree-walking interpreter for the Virtual Rust VM.
//!
//! # Architecture
//!
//! The interpreter is split into focused sub-modules:
//!
//! | Module          | Responsibility                                    |
//! |-----------------|---------------------------------------------------|
//! | [`value`]       | Runtime value types (`Value` enum)                 |
//! | [`environment`] | Variable scoping (scope stack)                     |
//! | [`error`]       | `RuntimeError` definition                          |
//! | [`builtins`]    | Built-in functions & macro dispatch                |
//! | [`methods`]     | Method call dispatch (String, Array, Int, …)       |
//! | [`format`]      | Format-string processing (`{}`, `{:?}`, `{:.2}`)  |
//! | [`pattern`]     | Pattern matching for `match` expressions           |
//! | [`cast`]        | Type casting (`as` expressions)                    |

pub mod builtins;
pub mod cast;
pub mod environment;
pub mod error;
pub mod format;
pub mod methods;
pub mod pattern;
pub mod value;

pub use environment::Environment;
pub use error::RuntimeError;
pub use value::Value;

use std::collections::HashMap;

use crate::ast::*;

/// The tree-walking interpreter that evaluates an AST.
///
/// Maintains an [`Environment`] for variable storage and a registry
/// of struct definitions encountered during execution.
pub struct Interpreter {
    pub env: Environment,
    struct_defs: HashMap<String, Vec<(String, Type)>>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    /// Creates a new interpreter with a fresh environment.
    pub fn new() -> Self {
        let mut interp = Interpreter {
            env: Environment::new(),
            struct_defs: HashMap::new(),
        };
        // Register built-in constants
        interp
            .env
            .define("None".to_string(), Value::Option(None), false);
        interp
    }

    /// Runs a complete program (list of top-level statements).
    pub fn run(&mut self, program: Vec<Expr>) -> Result<Value, RuntimeError> {
        let mut result = Value::Unit;
        for expr in program {
            result = self.eval(&expr)?;
            if let Value::Return(val) = result {
                return Ok(val.map(|v| *v).unwrap_or(Value::Unit));
            }
        }
        Ok(result)
    }

    // ── Core evaluation ──────────────────────────────────────────────

    /// Evaluates a single expression / statement.
    pub fn eval(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            // ── Literals ─────────────────────────────────────────────
            Expr::IntLiteral(n) => Ok(Value::Int(*n)),
            Expr::FloatLiteral(n) => Ok(Value::Float(*n)),
            Expr::StringLiteral(s) => Ok(Value::String(s.clone())),
            Expr::CharLiteral(c) => Ok(Value::Char(*c)),
            Expr::BoolLiteral(b) => Ok(Value::Bool(*b)),
            Expr::Unit => Ok(Value::Unit),

            // ── Identifiers ──────────────────────────────────────────
            Expr::Ident(name) => self.eval_ident(name),

            // ── Operators ────────────────────────────────────────────
            Expr::BinaryOp { left, op, right } => {
                let lv = self.eval(left)?;
                let rv = self.eval(right)?;
                self.eval_binary_op(&lv, op, &rv)
            }
            Expr::UnaryOp { op, expr } => {
                let val = self.eval(expr)?;
                self.eval_unary_op(op, &val)
            }

            // ── Bindings & assignment ────────────────────────────────
            Expr::Let {
                name,
                mutable,
                value,
                ..
            } => self.eval_let(name, *mutable, value.as_deref()),

            Expr::Assign { target, value } => {
                let val = self.eval(value)?;
                self.assign_to(target, val)?;
                Ok(Value::Unit)
            }
            Expr::CompoundAssign { target, op, value } => {
                let current = self.eval(target)?;
                let rhs = self.eval(value)?;
                let result = self.eval_binary_op(&current, op, &rhs)?;
                self.assign_to(target, result)?;
                Ok(Value::Unit)
            }

            // ── Control flow ─────────────────────────────────────────
            Expr::Block(stmts) => self.eval_block(stmts),
            Expr::If {
                condition,
                then_block,
                else_block,
            } => self.eval_if(condition, then_block, else_block.as_deref()),
            Expr::While { condition, body } => self.eval_while(condition, body),
            Expr::Loop { body } => self.eval_loop(body),
            Expr::For {
                var,
                iterator,
                body,
            } => self.eval_for(var, iterator, body),
            Expr::Range {
                start,
                end,
                inclusive,
            } => self.eval_range(start.as_deref(), end.as_deref(), *inclusive),
            Expr::Break(val) => {
                let v = val.as_ref().map(|e| self.eval(e)).transpose()?;
                Ok(Value::Break(v.map(Box::new)))
            }
            Expr::Continue => Ok(Value::Continue),
            Expr::Return(val) => {
                let v = val.as_ref().map(|e| self.eval(e)).transpose()?;
                Ok(Value::Return(v.map(Box::new)))
            }

            // ── Functions & calls ────────────────────────────────────
            Expr::FnDef {
                name, params, body, ..
            } => self.eval_fn_def(name, params, body),

            Expr::FnCall { name, args } => self.eval_fn_call(name, args),

            Expr::MethodCall {
                object,
                method,
                args,
            } => {
                let obj = self.eval(object)?;
                let evaluated_args = self.eval_slice(args)?;
                self.call_method(obj, method, evaluated_args)
            }
            Expr::MacroCall { name, args } => self.call_macro(name, args),

            // ── Data structures ──────────────────────────────────────
            Expr::ArrayLiteral(elements) => {
                let values = self.eval_slice(elements)?;
                Ok(Value::Array(values))
            }
            Expr::ArrayRepeat { value, count } => self.eval_array_repeat(value, count),
            Expr::TupleLiteral(elements) => {
                let values = self.eval_slice(elements)?;
                Ok(Value::Tuple(values))
            }
            Expr::VecMacro(elements) => {
                let values = self.eval_slice(elements)?;
                Ok(Value::Array(values))
            }

            // ── Indexing & field access ──────────────────────────────
            Expr::Index { object, index } => self.eval_index(object, index),
            Expr::FieldAccess { object, field } => self.eval_field_access(object, field),

            // ── Structs ──────────────────────────────────────────────
            Expr::StructDef { name, fields } => {
                self.struct_defs.insert(name.clone(), fields.clone());
                Ok(Value::Unit)
            }
            Expr::StructInit { name, fields } => self.eval_struct_init(name, fields),

            // ── Match ────────────────────────────────────────────────
            Expr::Match { expr, arms } => self.eval_match(expr, arms),

            // ── Type cast ────────────────────────────────────────────
            Expr::TypeCast { expr, target_type } => {
                let val = self.eval(expr)?;
                cast::type_cast(val, target_type)
            }

            // ── Closures & references ────────────────────────────────
            Expr::Closure { params, body } => Ok(Value::Closure {
                params: params.clone(),
                body: body.clone(),
                env: self.env.clone(),
            }),
            Expr::Ref { expr, .. } | Expr::Deref(expr) => self.eval(expr),
        }
    }

    // ── Evaluation helpers ───────────────────────────────────────────

    fn eval_ident(&self, name: &str) -> Result<Value, RuntimeError> {
        if name == "Some" {
            return Ok(Value::Function {
                name: "Some".to_string(),
                params: vec![("value".to_string(), Type::Inferred)],
                body: Box::new(Expr::Ident("value".to_string())),
                closure_env: None,
            });
        }
        self.env
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeError::new(format!("Undefined variable: '{name}'")))
    }

    fn eval_let(
        &mut self,
        name: &str,
        mutable: bool,
        value: Option<&Expr>,
    ) -> Result<Value, RuntimeError> {
        let val = match value {
            Some(expr) => {
                let v = self.eval(expr)?;
                if let Value::Return(_) = &v {
                    return Ok(v);
                }
                v
            }
            None => Value::Unit,
        };
        self.env.define(name.to_string(), val, mutable);
        Ok(Value::Unit)
    }

    fn eval_block(&mut self, stmts: &[Expr]) -> Result<Value, RuntimeError> {
        self.env.push_scope();
        let mut result = Value::Unit;
        for stmt in stmts {
            result = self.eval(stmt)?;
            if matches!(&result, Value::Return(_) | Value::Break(_) | Value::Continue) {
                self.env.pop_scope();
                return Ok(result);
            }
        }
        self.env.pop_scope();
        Ok(result)
    }

    fn eval_if(
        &mut self,
        condition: &Expr,
        then_block: &Expr,
        else_block: Option<&Expr>,
    ) -> Result<Value, RuntimeError> {
        let cond = self.eval(condition)?;
        if cond.is_truthy() {
            self.eval(then_block)
        } else if let Some(else_b) = else_block {
            self.eval(else_b)
        } else {
            Ok(Value::Unit)
        }
    }

    fn eval_while(&mut self, condition: &Expr, body: &Expr) -> Result<Value, RuntimeError> {
        loop {
            if !self.eval(condition)?.is_truthy() {
                break;
            }
            match self.eval(body)? {
                Value::Break(v) => return Ok(v.map(|v| *v).unwrap_or(Value::Unit)),
                Value::Continue => continue,
                Value::Return(v) => return Ok(Value::Return(v)),
                _ => {}
            }
        }
        Ok(Value::Unit)
    }

    fn eval_loop(&mut self, body: &Expr) -> Result<Value, RuntimeError> {
        loop {
            match self.eval(body)? {
                Value::Break(v) => return Ok(v.map(|v| *v).unwrap_or(Value::Unit)),
                Value::Continue => continue,
                Value::Return(v) => return Ok(Value::Return(v)),
                _ => {}
            }
        }
    }

    fn eval_for(
        &mut self,
        var: &str,
        iterator: &Expr,
        body: &Expr,
    ) -> Result<Value, RuntimeError> {
        let iter_val = self.eval(iterator)?;
        let items = value_to_iterator(iter_val)?;
        for item in items {
            self.env.push_scope();
            self.env.define(var.to_string(), item, true);
            let result = self.eval(body)?;
            self.env.pop_scope();
            match result {
                Value::Break(v) => return Ok(v.map(|v| *v).unwrap_or(Value::Unit)),
                Value::Continue => continue,
                Value::Return(v) => return Ok(Value::Return(v)),
                _ => {}
            }
        }
        Ok(Value::Unit)
    }

    fn eval_range(
        &mut self,
        start: Option<&Expr>,
        end: Option<&Expr>,
        inclusive: bool,
    ) -> Result<Value, RuntimeError> {
        let start_val = match start {
            Some(e) => expect_int(self.eval(e)?, "Range start")?,
            None => 0,
        };
        match end {
            Some(e) => {
                let end_val = expect_int(self.eval(e)?, "Range end")?;
                let items: Vec<Value> = if inclusive {
                    (start_val..=end_val).map(Value::Int).collect()
                } else {
                    (start_val..end_val).map(Value::Int).collect()
                };
                Ok(Value::Array(items))
            }
            None => Ok(Value::Array(Vec::new())),
        }
    }

    // ── Function definitions & calls ─────────────────────────────────

    fn eval_fn_def(
        &mut self,
        name: &str,
        params: &[(String, Type)],
        body: &Expr,
    ) -> Result<Value, RuntimeError> {
        let func = Value::Function {
            name: name.to_string(),
            params: params.to_vec(),
            body: Box::new(body.clone()),
            closure_env: None,
        };
        self.env.define(name.to_string(), func, false);
        Ok(Value::Unit)
    }

    fn eval_fn_call(&mut self, name: &str, args: &[Expr]) -> Result<Value, RuntimeError> {
        let evaluated_args = self.eval_slice(args)?;

        // Try built-ins first
        if let Some(result) = self.call_builtin(name, &evaluated_args)? {
            return Ok(result);
        }

        let func = self
            .env
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeError::new(format!("Undefined function: '{name}'")))?;

        self.call_function(func, evaluated_args)
    }

    /// Calls a function or closure value with the given arguments.
    pub(crate) fn call_function(
        &mut self,
        func: Value,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        match func {
            Value::Function {
                params,
                body,
                closure_env,
                ..
            } => {
                let saved_env = self.env.clone();
                if let Some(env) = closure_env {
                    self.env = env;
                }
                self.env.push_scope();
                for (i, (param_name, _)) in params.iter().enumerate() {
                    let val = args.get(i).cloned().unwrap_or(Value::Unit);
                    self.env.define(param_name.clone(), val, true);
                }
                let result = self.eval(&body)?;
                self.env.pop_scope();
                self.env = saved_env;
                unwrap_return(result)
            }
            Value::Closure {
                params, body, env, ..
            } => {
                let saved_env = self.env.clone();
                self.env = env;
                self.env.push_scope();
                for (i, (param_name, _)) in params.iter().enumerate() {
                    let val = args.get(i).cloned().unwrap_or(Value::Unit);
                    self.env.define(param_name.clone(), val, true);
                }
                let result = self.eval(&body)?;
                self.env.pop_scope();
                self.env = saved_env;
                unwrap_return(result)
            }
            _ => Err(RuntimeError::new(format!(
                "'{}' is not callable",
                func.type_name()
            ))),
        }
    }

    // ── Data structure evaluation ────────────────────────────────────

    fn eval_array_repeat(
        &mut self,
        value: &Expr,
        count: &Expr,
    ) -> Result<Value, RuntimeError> {
        let val = self.eval(value)?;
        let cnt = expect_int(self.eval(count)?, "Array repeat count")? as usize;
        Ok(Value::Array(vec![val; cnt]))
    }

    fn eval_index(&mut self, object: &Expr, index: &Expr) -> Result<Value, RuntimeError> {
        let obj = self.eval(object)?;
        let idx = self.eval(index)?;

        match (&obj, &idx) {
            (Value::Array(arr), Value::Int(i)) => {
                let index = normalize_index(*i, arr.len());
                arr.get(index).cloned().ok_or_else(|| {
                    RuntimeError::new(format!(
                        "Index {i} out of bounds for array of length {}",
                        arr.len()
                    ))
                })
            }
            (Value::String(s), Value::Int(i)) => {
                let index = normalize_index(*i, s.len());
                s.chars().nth(index).map(Value::Char).ok_or_else(|| {
                    RuntimeError::new(format!(
                        "Index {i} out of bounds for string of length {}",
                        s.len()
                    ))
                })
            }
            (Value::Tuple(elems), Value::Int(i)) => {
                elems.get(*i as usize).cloned().ok_or_else(|| {
                    RuntimeError::new(format!(
                        "Index {i} out of bounds for tuple of length {}",
                        elems.len()
                    ))
                })
            }
            _ => Err(RuntimeError::new(format!(
                "Cannot index {} with {}",
                obj.type_name(),
                idx.type_name()
            ))),
        }
    }

    fn eval_field_access(&mut self, object: &Expr, field: &str) -> Result<Value, RuntimeError> {
        let obj = self.eval(object)?;
        match &obj {
            Value::Struct { fields, .. } => {
                fields.get(field).cloned().ok_or_else(|| {
                    RuntimeError::new(format!("No field '{field}' on struct"))
                })
            }
            Value::Tuple(elements) => {
                let idx: usize = field.parse().map_err(|_| {
                    RuntimeError::new(format!("Invalid tuple field: {field}"))
                })?;
                elements.get(idx).cloned().ok_or_else(|| {
                    RuntimeError::new(format!("Tuple index {idx} out of bounds"))
                })
            }
            _ => Err(RuntimeError::new(format!(
                "Cannot access field '{field}' on {}",
                obj.type_name()
            ))),
        }
    }

    fn eval_struct_init(
        &mut self,
        name: &str,
        fields: &[(String, Expr)],
    ) -> Result<Value, RuntimeError> {
        let mut field_values = HashMap::new();
        for (fname, fexpr) in fields {
            field_values.insert(fname.clone(), self.eval(fexpr)?);
        }
        Ok(Value::Struct {
            name: name.to_string(),
            fields: field_values,
        })
    }

    fn eval_match(&mut self, expr: &Expr, arms: &[MatchArm]) -> Result<Value, RuntimeError> {
        let val = self.eval(expr)?;
        for arm in arms {
            if self.match_pattern(&arm.pattern, &val)? {
                self.env.push_scope();
                self.bind_pattern(&arm.pattern, &val)?;
                let result = self.eval(&arm.body)?;
                self.env.pop_scope();
                return Ok(result);
            }
        }
        Err(RuntimeError::new("Non-exhaustive match"))
    }

    // ── Assignment targets ───────────────────────────────────────────

    fn assign_to(&mut self, target: &Expr, value: Value) -> Result<(), RuntimeError> {
        match target {
            Expr::Ident(name) => self.env.set(name, value),
            Expr::Index { object, index } => self.assign_to_index(object, index, value),
            Expr::FieldAccess { object, field } => self.assign_to_field(object, field, value),
            _ => Err(RuntimeError::new("Invalid assignment target")),
        }
    }

    fn assign_to_index(
        &mut self,
        object: &Expr,
        index: &Expr,
        value: Value,
    ) -> Result<(), RuntimeError> {
        let Expr::Ident(name) = object else {
            return Err(RuntimeError::new("Complex index assignment not supported"));
        };
        let idx = expect_int(self.eval(index)?, "Index")? as usize;
        let mut arr = self
            .env
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeError::new(format!("Undefined variable: '{name}'")))?;
        let Value::Array(ref mut elements) = arr else {
            return Err(RuntimeError::new("Cannot index non-array"));
        };
        if idx >= elements.len() {
            return Err(RuntimeError::new(format!("Index {idx} out of bounds")));
        }
        elements[idx] = value;
        self.env.set(name, arr)
    }

    fn assign_to_field(
        &mut self,
        object: &Expr,
        field: &str,
        value: Value,
    ) -> Result<(), RuntimeError> {
        let Expr::Ident(name) = object else {
            return Err(RuntimeError::new("Complex field assignment not supported"));
        };
        let mut obj = self
            .env
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeError::new(format!("Undefined variable: '{name}'")))?;
        let Value::Struct { ref mut fields, .. } = obj else {
            return Err(RuntimeError::new("Cannot set field on non-struct"));
        };
        fields.insert(field.to_string(), value);
        self.env.set(name, obj)
    }

    // ── Binary / unary operators ─────────────────────────────────────

    pub(crate) fn eval_binary_op(
        &self,
        left: &Value,
        op: &BinOp,
        right: &Value,
    ) -> Result<Value, RuntimeError> {
        // String concatenation
        if let (Value::String(a), BinOp::Add, Value::String(b)) = (left, op, right) {
            return Ok(Value::String(format!("{a}{b}")));
        }

        match (left, right) {
            (Value::Int(a), Value::Int(b)) => eval_int_op(*a, op, *b),
            (Value::Float(a), Value::Float(b)) => eval_float_op(*a, op, *b),
            (Value::Int(a), Value::Float(b)) => eval_float_op(*a as f64, op, *b),
            (Value::Float(a), Value::Int(b)) => eval_float_op(*a, op, *b as f64),
            (Value::Bool(a), Value::Bool(b)) => eval_bool_op(*a, op, *b),
            (Value::String(a), Value::String(b)) => eval_string_cmp(a, op, b),
            _ => Err(RuntimeError::new(format!(
                "Cannot apply {op:?} to {} and {}",
                left.type_name(),
                right.type_name()
            ))),
        }
    }

    fn eval_unary_op(&self, op: &UnaryOp, val: &Value) -> Result<Value, RuntimeError> {
        match (op, val) {
            (UnaryOp::Neg, Value::Int(n)) => Ok(Value::Int(-n)),
            (UnaryOp::Neg, Value::Float(n)) => Ok(Value::Float(-n)),
            (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
            (UnaryOp::Not, Value::Int(n)) => Ok(Value::Int(!n)),
            _ => Err(RuntimeError::new(format!(
                "Cannot apply {op:?} to {}",
                val.type_name()
            ))),
        }
    }

    /// Checks value equality using the `==` binary operator.
    pub(crate) fn values_equal(&self, left: &Value, right: &Value) -> bool {
        matches!(
            self.eval_binary_op(left, &BinOp::Eq, right),
            Ok(Value::Bool(true))
        )
    }

    /// Processes a format string; delegates to [`format::format_string`].
    pub(crate) fn format_string(
        &self,
        fmt: &str,
        args: &[Value],
    ) -> Result<String, RuntimeError> {
        format::format_string(fmt, args)
    }
}

// ── Free-standing helpers ────────────────────────────────────────────────

/// Converts a value into an iterable list of values.
fn value_to_iterator(val: Value) -> Result<Vec<Value>, RuntimeError> {
    match val {
        Value::Array(items) => Ok(items),
        Value::String(s) => Ok(s.chars().map(Value::Char).collect()),
        _ => Err(RuntimeError::new(format!(
            "Cannot iterate over {}",
            val.type_name()
        ))),
    }
}

/// Unwraps a `Value::Return` into its inner value.
fn unwrap_return(result: Value) -> Result<Value, RuntimeError> {
    match result {
        Value::Return(Some(v)) => Ok(*v),
        Value::Return(None) => Ok(Value::Unit),
        other => Ok(other),
    }
}

/// Extracts an `i64` from a `Value::Int`, or returns an error.
fn expect_int(val: Value, context: &str) -> Result<i64, RuntimeError> {
    match val {
        Value::Int(n) => Ok(n),
        other => Err(RuntimeError::new(format!(
            "{context} must be integer, got {}",
            other.type_name()
        ))),
    }
}

/// Normalizes a potentially-negative index into a valid `usize`.
fn normalize_index(i: i64, len: usize) -> usize {
    if i < 0 {
        (len as i64 + i) as usize
    } else {
        i as usize
    }
}

// ── Operator evaluation (pure functions) ─────────────────────────────────

fn eval_int_op(a: i64, op: &BinOp, b: i64) -> Result<Value, RuntimeError> {
    match op {
        BinOp::Add => Ok(Value::Int(a.wrapping_add(b))),
        BinOp::Sub => Ok(Value::Int(a.wrapping_sub(b))),
        BinOp::Mul => Ok(Value::Int(a.wrapping_mul(b))),
        BinOp::Div => {
            if b == 0 {
                Err(RuntimeError::new("Division by zero"))
            } else {
                Ok(Value::Int(a / b))
            }
        }
        BinOp::Mod => {
            if b == 0 {
                Err(RuntimeError::new("Modulo by zero"))
            } else {
                Ok(Value::Int(a % b))
            }
        }
        BinOp::Eq => Ok(Value::Bool(a == b)),
        BinOp::NotEq => Ok(Value::Bool(a != b)),
        BinOp::Lt => Ok(Value::Bool(a < b)),
        BinOp::LtEq => Ok(Value::Bool(a <= b)),
        BinOp::Gt => Ok(Value::Bool(a > b)),
        BinOp::GtEq => Ok(Value::Bool(a >= b)),
        BinOp::BitAnd => Ok(Value::Int(a & b)),
        BinOp::BitOr => Ok(Value::Int(a | b)),
        BinOp::BitXor => Ok(Value::Int(a ^ b)),
        BinOp::Shl => Ok(Value::Int(a << b)),
        BinOp::Shr => Ok(Value::Int(a >> b)),
        _ => Err(RuntimeError::new(format!(
            "Unsupported operation {op:?} on integers"
        ))),
    }
}

fn eval_float_op(a: f64, op: &BinOp, b: f64) -> Result<Value, RuntimeError> {
    match op {
        BinOp::Add => Ok(Value::Float(a + b)),
        BinOp::Sub => Ok(Value::Float(a - b)),
        BinOp::Mul => Ok(Value::Float(a * b)),
        BinOp::Div => Ok(Value::Float(a / b)),
        BinOp::Mod => Ok(Value::Float(a % b)),
        BinOp::Eq => Ok(Value::Bool(a == b)),
        BinOp::NotEq => Ok(Value::Bool(a != b)),
        BinOp::Lt => Ok(Value::Bool(a < b)),
        BinOp::LtEq => Ok(Value::Bool(a <= b)),
        BinOp::Gt => Ok(Value::Bool(a > b)),
        BinOp::GtEq => Ok(Value::Bool(a >= b)),
        _ => Err(RuntimeError::new(format!(
            "Unsupported operation {op:?} on floats"
        ))),
    }
}

fn eval_bool_op(a: bool, op: &BinOp, b: bool) -> Result<Value, RuntimeError> {
    match op {
        BinOp::And => Ok(Value::Bool(a && b)),
        BinOp::Or => Ok(Value::Bool(a || b)),
        BinOp::Eq => Ok(Value::Bool(a == b)),
        BinOp::NotEq => Ok(Value::Bool(a != b)),
        _ => Err(RuntimeError::new(format!(
            "Unsupported operation {op:?} on booleans"
        ))),
    }
}

fn eval_string_cmp(a: &str, op: &BinOp, b: &str) -> Result<Value, RuntimeError> {
    match op {
        BinOp::Eq => Ok(Value::Bool(a == b)),
        BinOp::NotEq => Ok(Value::Bool(a != b)),
        BinOp::Lt => Ok(Value::Bool(a < b)),
        BinOp::LtEq => Ok(Value::Bool(a <= b)),
        BinOp::Gt => Ok(Value::Bool(a > b)),
        BinOp::GtEq => Ok(Value::Bool(a >= b)),
        _ => Err(RuntimeError::new(format!(
            "Unsupported operation {op:?} on strings"
        ))),
    }
}
