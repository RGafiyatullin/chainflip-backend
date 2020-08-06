pub mod common;
pub mod quoter;
pub mod side_chain;
pub mod transactions;
pub mod utils;
pub mod vault;

pub mod logging;

#[macro_use]
extern crate log;

/// Temporary funciton to demostrate how to use
/// unit/integration tests
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// Unit test sample
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
}
