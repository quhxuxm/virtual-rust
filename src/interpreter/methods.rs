//! Method dispatch for built-in types.
//!
//! Each section handles methods for a specific value type:
//! String, Array/Vec, Int, Float, Char, Bool, Option.

use crate::ast::BinOp;
use crate::interpreter::error::RuntimeError;
use crate::interpreter::value::Value;
use crate::interpreter::Interpreter;

impl Interpreter {
    /// Dispatches a method call on a runtime value.
    pub(crate) fn call_method(
        &mut self,
        object: Value,
        method: &str,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        match &object {
            Value::String(s) => self.string_method(s.clone(), method, args),
            Value::Array(arr) => self.array_method(arr.clone(), method, args),
            Value::Int(n) => self.int_method(*n, method, &args),
            Value::Float(n) => self.float_method(*n, method, &args),
            Value::Bool(b) => self.bool_method(*b, method),
            Value::Char(c) => self.char_method(*c, method),
            Value::Option(opt) => self.option_method(opt.clone(), method, args),
            _ => {
                // Generic methods available on all types
                match method {
                    "to_string" => Ok(Value::String(format!("{object}"))),
                    "clone" => Ok(object.clone()),
                    _ => Err(RuntimeError::new(format!(
                        "Unknown method '{method}' on type '{}'",
                        object.type_name()
                    ))),
                }
            }
        }
    }
}

// ── String methods ───────────────────────────────────────────────────────

impl Interpreter {
    fn string_method(
        &mut self,
        s: String,
        method: &str,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        match method {
            "len" => Ok(Value::Int(s.len() as i64)),
            "is_empty" => Ok(Value::Bool(s.is_empty())),
            "contains" => {
                let substr = expect_string_arg(&args, "contains")?;
                Ok(Value::Bool(s.contains(substr.as_str())))
            }
            "starts_with" => {
                let prefix = expect_string_arg(&args, "starts_with")?;
                Ok(Value::Bool(s.starts_with(prefix.as_str())))
            }
            "ends_with" => {
                let suffix = expect_string_arg(&args, "ends_with")?;
                Ok(Value::Bool(s.ends_with(suffix.as_str())))
            }
            "trim" => Ok(Value::String(s.trim().to_string())),
            "trim_start" => Ok(Value::String(s.trim_start().to_string())),
            "trim_end" => Ok(Value::String(s.trim_end().to_string())),
            "to_uppercase" => Ok(Value::String(s.to_uppercase())),
            "to_lowercase" => Ok(Value::String(s.to_lowercase())),
            "replace" => {
                if args.len() < 2 {
                    return Err(RuntimeError::new("replace() expects two arguments"));
                }
                match (&args[0], &args[1]) {
                    (Value::String(from), Value::String(to)) => {
                        Ok(Value::String(s.replace(from.as_str(), to.as_str())))
                    }
                    _ => Err(RuntimeError::new(
                        "replace() expects two string arguments",
                    )),
                }
            }
            "split" => {
                let delim = expect_string_arg(&args, "split")?;
                let parts = s
                    .split(delim.as_str())
                    .map(|p| Value::String(p.to_string()))
                    .collect();
                Ok(Value::Array(parts))
            }
            "chars" => Ok(Value::Array(s.chars().map(Value::Char).collect())),
            "bytes" => Ok(Value::Array(
                s.bytes().map(|b| Value::Int(b as i64)).collect(),
            )),
            "to_string" => Ok(Value::String(s)),
            "parse" => {
                if let Ok(n) = s.parse::<i64>() {
                    Ok(Value::Option(Some(Box::new(Value::Int(n)))))
                } else if let Ok(n) = s.parse::<f64>() {
                    Ok(Value::Option(Some(Box::new(Value::Float(n)))))
                } else {
                    Ok(Value::Option(None))
                }
            }
            "push_str" => {
                let extra = expect_string_arg(&args, "push_str")?;
                Ok(Value::String(format!("{s}{extra}")))
            }
            "push" => {
                let c = expect_char_arg(&args, "push")?;
                let mut new_s = s;
                new_s.push(c);
                Ok(Value::String(new_s))
            }
            "repeat" => {
                let n = expect_int_arg(&args, "repeat")?;
                Ok(Value::String(s.repeat(n as usize)))
            }
            "lines" => {
                let lines = s.lines().map(|l| Value::String(l.to_string())).collect();
                Ok(Value::Array(lines))
            }
            "clone" => Ok(Value::String(s)),
            _ => Err(RuntimeError::new(format!(
                "Unknown method '{method}' on String"
            ))),
        }
    }
}

// ── Array / Vec methods ──────────────────────────────────────────────────

