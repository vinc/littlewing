pub mod cli;
pub mod uci;
pub mod xboard;

#[repr(u8)]
#[derive(Clone, Eq, PartialEq)]
pub enum Protocol {
    CLI,
    UCI,
    XBoard
}
