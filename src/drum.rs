use serde::{Serialize, Deserialize};
use midi_msg::{MidiMsg, ChannelVoiceMsg, Channel};

use crate::{graphics::*, midi::MidiProcessor};

#[derive(Serialize, Deserialize)]
pub struct Kit {
    pub drums: Vec<Drum>,
}
impl MidiProcessor for Kit {
    fn deal_with(&mut self, message: MidiMsg) {
        match message {
            MidiMsg::ChannelVoice {channel, msg} => {
                if channel == Channel::Ch10 {
                    match msg {
                        ChannelVoiceMsg::NoteOn { note, velocity: _ } => {
                            for drum in &mut self.drums {
                                if drum.note == note {
                                    drum.state.trigger();
                                }
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

#[derive(Serialize, Deserialize)]
pub struct Drum {
    pub xpos: u32,
    pub ypos: u32,
    pub note: u8,
    pub gfx: DrumGfx,
    pub decay_time: u32,
    pub style: DrumStyle,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub state: DrumState
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "gfx")]
pub enum DrumGfx {
    #[serde(rename = "plain")]
    Plain {
        xsize: u32, 
        ysize: u32, 
        color: RGB
    },
    #[serde(rename = "bitmap")]
    Bitmap{
        bitmap: Bitmap
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum DrumStyle {
    #[serde(rename = "plain")]
    Plain,
    #[serde(rename = "decay")]
    Decay,
}

#[derive(Default, PartialEq)]
pub struct DrumState {
    pub triggered: bool,
    pub age: u32,
}
impl DrumState {
    pub fn trigger(&mut self) {
        self.triggered = true;
        self.age = 0;
    }
    pub fn reset(&mut self) {
        self.triggered = false;
        self.age = 0;
    }

    pub fn tick_or_reset(&mut self, decay_time: u32) -> bool{
        if self.age >= decay_time {
            self.reset();
            false
        } else {
            self.age += 1;
            true
        }
    }
}
