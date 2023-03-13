use minifb::{Window, WindowOptions};
use fontdue::Font;
use std::cmp::min;

use crate::{
    graphics::*, 
    chord::{Chord, ChordStyle}, 
    midi::Note, 
    gui::{Direction, KeyboardFocus, Inst}, 
    Setup, 
    drum::{Drum, DrumGfx, DrumStyle}, 
    text::{TextDirection, Text, TextStyle}
};

pub struct Win {
    pub width: u32,
    pub height: u32,
    buffer: Vec<u32>,
    pub window: Window,
    pub tick: usize,
    main_font: Font,
    pub running: bool,
    pub status_message: StatusMessage
}

impl Win {
    pub fn init(width: u32, height: u32) -> Self {
        let font = include_bytes!("../misaki_gothic_2nd.ttf") as &[u8];
        let misaki = Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();

        let mut w = Win {
            width,
            height,
            buffer: vec![0; width as usize * height as usize],
            window: Window::new(
                "grooveanim thing idk (meows softly)",
                width as usize,
                height as usize,
                WindowOptions::default(),
            ).unwrap_or_else(|e| {
                panic!("uhh... {}", e);
            }),
            tick: 0,
            main_font: misaki,
            running: true,
            status_message: StatusMessage::None
        };
        w.window.limit_update_rate(Some(std::time::Duration::from_micros(8000)));
        w
    }

    pub fn update(&mut self) {
        self.window.update_with_buffer(&self.buffer, self.width as usize, self.height as usize).unwrap();
        self.tick += 1;
    }

    pub fn clear(&mut self) {
        self.buffer = vec![0; self.width as usize * self.height as usize];
    }

    fn set_pixel(&mut self, x: u32, y: u32, color: &RGB) {
        let screen_pos = ((y as usize) * self.width as usize) + x as usize;
        if x >= self.width || y >= self.height {
            return;
        }
        self.buffer[screen_pos] = color.irl();
    }

    pub fn draw_rect(&mut self, x: u32, y: u32, xsize: u32, ysize: u32, color: &RGB) {
        for y in y..(y + ysize) {
            for x in x..(x + xsize) {
                self.set_pixel(x, y, color);
            }
        }
    }

    pub fn draw_outline_rect(&mut self, xpos: u32, ypos: u32, xsize: u32, ysize: u32, thickness: u32, dotted: bool, color: &RGB) {
        for x in xpos..(xpos+xsize) {
            if dotted && (x % 2) == 0 {
                continue;
            }
            for y in ypos..(ypos+thickness) {
                self.set_pixel(x, y, color);
            }
            for y in ypos+ysize-thickness..(ypos+ysize) {
                self.set_pixel(x, y, color);
            }
        }
        for y in ypos..(ypos+ysize) {
            if dotted && (y % 2) == 0 {
                continue;
            }
            for x in xpos..(xpos+thickness) {
                self.set_pixel(x, y, color);
            }
            for x in xpos+xsize-thickness..(xpos+xsize) {
                self.set_pixel(x, y, color);
            }
        }
    }

    fn draw_hatched_rect2(&mut self, xpos: u32, ypos: u32, xsize: u32, ysize: u32, thickness: u32, color: &RGB) {
        self.draw_outline_rect(xpos, ypos, xsize, ysize, thickness, false, color);
        for y in ypos..(ypos + ysize) {
            if (y % 4) != 0 {
                continue;
            }
            for x in xpos..(xpos + xsize) {
                if (x % 4) == 0 {
                    self.set_pixel(x, y, color);
                }
            }
        }
    }

    pub fn draw_hatched_rect(&mut self, xpos: u32, ypos: u32, xsize: u32, ysize: u32, thickness: u32, color: &RGB) {
        self.draw_outline_rect(xpos, ypos, xsize, ysize, thickness, false, color);
        for y in ypos..(ypos + ysize) {
            for x in xpos..(xpos + xsize) {
                if (x % 6) == (y % 6) {
                    self.set_pixel(x, y, color);
                }
            }
        }
    }

    pub fn draw_bitmap(&mut self, xpos: u32, ypos: u32, dimmed: Option<f32>, bitmap: &Bitmap) {
        for y in ypos..ypos + bitmap.ysize {
            for x in xpos..xpos + bitmap.xsize {
                let screen_pos = ((y as usize) * self.width as usize) + x as usize;
                if screen_pos >= self.buffer.len() {
                    return;
                }
                let data = bitmap.color(x - xpos, y - ypos, dimmed);
                if data != 0 {
                    self.buffer[screen_pos] = data;
                }
            }
        }
    }

