use minifb::Key;
use crate::{chord::Chord, drum::Drum, Setup, midi::Channel};


#[derive(Debug)]
pub enum Inst {
    Chord,
    Drum,
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right
}
impl Direction {
    pub fn from_key(key: &Key) -> Option<Direction> {
        match key {
            Key::Up => Some(Direction::Up),
            Key::Down => Some(Direction::Down),
            Key::Left => Some(Direction::Left),
            Key::Right => Some(Direction::Right),
            _ => None
        }
    }
}

#[derive(Debug)]
pub struct KeyboardFocus {
    pub editing: bool,
    pub index: u8,
    pub inst: Inst,
    pub preview_all: bool,
}
impl KeyboardFocus {
    pub fn new() -> Self {
        Self {
            editing: false,
            index: 0,
            inst: Inst::Chord,
            preview_all: false
        }
    }
    pub fn get_focused_chord(&self, chords: &Vec<Chord>) -> Option<usize> {
        for (i, c) in chords.iter().enumerate() {
            if c.channel == Channel::from(self.index) {
                return Some(i);
            }
        }
        None
    }
    pub fn get_focused_drum(&self, drums: &Vec<Drum>) -> Option<usize> {
        if (self.index as usize) < drums.len() {
            Some(self.index as usize)
        } else {
            None
        }
    }
    pub fn adjustment(&self, setup: &mut Setup, direction: Direction, shift: bool, ctrl: bool) {
        if !self.editing {
            return;
        }
        let amount = if shift && ctrl {
            1
        } else if shift && !ctrl {
            5
        } else if ctrl && !shift {
            20
        } else {
            10
        };
        match self.inst {
            Inst::Chord => {
                let f = self.get_focused_chord(&setup.chords);
                if f.is_some() {
                    match direction {
                        Direction::Down => {
                            if setup.chords[f.unwrap()].width as i32 - amount as i32 > 0 {
                                setup.chords[f.unwrap()].width -= amount/2
                            }
                        },
                        Direction::Up => {
                            setup.chords[f.unwrap()].width += amount/2
                        },
                        Direction::Left => {
                            if setup.chords[f.unwrap()].xpos as i32 - amount as i32 > 0 {
                                setup.chords[f.unwrap()].xpos -= amount
                            }
                        },
                        Direction::Right => setup.chords[f.unwrap()].xpos += amount,
                    }
                }
                
            },
            Inst::Drum => {
                let f = self.get_focused_drum(&setup.kit.drums);
                if f.is_some() {
                    match direction {
                        Direction::Down => {
                            setup.kit.drums[f.unwrap()].ypos += amount
                        },
                        Direction::Up => {
                            if setup.kit.drums[f.unwrap()].ypos as i32 - amount as i32  > 0 {
                                setup.kit.drums[f.unwrap()].ypos -= amount
                            }
                        },
                        Direction::Left => {
                            if setup.kit.drums[f.unwrap()].xpos as i32 - amount as i32 > 0 {
                                setup.kit.drums[f.unwrap()].xpos -= amount
                            }
                        },
                        Direction::Right => setup.kit.drums[f.unwrap()].xpos += amount,
                    }
                }
            },
        }
    }

    pub fn mouse_adjustment(&self, setup: &mut Setup, m_x: f32, m_y: f32) {
        if !self.editing {
            return;
        }
        let x = ((m_x / 10.0).round() * 10.0) as u32;
        let y = ((m_y / 10.0).round() * 10.0) as u32;
        match self.inst {
            Inst::Chord => {
                let f = self.get_focused_chord(&setup.chords);
                if f.is_some() {
                    setup.chords[f.unwrap()].xpos = x;
                }
                
            },
            Inst::Drum => {
                let f = self.get_focused_drum(&setup.kit.drums);
                if f.is_some() {
                    setup.kit.drums[f.unwrap()].xpos = x;
                    setup.kit.drums[f.unwrap()].ypos = y;
                }
            },
        }
    }
    
}