impl Interpreter {
    fn array_method(
        &mut self,
        arr: Vec<Value>,
        method: &str,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        match method {
            // ── Size & access ────────────────────────────────────────
            "len" | "count" => Ok(Value::Int(arr.len() as i64)),
            "is_empty" => Ok(Value::Bool(arr.is_empty())),
            "first" => Ok(Value::Option(arr.first().map(|v| Box::new(v.clone())))),
            "last" => Ok(Value::Option(arr.last().map(|v| Box::new(v.clone())))),

            // ── Mutation-style (returns new value) ───────────────────
            "push" => {
                let mut new_arr = arr;
                if let Some(val) = args.into_iter().next() {
                    new_arr.push(val);
                }
                Ok(Value::Array(new_arr))
            }
            "pop" => {
                let mut new_arr = arr;
                let popped = new_arr.pop();
                Ok(Value::Option(popped.map(Box::new)))
            }
            "reverse" => {
                let mut new_arr = arr;
                new_arr.reverse();
                Ok(Value::Array(new_arr))
            }

            // ── Search ───────────────────────────────────────────────
            "contains" => {
                if let Some(val) = args.first() {
                    let found = arr
                        .iter()
                        .any(|v| format!("{v}") == format!("{val}"));
                    Ok(Value::Bool(found))
                } else {
                    Ok(Value::Bool(false))
                }
            }

            // ── Iterators (pass-through) ─────────────────────────────
            "iter" | "into_iter" | "collect" => Ok(Value::Array(arr)),

            // ── Higher-order methods ─────────────────────────────────
            "map" => self.array_hof_map(arr, args),
            "filter" => self.array_hof_filter(arr, args),
            "fold" => self.array_hof_fold(arr, args),
            "for_each" => self.array_hof_for_each(arr, args),
            "any" => self.array_hof_any(arr, args),
            "all" => self.array_hof_all(arr, args),
            "find" => self.array_hof_find(arr, args),
            "position" => self.array_hof_position(arr, args),
            "flat_map" => self.array_hof_flat_map(arr, args),

            // ── Transform ────────────────────────────────────────────
            "enumerate" => {
                let enumerated = arr
                    .iter()
                    .enumerate()
                    .map(|(i, v)| Value::Tuple(vec![Value::Int(i as i64), v.clone()]))
                    .collect();
                Ok(Value::Array(enumerated))
            }
            "zip" => {
                let other = match args.first() {
                    Some(Value::Array(a)) => a,
                    _ => return Err(RuntimeError::new("zip() expects an array argument")),
                };
                let zipped = arr
                    .iter()
                    .zip(other.iter())
                    .map(|(a, b)| Value::Tuple(vec![a.clone(), b.clone()]))
                    .collect();
                Ok(Value::Array(zipped))
            }
            "skip" => {
                let n = expect_int_arg(&args, "skip")? as usize;
                Ok(Value::Array(arr.into_iter().skip(n).collect()))
            }
            "take" => {
                let n = expect_int_arg(&args, "take")? as usize;
                Ok(Value::Array(arr.into_iter().take(n).collect()))
            }

            // ── Aggregation ──────────────────────────────────────────
            "sum" => {
                let mut acc = Value::Int(0);
                for item in &arr {
                    acc = self.eval_binary_op(&acc, &BinOp::Add, item)?;
                }
                Ok(acc)
            }
            "product" => {
                let mut acc = Value::Int(1);
                for item in &arr {
                    acc = self.eval_binary_op(&acc, &BinOp::Mul, item)?;
                }
                Ok(acc)
            }
            "min" => self.array_min_max(&arr, true),
            "max" => self.array_min_max(&arr, false),
            "join" => {
                let sep = match args.first() {
                    Some(Value::String(s)) => s.as_str(),
                    _ => "",
                };
                let parts: Vec<String> = arr.iter().map(|v| format!("{v}")).collect();
                Ok(Value::String(parts.join(sep)))
            }

            _ => Err(RuntimeError::new(format!(
                "Unknown method '{method}' on array"
            ))),
        }
    }

    // ── Higher-order function helpers ────────────────────────────────

    fn array_hof_map(&mut self, arr: Vec<Value>, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let func = expect_callable(args, "map")?;
        let mut results = Vec::with_capacity(arr.len());
        for item in arr {
            results.push(self.call_function(func.clone(), vec![item])?);
        }
        Ok(Value::Array(results))
    }

    fn array_hof_filter(
        &mut self,
        arr: Vec<Value>,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let func = expect_callable(args, "filter")?;
        let mut results = Vec::new();
        for item in arr {
            let keep = self.call_function(func.clone(), vec![item.clone()])?;
            if keep.is_truthy() {
                results.push(item);
            }
        }
        Ok(Value::Array(results))
    }

    fn array_hof_fold(
        &mut self,
        arr: Vec<Value>,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        if args.len() < 2 {
            return Err(RuntimeError::new(
                "fold() expects an initial value and a closure",
            ));
        }
        let mut acc = args[0].clone();
        let func = args[1].clone();
        for item in arr {
            acc = self.call_function(func.clone(), vec![acc, item])?;
        }
        Ok(acc)
    }

