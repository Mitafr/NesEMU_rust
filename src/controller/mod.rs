
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub enum KeyStatus {
    A = 0b0000_0001,
    B = 0b0000_0010,
    Select = 0b0000_0100,
    Start = 0b0000_1000,
    Up = 0b0001_0000,
    Down = 0b0010_0000,
    Left = 0b0100_0000,
    Right = 0b1000_0000,
}

struct Register {
    bit: u8,
    key: u8,
}

impl Register {
    pub fn new() -> Register {
        Register {
            bit: 0x00,
            key: 0x00,
        }
    }
    pub fn write(&mut self, v: u8) {
        self.bit = v;
    }
}

pub struct Controller {
    register: Register,
    addr: u16,
    strobe: bool
}

impl Controller {
    pub fn new() -> Controller {
        Controller {
            register: Register::new(),
            addr: 0x00,
            strobe: false
        }
    }
    pub fn write(&mut self, v: u8) -> u8 {
        self.register.write(v);
        self.strobe = self.register.bit&1 == 1;
        if self.strobe {
            self.addr = 0x00;
        }
        self.addr = v as u16;
        v
    }
    pub fn read(&mut self) -> u8 {
        let value = self.register.key.wrapping_shr(self.addr as u32) & 1;
        if !self.strobe {
            self.addr = self.addr>>1;
        }
        println!("Reading Controller value {:x?} register Key {:x?} Addr {:08b}", value, self.register.key, self.addr);
        value | 0x40
    }
    pub fn poll_events(&mut self, event: &Event) {
        match event {
            Event::KeyDown { keycode: Some(Keycode::A), ..} => self.register.key |= KeyStatus::A as u8,
            Event::KeyDown { keycode: Some(Keycode::B), ..} => self.register.key |= KeyStatus::B as u8,
            Event::KeyDown { keycode: Some(Keycode::C), ..} => self.register.key |= KeyStatus::Select as u8,
            Event::KeyDown { keycode: Some(Keycode::Return), ..} => self.register.key |= KeyStatus::Start as u8,
            Event::KeyDown { keycode: Some(Keycode::Up), ..} => self.register.key |= KeyStatus::Up as u8,
            Event::KeyDown { keycode: Some(Keycode::Down), ..} => self.register.key |= KeyStatus::Down as u8,
            Event::KeyDown { keycode: Some(Keycode::Left), ..} => self.register.key |= KeyStatus::Left as u8,
            Event::KeyDown { keycode: Some(Keycode::Right), ..} => self.register.key |= KeyStatus::Right as u8,
            Event::KeyUp { keycode: Some(Keycode::A), ..} => self.register.key &= KeyStatus::A as u8 ^ 1,
            Event::KeyUp { keycode: Some(Keycode::B), ..} => self.register.key &= KeyStatus::B as u8 ^ 1,
            Event::KeyUp { keycode: Some(Keycode::C), ..} => self.register.key &= KeyStatus::Select as u8 ^ 1,
            Event::KeyUp { keycode: Some(Keycode::D), ..} => self.register.key &= KeyStatus::Start as u8 ^ 1,
            Event::KeyUp { keycode: Some(Keycode::Up), ..} => self.register.key &= KeyStatus::Up as u8 ^ 1,
            Event::KeyUp { keycode: Some(Keycode::Down), ..} => self.register.key &= KeyStatus::Down as u8 ^ 1,
            Event::KeyUp { keycode: Some(Keycode::Left), ..} => self.register.key &= KeyStatus::Left as u8 ^ 1,
            Event::KeyUp { keycode: Some(Keycode::Right), ..} => self.register.key &= KeyStatus::Right as u8 ^ 1,
            _ => {}
        }
    }
}