    pub fn draw_chord(&mut self, chord: &mut Chord, global_transpose: i32) {
        let mut to_remove: Vec<Note> = vec![];

        for note in &mut chord.notes {
            // height of a note is 8px
            let height: u32 = 8;
            // gap of 4 between each
            let effective_pitch = note.pitch as i32 + chord.transpose + global_transpose;
            if effective_pitch < 0 {
                continue; // too lowwwww
            }
            if (effective_pitch as u32 * (height + 4)) > self.height {
                continue; // out of range
            }
            let mut ypos = self.height - (effective_pitch as u32 * (height + 4));

            // vibrato
            let saw = self.tick % 10;
            let wiggle = chord.modulation as f32 / 32000.0;
            if saw < 5 {
                ypos += (wiggle * saw as f32) as u32;
            } else {
                ypos -= (wiggle * saw as f32) as u32;
            }

            // velocity
            let (mut color, mut shadow) = if chord.velocity_sense {
                let _color = chord.color.dimmed(note.velocity as f32 / 127.0);
                let _shadow = chord.shadow.dimmed(note.velocity as f32 / 127.0);
                (_color, _shadow)
            }  else {
                (chord.color, chord.shadow)
            };

            let mut xpos = chord.xpos;
            let mut width = chord.width;

            match chord.style {
                ChordStyle::Plain => {},
                ChordStyle::Decay{time} => {
                    if note.age >= time {
                        to_remove.push(note.clone());
                        continue;
                    } else {
                    let f = (time - note.age) as f32 / time as f32;
                    color = color.dimmed(f);
                    shadow = shadow.dimmed(f);
                    }
                },
                ChordStyle::DecayRelease{time} =>{
                    if note.age >= time {
                        to_remove.push(note.clone());
                        continue;
                    } else {
                        let f = (time - note.age) as f32 / time as f32;
                        color = color.dimmed(f);
                        shadow = shadow.dimmed(f);
                    }
                },
                ChordStyle::Grow{speed} => {
                    if note.age * speed <= chord.width {
                        width = note.age * speed;
                    }
                },
                ChordStyle::Marquee{speed} => {
                    if note.age * speed > self.width + chord.width - xpos{
                        to_remove.push(note.clone());
                        continue;
                    }
                    if note.age * speed < (self.width - xpos) {
                        xpos = self.width - note.age * speed;
                    } else {

                        width = chord.width + self.width - xpos + 1 - note.age * speed;
                    }
                },
            }

            if chord.style != ChordStyle::Plain {
                note.age += 1;
                
            }

            self.draw_rect(xpos, ypos, width, height, &color);
            self.draw_rect(xpos, ypos + height, width, 2, &shadow);
        }

        // clear decayed notes
        for note in to_remove {
            chord.notes.retain(|x| x.age != note.age);
        }
    }

    pub fn draw_ghost_chord(&mut self, chord: &Chord) {
        let c_white = RGB::fff();
        let xpos = min(chord.xpos, 550);
        if chord.xpos >= self.width - 1 {
            self.draw_arrow(xpos + 24, 18, Direction::Right);
        }
        let ypos = 24*chord.channel as u32;
        self.draw_plaintext(xpos, ypos + 16 , 16.0, format!("ch{}", 1+chord.channel as u8), TextDirection::Horizontal, &c_white);
        self.draw_plaintext(xpos, ypos + 30 , 16.0, format!("x:{} w:{}", chord.xpos, chord.width), TextDirection::Horizontal, &c_white);
        for i in 0..10 {
            self.draw_outline_rect(chord.xpos, 10 + (12*4*i) + ypos, chord.width, 8, 2, true, &chord.color);
        }
    }

    pub fn draw_drum(&mut self, drum: &mut Drum) {
        if drum.state.triggered {
            if !drum.state.tick_or_reset(drum.decay_time) {
                return;
            }
            match &drum.gfx {
                DrumGfx::Plain { xsize, ysize, color } => {
                    let color = if drum.style == DrumStyle::Decay {
                        color.dimmed((drum.decay_time - drum.state.age) as f32 / drum.decay_time as f32)
                    } else {
                        *color
                    };
                    self.draw_rect(drum.xpos, drum.ypos, *xsize, *ysize, &color)
                },
                DrumGfx::Bitmap { bitmap  } => {
                    if drum.style == DrumStyle::Decay {
                        let f = (drum.decay_time - drum.state.age) as f32 / drum.decay_time as f32;
                        self.draw_bitmap(drum.xpos, drum.ypos, Some(f), &bitmap);
                    } else {
                        self.draw_bitmap(drum.xpos, drum.ypos, None, &bitmap);
                    };
                    
                },
            }
            
        }
    }

