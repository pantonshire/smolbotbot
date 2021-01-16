#[macro_use]
extern crate diesel;

pub mod connect;
pub mod schema;
pub mod model;
pub mod new;
pub mod create;

pub use crate::connect::connect_env;
pub use crate::model::*;
pub use crate::new::*;
pub use crate::create::Create;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