    fn array_hof_for_each(
        &mut self,
        arr: Vec<Value>,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let func = expect_callable(args, "for_each")?;
        for item in arr {
            self.call_function(func.clone(), vec![item])?;
        }
        Ok(Value::Unit)
    }

    fn array_hof_any(&mut self, arr: Vec<Value>, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let func = expect_callable(args, "any")?;
        for item in arr {
            if self.call_function(func.clone(), vec![item])?.is_truthy() {
                return Ok(Value::Bool(true));
            }
        }
        Ok(Value::Bool(false))
    }

    fn array_hof_all(&mut self, arr: Vec<Value>, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let func = expect_callable(args, "all")?;
        for item in arr {
            if !self.call_function(func.clone(), vec![item])?.is_truthy() {
                return Ok(Value::Bool(false));
            }
        }
        Ok(Value::Bool(true))
    }

    fn array_hof_find(
        &mut self,
        arr: Vec<Value>,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let func = expect_callable(args, "find")?;
        for item in arr {
            if self
                .call_function(func.clone(), vec![item.clone()])?
                .is_truthy()
            {
                return Ok(Value::Option(Some(Box::new(item))));
            }
        }
        Ok(Value::Option(None))
    }

    fn array_hof_position(
        &mut self,
        arr: Vec<Value>,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let func = expect_callable(args, "position")?;
        for (i, item) in arr.iter().enumerate() {
            if self
                .call_function(func.clone(), vec![item.clone()])?
                .is_truthy()
            {
                return Ok(Value::Option(Some(Box::new(Value::Int(i as i64)))));
            }
        }
        Ok(Value::Option(None))
    }

    fn array_hof_flat_map(
        &mut self,
        arr: Vec<Value>,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        let func = expect_callable(args, "flat_map")?;
        let mut results = Vec::new();
        for item in arr {
            let result = self.call_function(func.clone(), vec![item])?;
            match result {
                Value::Array(inner) => results.extend(inner),
                other => results.push(other),
            }
        }
        Ok(Value::Array(results))
    }

    fn array_min_max(&self, arr: &[Value], is_min: bool) -> Result<Value, RuntimeError> {
        if arr.is_empty() {
            return Ok(Value::Option(None));
        }
        let cmp_op = if is_min { BinOp::Lt } else { BinOp::Gt };
        let mut best = arr[0].clone();
        for item in arr.iter().skip(1) {
            if let Value::Bool(true) = self.eval_binary_op(item, &cmp_op, &best)? {
                best = item.clone();
            }
        }
        Ok(Value::Option(Some(Box::new(best))))
    }
}

// ── Integer methods ──────────────────────────────────────────────────────

impl Interpreter {
    fn int_method(
        &self,
        n: i64,
        method: &str,
        args: &[Value],
    ) -> Result<Value, RuntimeError> {
        match method {
            "abs" => Ok(Value::Int(n.abs())),
            "pow" => {
                let exp = expect_int_arg(args, "pow")?;
                Ok(Value::Int(n.pow(exp as u32)))
            }
            "to_string" => Ok(Value::String(n.to_string())),
            "min" => {
                let other = expect_int_arg(args, "min")?;
                Ok(Value::Int(n.min(other)))
            }
            "max" => {
                let other = expect_int_arg(args, "max")?;
                Ok(Value::Int(n.max(other)))
            }
            "clamp" => {
                if args.len() < 2 {
                    return Err(RuntimeError::new("clamp() expects two arguments"));
                }
                match (&args[0], &args[1]) {
                    (Value::Int(min), Value::Int(max)) => Ok(Value::Int(n.clamp(*min, *max))),
                    _ => Err(RuntimeError::new("clamp() expects integer arguments")),
                }
            }
            _ => Err(RuntimeError::new(format!(
                "Unknown method '{method}' on i64"
            ))),
        }
    }
}

// ── Float methods ────────────────────────────────────────────────────────

