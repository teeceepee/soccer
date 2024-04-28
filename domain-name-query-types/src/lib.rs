use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum QType {
    A,
    // Aaaa,
}

impl fmt::Display for QType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            QType::A => {
                "A"
            }
            // QType::Aaaa => {
            //     "AAAA"
            // }
        };
        write!(f, "{}", s)
    }
}

pub type Name = String;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct NameQuery {
    pub name: Name,
    pub q_type: QType,
}

impl NameQuery {
    pub fn a_record(name_str: &str) -> Self {
        Self {
            name: Name::from(name_str),
            q_type: QType::A,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
