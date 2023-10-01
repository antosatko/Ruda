use std::collections::HashMap;

use crate::{intermediate::dictionary, libloader};

type Dictionaries = HashMap<String, dictionary::Dictionary>;
type Binaries = HashMap<String, libloader::Dictionary>;

pub fn prep((dictionaries, binaries): &mut (Dictionaries, Binaries)) -> Result<(), PrepError> {
    Ok(())
}

pub enum PrepError {}
