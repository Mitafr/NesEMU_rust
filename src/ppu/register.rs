use crate::ppu::mem::PpuMem;
use crate::ppu::sprite::SpriteMem;
use crate::ppu::palette::PaletteVram;

use std::fmt;

pub struct PpuRegister {
    r_ctrl_zero: u8,
    r_ctrl_one: u8,
    r_status: u8,
    r_oam_addr: u8,
    r_oam_data: u8,
    r_scroll: u8,
    r_addr: u16,
    r_t_addr: u16,
    r_data: u8,
    r_oam_dma: u8,
    r_writing_lower_addr: bool,
}

pub trait Register {
    // CTRL $2000
    fn get_nametable_address(&self) -> usize;
    fn get_incr_value(&self) -> u8;
    fn get_sprite_table(&self) -> u8;
    fn get_background_table(&self) -> u8;
    fn get_sprite_size(&self) -> u8;
    fn get_slave_mode(&self) -> u8;
    fn get_irq_enable(&self) -> u8;
    fn get_ctrl_zero(&self) -> u8;

    // CTRL $2001
    fn get_background_visibility(&self) -> u8;
    fn get_sprite_visibility(&self) -> u8;
    fn get_sprite_clipping(&self) -> u8;
    fn get_background_clipping(&self) -> u8;
    fn get_ctrl_one(&self) -> u8;

    // $2002
    fn read_status(&mut self) -> u8;
    fn get_status(& self) -> u8;
    fn get_oam_addr(&self) -> u8;
    fn read_oam(&self) -> u8;
    fn get_scroll(&self) -> u8;
    fn get_addr(&self) -> u16;
    fn read_data(&mut self) -> u8;
    fn get_oam_dma(&self) -> u8;
    fn clear_vblank(&mut self) -> &mut Self;
    fn set_vblank(&mut self) -> &mut Self;
    fn clear_spritehit(&mut self) -> &mut Self;

    fn set_ctrl_zero(&mut self, v: u8) -> &mut Self;
    fn set_ctrl_one(&mut self, v: u8) -> &mut Self;
    fn set_status(&mut self, v: u8) -> &mut Self;
    fn set_oam_addr(&mut self, v: u8) -> &mut Self;
    fn write_oam_data(&mut self, v: u8, spr_mem: &mut SpriteMem) -> &mut Self;
    fn set_scroll(&mut self, v: u8) -> &mut Self;
    fn set_addr(&mut self, v: u16) -> &mut Self;
    fn write_data<P: PaletteVram>(&mut self, v: u8, mem: &mut PpuMem, palette: &mut P) -> &mut Self;
    fn set_oam_dma(&mut self, v: u8) -> &mut Self;

    fn incr_addr(&mut self) -> &mut Self;

    fn peek(&mut self, i: usize) -> u8;
    fn write<P: PaletteVram>(&mut self, i: usize, v: u8, mem: &mut PpuMem, palette: &mut P, spr_mem: &mut SpriteMem) -> u8;
}

impl PpuRegister {
    pub fn new() -> PpuRegister {
        PpuRegister {
            r_ctrl_zero: 0x00,
            r_ctrl_one: 0x00,
            r_status: 0x00,
            r_oam_addr: 0x00,
            r_oam_data: 0x00,
            r_scroll: 0x00,
            r_addr: 0x00,
            r_t_addr: 0x00,
            r_data: 0x00,
            r_oam_dma: 0x00,
            r_writing_lower_addr: false,
        }
    }
}

