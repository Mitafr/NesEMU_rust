pub trait Memory {
    fn get_size(&self) -> usize;

    fn peek(&self, i: u16) -> u8;
    fn write(&mut self, i: u16, value: u8) -> u8;

    fn get_mem(&self) -> &[u8];
}
