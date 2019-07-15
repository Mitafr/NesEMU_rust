pub trait Memory {
    fn get_size(&self) -> usize;

    fn peek(&mut self, i: usize) -> u8;
    fn write(&mut self, i: usize, value: u8) -> u8;

    fn load_program(&mut self, data: &mut Vec<u8>) -> &mut Self;

    fn get_mem(&self) -> &[u8];
}