impl Register for PpuRegister {
    fn get_nametable_address(&self) -> usize {
        match (self.r_ctrl_zero >> 0) & 0x03 {
            0 => 0x2000,
            1 => 0x2400,
            2 => 0x2800,
            3 => 0x2C00,
            _ => 0
        }
    }
    fn get_incr_value(&self) -> u8 {
        if (self.r_ctrl_zero >> 2) & 0x01 == 0x0 {
            return 1;
        }
        return 32;
    }
    fn get_sprite_table(&self) -> u8 {
        (self.r_ctrl_zero >> 3) & 0x01
    }
    fn get_background_table(&self) -> u8 {
        (self.r_ctrl_zero >> 4) & 0x01
    }
    fn get_sprite_size(&self) -> u8 {
        (self.r_ctrl_zero >> 5) & 0x01
    }
    fn get_slave_mode(&self) -> u8 {
        (self.r_ctrl_zero >> 6) & 0x01
    }
    fn get_irq_enable(&self) -> u8 {
        (self.r_ctrl_zero >> 7) & 0x01
    }
    fn get_ctrl_zero(&self) -> u8 {
        self.r_ctrl_zero
    }


    fn get_background_visibility(&self) -> u8 {
        (self.r_ctrl_one >> 4) & 0x01
    }
    fn get_sprite_visibility(&self) -> u8 {
        (self.r_ctrl_one >> 3) & 0x01
    }
    fn get_sprite_clipping(&self) -> u8 {
        (self.r_ctrl_one >> 2) & 0x01
    }
    fn get_background_clipping(&self) -> u8 {
        (self.r_ctrl_one >> 1) & 0x01
    }
    fn get_ctrl_one(&self) -> u8 {
        self.r_ctrl_one
    }

    fn incr_addr(&mut self) -> &mut Self {
        let value = self.get_incr_value() as u16;
        self.r_addr += value;
        self
    }


    fn clear_vblank(&mut self) -> &mut Self {
        self.r_status &= 0b0111_1111;
        self
    }
    fn set_vblank(&mut self) -> &mut Self {
        self.r_status |= 0x80;
        self
    }
 
    fn clear_spritehit(&mut self) -> &mut Self {
        self.r_status &= 0b1011_1111;
        self
    }
    fn read_status(&mut self) -> u8 {
        let data = self.r_status;
        self.r_writing_lower_addr = false;
        self.clear_vblank().clear_spritehit();
        data
    }
    fn get_status(&self) -> u8 {
        self.r_status
    }
    fn get_oam_addr(&self) -> u8 {
        self.r_oam_addr
    }
    fn read_oam(&self) -> u8 {
        self.r_oam_data
    }
    fn get_scroll(&self) -> u8 {
        self.r_scroll
    }
    fn get_addr(&self) -> u16 {
        self.r_addr
    }
    fn read_data(&mut self) -> u8 {
        self.incr_addr();
        self.r_data
    }
    fn get_oam_dma(&self) -> u8 {
        self.r_oam_dma
    }

