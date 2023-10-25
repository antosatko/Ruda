use std::collections::HashMap;

use crate::{intermediate::dictionary, libloader};

use self::dict::prep_dict;

pub type Dictionaries = HashMap<String, dictionary::Dictionary>;
pub type Binaries = HashMap<String, libloader::Dictionary>;

pub struct Context(pub Dictionaries, pub Binaries);

impl Context {
    pub fn new(dictionaries: Dictionaries, binaries: Binaries) -> Self {
        Self(dictionaries, binaries)
    }
    pub fn destruct(&self) -> (&Dictionaries, &Binaries) {
        (&self.0, &self.1)
    }
    pub fn get_main(&self) -> &dictionary::Function {
        let fns = &self.0.get("main.rd").unwrap().functions;
        fns.iter().find(|f| f.identifier.as_ref().unwrap() == "main").unwrap()
    }
}

pub fn prep(context: &mut Context) -> Result<(), PrepError> {
    let keys = context.0.keys().cloned().collect::<Vec<_>>();
    for name in keys {
        prep_dict(&name, context)?;
    }
    Ok(())
}

pub enum PrepError {
    CouldNotLoadConstants(Vec<PrepError>),
    ConstNotFound(String),
}


mod dict {
    use crate::intermediate;

    use super::*;

    pub fn prep_dict(name: &str, context: &mut Context) -> Result<(), super::PrepError> {
        match prep_consts(name, context) {
            Ok(()) => (),
            Err(err) => return Err(err)
        }
        Ok(())
    }

    fn prep_consts(name: &str, context: &mut Context) -> Result<(), super::PrepError> {
        let mut const_values = HashMap::new();
        let mut errs = Vec::with_capacity(context.0.get_mut(name).unwrap().constants.len());
        loop {
            let mut done = true;
            let mut added = false;
            for constant in &context.0.get(name).unwrap().constants {
                let value = match prep_const(&constant, name, context) {
                    Ok(value) => {added = true; value}
                    Err(err) => {done = false; errs.push(err); continue;}
                };
                const_values.insert(constant.identifier.to_string(), value);
            }
            let keys = const_values.keys().cloned().collect::<Vec<_>>();
            for key in keys {
                let idx = context.0.get_mut(name).unwrap().constants.iter().position(|c| c.identifier == key).unwrap();
                context.0.get_mut(name).unwrap().constants[idx].real_value = const_values.remove(&key);
            }
            if done {
                break;
            }
            if !added {
                return Err(super::PrepError::CouldNotLoadConstants(errs));
            }
            errs.clear();
            const_values.clear();
        }

        Ok(())
    }

    fn prep_const(constant: &intermediate::dictionary::Constant, name: &str, context: &Context) -> Result<intermediate::dictionary::ConstValue, super::PrepError> {
        Ok(intermediate::dictionary::ConstValue::Int(0))
    }
}