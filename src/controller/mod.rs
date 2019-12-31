
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
#[derive(Debug, Copy, Clone)]
pub enum KeyStatus {
    Idle = 0,
    A = 1 << 0,
    B = 1 << 1,
    Select = 1 << 2,
    Start = 1 << 3,
    Up = 1 << 4,
    Down = 1 << 5,
    Left = 1 << 6,
    Right = 1 << 7,
}

pub struct Controller {
    addr: u8,
    strobe: bool,
    key: KeyStatus,
}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            addr: 0x00,
            strobe: false,
            key: KeyStatus::Idle,
        }
    }
    pub fn write(&mut self, v: u8) -> u8 {
        self.strobe = v&1 == 1;
        if self.strobe {
            self.addr = self.key as u8;
        }
        v
    }
    pub fn read(&mut self) -> u8 {
        if self.strobe {
            self.addr = self.key as u8;
            self.addr & 1
        } else {
            let old = self.addr;
            self.addr = self.addr>>1;
            old&1
        }
    }
    pub fn poll_events(&mut self, event: &Event) {
        match event {
            Event::KeyDown { keycode: Some(Keycode::A), ..} => self.key = KeyStatus::A,
            Event::KeyDown { keycode: Some(Keycode::B), ..} => self.key = KeyStatus::B,
            Event::KeyDown { keycode: Some(Keycode::C), ..} => self.key = KeyStatus::Select,
            Event::KeyDown { keycode: Some(Keycode::Return), ..} => self.key = KeyStatus::Start,
            Event::KeyDown { keycode: Some(Keycode::Up), ..} => self.key = KeyStatus::Up,
            Event::KeyDown { keycode: Some(Keycode::Down), ..} => self.key = KeyStatus::Down,
            Event::KeyDown { keycode: Some(Keycode::Left), ..} => self.key = KeyStatus::Left,
            Event::KeyDown { keycode: Some(Keycode::Right), ..} => self.key = KeyStatus::Right,
            _ => {self.key = KeyStatus::Idle}
        }
    }
}