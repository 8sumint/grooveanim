use serde::{Serialize, Deserialize};
use midi_msg::{MidiMsg, ChannelVoiceMsg, Channel};

use crate::midi::*;
use crate::graphics::RGB;

#[derive(Serialize, Deserialize)]
pub enum TextStyle {
    ByLine,
    ByWord,
    ByCharacter,
    ByCharacterUntilSubmit,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum TextDirection {
    #[serde(rename = "horizontal")]
    Horizontal,
    #[serde(rename = "vertical")]
    Vertical,
    #[serde(rename = "diagonal")]
    Diagonal
}

#[derive(Serialize, Deserialize)]
pub struct Text {
    pub xpos: u32,
    pub ypos: u32,
    #[serde(serialize_with = "unfrom_channel")]
    #[serde(deserialize_with = "from_channel")]
    pub channel: Channel,
    pub base_note: u8,
    pub direction: TextDirection,
    pub lines: Vec<String>,
    pub text_style: TextStyle,
    pub color: RGB,
    pub visibility: bool,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub current_line: usize,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub line_progress: usize,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    buffered_progress: usize,
}
impl MidiProcessor for Text {
    fn deal_with(&mut self, message: MidiMsg) {
        match message {
            MidiMsg::ChannelVoice {channel, msg} => {
                if channel == self.channel {
                    match msg {
                        ChannelVoiceMsg::NoteOn { note, velocity } => {
                            let n_base = 30;
                            let n_next = n_base + 1;
                            let n_two = n_base + 2;
                            let n_submit = n_base + 3;
                            let n_visible = n_base + 4;
                            let n_reset = n_base + 5;
                            let n_mode = n_base + 6;
                            
                            match note {
                                _x if note == n_base => {
                                    if self.current_line < self.lines.len() - 1 {
                                        self.current_line += 1;
                                        self.line_progress = 0;
                                        self.buffered_progress = 0;
                                    }
                                },
                                _x if note == n_next || note == n_two => {
                                    let number = if note == n_two {2} else {1};
                                    match self.text_style {
                                        TextStyle::ByLine => {
                                            if self.current_line < self.lines.len() - 1 {
                                                self.current_line += 1;
                                            }
                                        },
                                        TextStyle::ByWord => {
                                            self.line_progress += number;
                                        },
                                        TextStyle::ByCharacter => {
                                            self.line_progress += number;
                                            self.buffered_progress += number;
                                        },
                                        TextStyle::ByCharacterUntilSubmit => {
                                            self.buffered_progress += number;
                                        },
                                    }
                                },
                                _x if note == n_submit => {
                                    self.line_progress = self.buffered_progress;
                                },
                                _x if note == n_visible => {
                                    if velocity > 64 {
                                        self.visibility = true;
                                    } else {
                                        self.visibility = false;
                                    }
                                }
                                _x if note == n_reset => {
                                    self.current_line = 0;
                                    self.line_progress = 0;
                                    self.buffered_progress = 0;
                                },
                                _x if note == n_mode => {
                                    if velocity < 32 {
                                        self.text_style = TextStyle::ByLine;
                                    } else if velocity < 64 {
                                        self.text_style = TextStyle::ByWord;
                                    } else if velocity < 96 {
                                        self.text_style = TextStyle::ByCharacter;
                                    } else {
                                        self.text_style = TextStyle::ByCharacterUntilSubmit;
                                    }
                                    
                                },
                                _ => {}
                            }
                            
                        },
                        _ => {}
                    }
                }
            },
            _ => {}
        }
    }
}