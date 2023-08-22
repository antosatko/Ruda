use crate::intermediate;

use intermediate::dictionary::Dictionary;

pub mod TypesCheck {
    use crate::intermediate::dictionary::Dictionary;

    pub fn index_types(dictionary: &mut Dictionary) {
        println!("Types check started.");
        for (key, value) in &dictionary.identifiers {
            //println!("{}: {:?}", key, value);
        }
    }
}