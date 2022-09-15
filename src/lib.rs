pub mod connection;
pub mod server;

pub const BUFFER_SIZE: usize = 4096;
const BASE_MESSAGE_LEN: usize = "message +l #".len();

pub(crate) fn number_len(number: usize) -> usize {
    number.to_string().len()
}
