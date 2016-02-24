#![allow(dead_code)]

extern crate mio;

pub mod buffer;
pub mod dispatcher;
pub mod eloop;
pub mod provider;
pub mod route;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