impl Interpreter {
    fn float_method(
        &self,
        n: f64,
        method: &str,
        args: &[Value],
    ) -> Result<Value, RuntimeError> {
        match method {
            "abs" => Ok(Value::Float(n.abs())),
            "sqrt" => Ok(Value::Float(n.sqrt())),
            "floor" => Ok(Value::Float(n.floor())),
            "ceil" => Ok(Value::Float(n.ceil())),
            "round" => Ok(Value::Float(n.round())),
            "sin" => Ok(Value::Float(n.sin())),
            "cos" => Ok(Value::Float(n.cos())),
            "tan" => Ok(Value::Float(n.tan())),
            "log" => {
                if let Some(Value::Float(base)) = args.first() {
                    Ok(Value::Float(n.log(*base)))
                } else {
                    Ok(Value::Float(n.ln()))
                }
            }
            "ln" => Ok(Value::Float(n.ln())),
            "log2" => Ok(Value::Float(n.log2())),
            "log10" => Ok(Value::Float(n.log10())),
            "powi" => {
                let exp = expect_int_arg(args, "powi")?;
                Ok(Value::Float(n.powi(exp as i32)))
            }
            "powf" => {
                if let Some(Value::Float(exp)) = args.first() {
                    Ok(Value::Float(n.powf(*exp)))
                } else {
                    Err(RuntimeError::new("powf() expects a float argument"))
                }
            }
            "is_nan" => Ok(Value::Bool(n.is_nan())),
            "is_infinite" => Ok(Value::Bool(n.is_infinite())),
            "is_finite" => Ok(Value::Bool(n.is_finite())),
            "to_string" => Ok(Value::String(n.to_string())),
            _ => Err(RuntimeError::new(format!(
                "Unknown method '{method}' on f64"
            ))),
        }
    }
}

// ── Bool methods ─────────────────────────────────────────────────────────

impl Interpreter {
    fn bool_method(&self, b: bool, method: &str) -> Result<Value, RuntimeError> {
        match method {
            "to_string" => Ok(Value::String(b.to_string())),
            _ => Err(RuntimeError::new(format!(
                "Unknown method '{method}' on bool"
            ))),
        }
    }
}

// ── Char methods ─────────────────────────────────────────────────────────

impl Interpreter {
    fn char_method(&self, c: char, method: &str) -> Result<Value, RuntimeError> {
        match method {
            "is_alphabetic" => Ok(Value::Bool(c.is_alphabetic())),
            "is_numeric" => Ok(Value::Bool(c.is_numeric())),
            "is_alphanumeric" => Ok(Value::Bool(c.is_alphanumeric())),
            "is_whitespace" => Ok(Value::Bool(c.is_whitespace())),
            "is_uppercase" => Ok(Value::Bool(c.is_uppercase())),
            "is_lowercase" => Ok(Value::Bool(c.is_lowercase())),
            "to_uppercase" => Ok(Value::String(c.to_uppercase().to_string())),
            "to_lowercase" => Ok(Value::String(c.to_lowercase().to_string())),
            "to_string" => Ok(Value::String(c.to_string())),
            _ => Err(RuntimeError::new(format!(
                "Unknown method '{method}' on char"
            ))),
        }
    }
}

// ── Option methods ───────────────────────────────────────────────────────

impl Interpreter {
    fn option_method(
        &mut self,
        opt: Option<Box<Value>>,
        method: &str,
        args: Vec<Value>,
    ) -> Result<Value, RuntimeError> {
        match method {
            "unwrap" => match opt {
                Some(v) => Ok(*v),
                None => Err(RuntimeError::new("Called unwrap() on a None value")),
            },
            "unwrap_or" => match opt {
                Some(v) => Ok(*v),
                None => args
                    .into_iter()
                    .next()
                    .ok_or_else(|| RuntimeError::new("unwrap_or() expects a default value")),
            },
            "is_some" => Ok(Value::Bool(opt.is_some())),
            "is_none" => Ok(Value::Bool(opt.is_none())),
            "map" => {
                let func = expect_callable(args, "map")?;
                match opt {
                    Some(v) => {
                        let result = self.call_function(func, vec![*v])?;
                        Ok(Value::Option(Some(Box::new(result))))
                    }
                    None => Ok(Value::Option(None)),
                }
            }
            _ => Err(RuntimeError::new(format!(
                "Unknown method '{method}' on Option"
            ))),
        }
    }
}

// ── Argument extraction helpers ──────────────────────────────────────────

fn expect_string_arg(args: &[Value], method_name: &str) -> Result<String, RuntimeError> {
    match args.first() {
        Some(Value::String(s)) => Ok(s.clone()),
        _ => Err(RuntimeError::new(format!(
            "{method_name}() expects a string argument"
        ))),
    }
}

fn expect_int_arg(args: &[Value], method_name: &str) -> Result<i64, RuntimeError> {
    match args.first() {
        Some(Value::Int(n)) => Ok(*n),
        _ => Err(RuntimeError::new(format!(
            "{method_name}() expects an integer argument"
        ))),
    }
}

fn expect_char_arg(args: &[Value], method_name: &str) -> Result<char, RuntimeError> {
    match args.first() {
        Some(Value::Char(c)) => Ok(*c),
        _ => Err(RuntimeError::new(format!(
            "{method_name}() expects a char argument"
        ))),
    }
}

fn expect_callable(args: Vec<Value>, method_name: &str) -> Result<Value, RuntimeError> {
    args.into_iter().next().ok_or_else(|| {
        RuntimeError::new(format!("{method_name}() expects a closure argument"))
    })
}