    fn set_ctrl_zero(&mut self, v: u8) -> &mut Self {
        self.r_ctrl_zero = v & 0xFC;
        self.r_t_addr = (self.r_t_addr & 0x73FF) | ((v & 3) as u16) << 10;
        self
    }
    fn set_ctrl_one(&mut self, v: u8) -> &mut Self {
        self.r_ctrl_one = v;
        self
    }
    fn set_status(&mut self, v: u8) -> &mut Self {
        self.r_status = v;
        self
    }
    fn set_oam_addr(&mut self, v: u8) -> &mut Self {
        self.r_oam_addr = v;
        self
    }
    fn write_oam_data(&mut self, v: u8, spr_mem: &mut SpriteMem) -> &mut Self {
        self.r_oam_data = v;
        spr_mem.write_data(self.get_oam_addr() as usize, v);
        self.r_oam_addr += 1;
        self
    }
    fn set_scroll(&mut self, v: u8) -> &mut Self {
        self.r_scroll = v;
        self
    }
    fn set_addr(&mut self, v: u16) -> &mut Self {
        if self.r_writing_lower_addr {
            self.r_t_addr = self.r_t_addr & 0xFF00 | v;
            self.r_addr = self.r_t_addr;
            self.r_writing_lower_addr = false;
        } else {
            self.r_t_addr = self.r_t_addr & 0x00FF | ((v & 0x3F) as u16) << 8;
            self.r_writing_lower_addr = true;
        }
        self
    }
    fn write_data<P: PaletteVram>(&mut self, v: u8, mem: &mut PpuMem, palette: &mut P) -> &mut Self {
        self.r_data = v;
        mem.write_data(self.r_addr as usize, v, palette);
        self.incr_addr();
        self
    }
    fn set_oam_dma(&mut self, v: u8) -> &mut Self {
        self.r_oam_dma = v;
        self
    }
    fn peek(&mut self, i: usize) -> u8 {
        match i {
            0x2002 => self.read_status(),
            0x2004 => self.read_oam(),
            0x2007 => self.read_data(),
            _ => {
                panic!("Invalid read PPU at {:x?}", i);
            }
        }
    }
    fn write<P: PaletteVram>(&mut self, i: usize, v: u8, mem: &mut PpuMem, palette: &mut P, spr_mem: &mut SpriteMem) -> u8 {
        match i {
            0x2000 => self.set_ctrl_zero(v),
            0x2001 => self.set_ctrl_one(v),
            0x2003 => self.set_oam_addr(v),
            0x2004 => self.write_oam_data(v, spr_mem),
            0x2005 => self.set_scroll(v),
            0x2006 => self.set_addr(v as u16),
            0x2007 => self.write_data(v, mem, palette),
            _ => {
                panic!("Invalid write in PPU at {:x?} : {:x?}", i, v);
            }
        };
        v
    }
}

impl fmt::Display for PpuRegister {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "|\tPPU REGISTER\t")?;
        writeln!(f, "+---+-----------+-----------------+")?;
        writeln!(f, "|CTRL_2000 => {:08b}", self.get_ctrl_zero())?;
        writeln!(f, "|bit|libelle       |value            |")?;
        writeln!(f, "+---+--------------+-----------------+")?;
        writeln!(f, "| 1 |NAME_TABLE   \t => {:x?}", self.get_nametable_address())?;
        writeln!(f, "| 2 |INCR_VALUE   \t => {:x?}", self.get_incr_value())?;
        writeln!(f, "| 3 |SPRITE_TABLE \t => {:x?}", self.get_sprite_table())?;
        writeln!(f, "| 4 |BCKGRND_TABLE\t => {:x?}", self.get_background_table())?;
        writeln!(f, "| 5 |SPRITE_SIZE  \t => {:x?}", self.get_sprite_size())?;
        writeln!(f, "| 6 |SLAVE_MODE   \t => {:x?}", self.get_slave_mode())?;
        writeln!(f, "| 7 |IRQ_ENABLE   \t => {:x?}", self.get_irq_enable())?;
        writeln!(f, "+---+-----------+-----------------+")?;
        writeln!(f, "|CTRL_2001 => {:08b}", self.get_ctrl_one())?;
        writeln!(f, "|bit|libelle                |value            |")?;
        writeln!(f, "+---+-----------------------+-----------------+")?;
        writeln!(f, "| 2 |CLIPPING_BCKGRND  \t => {:x?}", self.get_background_clipping())?;
        writeln!(f, "| 3 |CLIPPING_SPRITE   \t => {:x?}", self.get_sprite_clipping())?;
        writeln!(f, "| 4 |VISIBILITY_BCKGRND\t => {:x?}", self.get_background_visibility())?;
        writeln!(f, "| 5 |VISIBILITY_SPRITE \t => {:x?}", self.get_sprite_visibility())?;
        writeln!(f, "|libelle              |value            |")?;
        writeln!(f, "+---------------------+-----------------+")?;
        writeln!(f, "|STATUS_2002   \t => {:08b}", self.get_status())?;
        writeln!(f, "|VRAM_ADDR_2006\t => {:08b}", self.get_addr())?;
        writeln!(f, "+---+-----------+-----------------+")?;
        writeln!(f, "|OAM\t => {:08b}", self.r_oam_data)?;
        Ok(())
    }
}