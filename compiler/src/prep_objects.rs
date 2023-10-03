use std::collections::HashMap;

use crate::{intermediate::dictionary, libloader};

use self::dict::prep_dict;

pub type Dictionaries = HashMap<String, dictionary::Dictionary>;
pub type Binaries = HashMap<String, libloader::Dictionary>;

pub struct Context(Dictionaries, Binaries);

impl Context {
    pub fn new(dictionaries: Dictionaries, binaries: Binaries) -> Self {
        Self(dictionaries, binaries)
    }
    pub fn destruct(self) -> (Dictionaries, Binaries) {
        (self.0, self.1)
    }
}

pub fn prep(context: &mut Context) -> Result<(), PrepError> {
    let keys = context.0.keys().cloned().collect::<Vec<_>>();
    for name in keys {
        prep_dict(&name, context)?;
    }
    Ok(())
}

pub enum PrepError {}


mod dict {
    use crate::intermediate;

    use super::*;

    pub fn prep_dict(name: &str, context: &mut Context) -> Result<(), super::PrepError> {
        prep_consts(name, context);
        Ok(())
    }

    fn prep_consts(name: &str, context: &mut Context) -> Result<(), super::PrepError> {
        let mut const_values = HashMap::new();
        for constant in &context.0[name].constants {
            let value = prep_const(&constant, name, context)?;
            const_values.insert(constant.identifier.to_string(), value);
        }
        for constant in 0..context.0[name].constants.len() {
            context.0.get_mut(name).unwrap().constants[constant].real_value = Some(const_values.remove(&context.0[name].constants[constant].identifier).unwrap());
        }

        Ok(())
    }

    fn prep_const(constant: &intermediate::dictionary::Constant, name: &str, context: &Context) -> Result<intermediate::dictionary::ConstValue, super::PrepError> {
        Ok(intermediate::dictionary::ConstValue::Undefined)
    }
}