    pub fn draw_ghost_drum(&mut self, drum: &Drum) {
        let c_white = RGB::fff();
        let (xsize, ysize) = match &drum.gfx {
            DrumGfx::Plain { xsize, ysize, color: _ } => (xsize, ysize),
            DrumGfx::Bitmap { bitmap } => (&bitmap.xsize, &bitmap.ysize),
        };
        let title_on_bottom: bool;
        let text_y = min(if drum.ypos < 70 {
            title_on_bottom = true;
            drum.ypos + ysize
        } else {
            title_on_bottom = false;
            drum.ypos - 41
        }, self.height - 60);
        let text_x = min(drum.xpos, self.width - 90);
        self.draw_plaintext(text_x, text_y, 16.0, format!("x: {: >3} y: {: >3}", drum.xpos, drum.ypos), TextDirection::Horizontal, &c_white);
        self.draw_plaintext(text_x, text_y + 20, 16.0, format!("w: {: >3} h: {: >3}", xsize, ysize), TextDirection::Horizontal, &c_white.dimmed(0.5));

        match &drum.gfx {
            DrumGfx::Bitmap { bitmap } => {
                let title_text_y = if title_on_bottom  {
                    text_y + 40
                } else {
                    text_y - 20
                };
                let title = bitmap.path.split('.').collect::<Vec<&str>>()[0].to_string();
        
                self.draw_plaintext(text_x, title_text_y, 16.0, title, TextDirection::Horizontal, &c_white);
            },
            _ => {},
        }
        
        // if offscreen...
        if drum.ypos >= self.height {
            self.draw_arrow(
                if drum.xpos >= self.width {self.width - 16} else {text_x},
                 text_y + 40, 
                 Direction::Down
            );
        } else if drum.xpos >= self.width - 1 {
            self.draw_arrow(self.width - 16, text_y + 40, Direction::Right);
        }

        let (xsize, ysize, color) = match &drum.gfx {
            DrumGfx::Plain { xsize, ysize, color } => (xsize, ysize, color.clone()),
            DrumGfx::Bitmap { bitmap } => {
                let color = bitmap.first_nonzero_color().clone();
                (&bitmap.xsize, &bitmap.ysize, color)
            },
        };
        self.draw_hatched_rect(drum.xpos, drum.ypos, *xsize, *ysize, 2, &color);
    }

    pub fn draw_plaintext_fw(&mut self, xpos: u32, ypos: u32, text: String, color: &RGB) {
        let mut position = 0;
        let size = 16.0;
        for c in text.chars() {
            if c.is_whitespace() {
                position += 8;
                continue;
            }
            let (metrics, char_bmp) = &self.main_font.rasterize(c, size);
            let c_x = xpos + position;
            let c_y = ypos; // meows
            let top_offset = size as u32 - metrics.height as u32;
            let left_offset = if c == 'i' {
                size as u32 - metrics.width as u32 - 5
            } else {
                size as u32 - metrics.width as u32 - 3
            }; // center it fr

            for y in 0..metrics.height {
                for x in 0..metrics.width {
                    if char_bmp[x + y*metrics.width] == 255 {
                        self.set_pixel(
                            c_x + x as u32 + left_offset, 
                            c_y + y as u32 + top_offset, 
                            color
                        ); 
                    }

                }     
            }
            position += 8;
        }
    }

    fn draw_plaintext(&mut self, xpos: u32, ypos: u32, size: f32, text: String, direction: TextDirection, color: &RGB) {
        let mut position = 0;
        for c in text.chars() {
            if c.is_whitespace() {
                position += 8;
                continue;
            }
            let (metrics, char_bmp) = &self.main_font.rasterize(c, size);
            let c_x = match direction {
                TextDirection::Horizontal => xpos + position,
                TextDirection::Vertical => xpos,
                TextDirection::Diagonal => xpos + position,
            };
            let c_y = match direction {
                TextDirection::Horizontal => ypos,
                TextDirection::Vertical => ypos + position,
                TextDirection::Diagonal => ypos + position,
            };
            
            let top_offset = match direction {
                TextDirection::Vertical => 0,
                _ => if c == '~' || c == '～' {
                    // epic special case for "nya~"
                    size as u32 / 2 - 1
                } else {
                    size as u32 - metrics.height as u32
                }
            };

            for y in 0..metrics.height {
                for x in 0..metrics.width {
                    if char_bmp[x + y*metrics.width] == 255 {
                        self.set_pixel(
                            c_x + x as u32, 
                            c_y + y as u32 + top_offset, 
                            color
                        ); 
                    }

                }     
            }
            //println!("{} {} {}", c, metrics.height, metrics.width);
            position += match direction {
                TextDirection::Vertical => metrics.height + 2,
                TextDirection::Horizontal => metrics.width,
                TextDirection::Diagonal => metrics.width,
            } as u32;

            position += 1; // spacing ><
        }
        //abort();
    }

    fn calc_text_width(&self, text: &str, size: f32) -> usize {
        let mut width = 0;
        for c in text.chars() {
            if c.is_whitespace() {
                width += 8;
                continue;
            } 
            let metrics = self.main_font.metrics(c, size);
            width += metrics.width + 1;
        }
        width
    }

