use serde::{Serialize, Deserialize};
use midi_msg::{MidiMsg, ChannelVoiceMsg, Channel, ChannelModeMsg};

use crate::midi::*;
use crate::graphics::RGB;

#[derive(PartialEq, Serialize, Deserialize)]
#[serde(tag = "style")]
pub enum ChordStyle {
    #[serde(rename = "plain")]
    Plain,
    #[serde(rename = "decay")]
    Decay{ 
        time: u32 
    },
    #[serde(rename = "decay_release")]
    DecayRelease{
        time: u32
    },
    #[serde(rename = "grow")]
    Grow{
        speed: u32
    },
    #[serde(rename = "marquee")]
    Marquee{
        speed: u32
    },
}

#[derive(Serialize, Deserialize)]
pub struct Chord {
    pub xpos: u32,
    pub width: u32,
    #[serde(serialize_with = "unfrom_channel")]
    #[serde(deserialize_with = "from_channel")]
    pub channel: Channel,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub notes: Vec<Note>,
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub modulation: u16,
    pub velocity_sense: bool,
    pub style: ChordStyle,
    pub transpose: i32,
    pub color: RGB,
    pub shadow: RGB,
}
impl MidiProcessor for Chord{
    fn deal_with(&mut self, message: MidiMsg) {
        match message {
            MidiMsg::ChannelVoice {channel, msg} => {
                if channel == self.channel {
                    match msg {
                        ChannelVoiceMsg::NoteOn { note, velocity } => {
                            self.notes.push(
                                Note {
                                    pitch: note,
                                    velocity,
                                    age: 0
                                }
                            );
                        },
                        ChannelVoiceMsg::NoteOff { note, velocity: _ } => {
                            match self.style {
                                ChordStyle::DecayRelease { time: _ } => {
                                    // nop
                                }
                                ChordStyle::Marquee { speed: _ } => {
                                    // nop 
                                }
                                _ => {
                                    self.notes.retain(|x| x.pitch != note);
                                }
                            }
                            
                        },
                        ChannelVoiceMsg::ControlChange { control } => {
                            match control {
                                midi_msg::ControlChange::ModWheel(val) => {
                                    self.modulation = val;
                                },
                                _ => {},
                            }
                        }
                        _ => {}
                    }
                }
            },
            MidiMsg::ChannelMode { channel, msg } => {
                match msg {
                    ChannelModeMsg::AllNotesOff 
                    | ChannelModeMsg::AllSoundOff => {
                        self.notes.clear();
                    },
                    _ => {}
                }
            }
            _ => {}
        }
    }

}