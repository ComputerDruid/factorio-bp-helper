use serde_json::Value;
use string_stack::StringStack;

#[derive(Debug, PartialEq, Eq)]
pub enum WalkAction {
    Enter,
    Break,
}

mod string_stack;

pub fn walk_json(to_walk: &mut Value, cb: &mut impl FnMut(&[&str], &mut Value) -> WalkAction) {
    let mut storage = vec![];
    let path = StringStack::new(&mut storage);
    walk_json_inner(path, to_walk, cb);
}

fn walk_json_inner<'a, 'parent>(
    mut path: StringStack<'a, 'parent>,
    to_walk: &mut Value,
    cb: &mut impl FnMut(&[&str], &mut Value) -> WalkAction,
) {
    match (cb)(path.as_slice(), to_walk) {
        WalkAction::Enter => {}
        WalkAction::Break => return,
    }
    match to_walk {
        Value::Array(values) => {
            for value in values {
                let new_path = path.push("[]");
                walk_json_inner(new_path, value, cb)
            }
        }
        Value::Object(map) => {
            for (key, value) in map {
                let new_path = path.push(key);
                walk_json_inner(new_path, value, cb)
            }
        }
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            //nothing to walk
        }
    }
}
