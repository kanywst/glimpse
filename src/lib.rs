pub mod app;
pub mod event;
pub mod github;
pub mod handlers;
pub mod semantics;
pub mod tui;
pub mod ui;
pub mod utils;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
