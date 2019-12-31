
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
            Event::KeyDown { keycode: Some(Keycode::Z), ..} => self.key = KeyStatus::B,
            Event::KeyDown { keycode: Some(Keycode::Space), ..} => self.key = KeyStatus::Select,
            Event::KeyDown { keycode: Some(Keycode::Return), ..} => self.key = KeyStatus::Start,
            Event::KeyDown { keycode: Some(Keycode::Up), ..} => self.key = KeyStatus::Up,
            Event::KeyDown { keycode: Some(Keycode::Down), ..} => self.key = KeyStatus::Down,
            Event::KeyDown { keycode: Some(Keycode::Left), ..} => self.key = KeyStatus::Left,
            Event::KeyDown { keycode: Some(Keycode::Right), ..} => self.key = KeyStatus::Right,
            Event::KeyUp { keycode: Some(Keycode::A), ..} => self.key = KeyStatus::Idle,
            Event::KeyUp { keycode: Some(Keycode::Z), ..} => self.key = KeyStatus::Idle,
            Event::KeyUp { keycode: Some(Keycode::Space), ..} => self.key = KeyStatus::Idle,
            Event::KeyUp { keycode: Some(Keycode::D), ..} => self.key = KeyStatus::Idle,
            Event::KeyUp { keycode: Some(Keycode::Up), ..} => self.key = KeyStatus::Idle,
            Event::KeyUp { keycode: Some(Keycode::Down), ..} => self.key = KeyStatus::Idle,
            Event::KeyUp { keycode: Some(Keycode::Left), ..} => self.key = KeyStatus::Idle,
            Event::KeyUp { keycode: Some(Keycode::Right), ..} => self.key = KeyStatus::Idle,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Controller;
    use super::KeyStatus;

    #[test]
    fn read_without_strobe_should_shift_right() {
        let mut controller = Controller::new();
        controller.addr = KeyStatus::B as u8;
        controller.strobe = false;
        controller.read();
        assert_eq!(controller.addr, KeyStatus::B as u8 >> 1);
    }
    #[test]
    fn read_with_strobe_should_return_keycode() {
        let mut controller = Controller::new();
        controller.key = KeyStatus::A;
        controller.strobe = true;
        controller.read();
        assert_eq!(controller.addr, 1);
        controller.key = KeyStatus::Up;
        controller.strobe = true;
        controller.read();
        assert_eq!(controller.addr, KeyStatus::Up as u8);
    }
    #[test]
    fn write_should_strobe() {
        let mut controller = Controller::new();
        controller.write(0x1);
        assert!(controller.strobe);
    }
    #[test]
    fn write_shouldnt_strobe() {
        let mut controller = Controller::new();
        controller.write(0xc0);
        assert!(!controller.strobe);
    }
}