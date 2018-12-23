use pulldown_cmark::{html, Parser};
use rocket_contrib::templates::tera::{Error, ErrorKind};
use serde_json::Value;
use std::collections::HashMap;

pub fn register(tera: &mut rocket_contrib::templates::tera::Tera) {
    tera.register_filter("markdown", markdown_filter);
    tera.register_function("current_version", Box::new(current_version));
    tera.register_function("min", Box::new(min));
    tera.register_function("max", Box::new(max));
    tera.register_function("range", Box::new(range));
}

fn fail<S: Into<String>, T>(f: S) -> Result<T, Error> {
    Err(Error::from_kind(ErrorKind::Msg(f.into())))
}

fn markdown_filter<S: std::hash::BuildHasher>(
    v: Value,
    _data: HashMap<String, Value, S>,
) -> Result<Value, Error> {
    if let Some(s) = v.as_str() {
        let parser = Parser::new(s);
        let mut html_buf = String::new();
        html::push_html(&mut html_buf, parser);
        Ok(Value::String(html_buf))
    } else {
        fail("Value is not a valid string")
    }
}

fn min<S: std::hash::BuildHasher>(map: HashMap<String, Value, S>) -> Result<Value, Error> {
    if let (Some(Value::Number(left)), Some(Value::Number(right))) =
        (map.get("left"), map.get("right"))
    {
        let left = left.as_i64().unwrap();
        let right = right.as_i64().unwrap();
        Ok(Value::Number(left.min(right).into()))
    } else {
        fail("Not implemented")
    }
}

fn max<S: std::hash::BuildHasher>(map: HashMap<String, Value, S>) -> Result<Value, Error> {
    if let (Some(Value::Number(left)), Some(Value::Number(right))) =
        (map.get("left"), map.get("right"))
    {
        let left = left.as_i64().unwrap();
        let right = right.as_i64().unwrap();
        Ok(Value::Number(left.max(right).into()))
    } else {
        fail("Not implemented")
    }
}

fn range<S: std::hash::BuildHasher>(map: HashMap<String, Value, S>) -> Result<Value, Error> {
    let result = if let (Some(Value::Number(from)), Some(Value::Number(to))) =
        (map.get("from"), map.get("to"))
    {
        from.as_i64().unwrap()..to.as_i64().unwrap()
    } else {
        return fail("Not implemented");
    };
    Ok(Value::Array(
        result.map(|v| Value::Number(v.into())).collect(),
    ))
}

pub fn current_version<S: std::hash::BuildHasher>(
    _: HashMap<String, Value, S>,
) -> Result<Value, Error> {
    Ok(Value::String(env!("CARGO_PKG_VERSION").to_owned()))
}