    pub fn draw_text(&mut self, text: &mut Text) {
        if !text.visibility {
            return;
        }
        let line = &text.lines[text.current_line];
        let sliced = match &text.text_style {
            TextStyle::ByLine => line.to_owned(),
            TextStyle::ByWord => {
                let mut tokens = line.split_whitespace();
                let mut s = String::new();
                for _ in 0..text.line_progress {
                    s.push_str(tokens.next().unwrap_or_default());
                    s.push_str(" ");
                }
                s
            },
            TextStyle::ByCharacterUntilSubmit |
            TextStyle::ByCharacter => {
                let chars = line.chars().collect::<Vec<_>>();
                let progress = if text.line_progress > chars.len() {
                    chars.len()
                } else {
                    text.line_progress
                };
                chars[0..progress].iter().cloned().collect::<String>()
            },
            
        };
        self.draw_plaintext(text.xpos, text.ypos, 16.0, sliced, text.direction, &text.color);
    }

    pub fn draw_status_message(&mut self) {
        let time = 300;
        let (since, text, color) = match &self.status_message {
            StatusMessage::None => {
                return;
            },
            StatusMessage::Some { text, since, color } => {
                if since + time < self.tick {
                    // done showing
                    self.status_message = StatusMessage::None;
                    return;
                } else {
                    (since, text.to_string(), color)
                }
            },
        };
        let f = (time - (self.tick - since)) as f32 / time as f32 * 3.0;
        let ypos = self.height - 22;
        let width = self.calc_text_width(&text, 16.0) as u32;
        let main_color = color.dimmed(f);
        let shadow_color = color.dimmed(f.clamp(0.0, 1.0) - 0.8);

        self.draw_rect(4, ypos + 2, width + 8, 16, &shadow_color);
        self.draw_rect(2, ypos, width + 8, 16, &main_color);
        self.draw_plaintext(3, ypos-2, 16.0, text, TextDirection::Horizontal, &RGB::new(0, 0, 0));

    }

    pub fn set_status_message(&mut self, message: &str, color: RGB) {
        self.status_message = StatusMessage::Some { 
            text: message.to_string(), 
            since: self.tick,
            color
        };
    }

    fn draw_arrow(&mut self, xpos: u32, ypos: u32, direction: Direction) {
        // cute
        let arrow_bytes = include_bytes!("arrow.bmp");
        let arrow = Bitmap::from_data(arrow_bytes).unwrap();
        match direction {
            Direction::Up => todo!(),
            Direction::Down => {
                let down_arrow = arrow.rotated();
                self.draw_bitmap(xpos, ypos, None, &down_arrow);
            },
            Direction::Left => todo!(),
            Direction::Right => {
                self.draw_bitmap(xpos, ypos, None, &arrow);
            },
        }
    }

    pub fn draw_editor(&mut self, setup: &Setup, focus: &KeyboardFocus) {
        let mut xpos = 1;
        let right_xpos = self.width - 70;
        match focus.inst {
            Inst::Chord => {
                let has = match focus.get_focused_chord(&setup.chords) {
                    Some(i) => {
                        self.draw_ghost_chord(&setup.chords[i]);
                        if setup.chords[i].xpos < 70 {
                            xpos = right_xpos;
                        }
                        true
                    },
                    None => false
                };
                if focus.preview_all {
                    for chord in &setup.chords {
                        self.draw_ghost_chord(chord);
                        if chord.xpos < 70 {
                            xpos = right_xpos;
                        }
                    }
                }
                self.draw_plaintext_fw(xpos, 16, format!("Chrd {: >2}", focus.index+1), &if has{RGB::fff()} else{RGB::color("red")});
            },
            Inst::Drum => {
                let has = match focus.get_focused_drum(&setup.kit.drums) {
                    Some(i) => {
                        self.draw_ghost_drum(&setup.kit.drums[i]);
                        if setup.kit.drums[i].xpos < 70 && setup.kit.drums[i].ypos < 100 {
                            xpos = right_xpos;
                        }
                        true
                    },
                    None => false
                };
                if focus.preview_all {
                    for drum in &setup.kit.drums {
                        self.draw_ghost_drum(drum);
                        if drum.xpos < 70 && drum.ypos < 100 {
                            xpos = right_xpos;
                        }
                    }
                }

                self.draw_plaintext_fw(xpos, 16, format!("Drum {: >2}", focus.index+1), &if has{RGB::fff()} else{RGB::color("red")});
            },
        }
        self.draw_plaintext_fw(xpos, 1, "Editing".to_string(), &RGB::fff());

    }



}

pub enum StatusMessage {
    None,
    Some {
        text: String,
        since: usize,
        color: RGB
    }
}

