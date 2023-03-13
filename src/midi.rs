use serde::{Serialize, Deserialize, Deserializer, Serializer};
use midi_msg::MidiMsg;


#[derive(Serialize, Deserialize, Clone)]
pub struct Note {
    pub pitch: u8,
    pub velocity: u8,
    pub age: u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Ch1,
    Ch2,
    Ch3,
    Ch4,
    Ch5,
    Ch6,
    Ch7,
    Ch8,
    Ch9,
    Ch10,
    Ch11,
    Ch12,
    Ch13,
    Ch14,
    Ch15,
    Ch16
}

impl std::convert::From<u8> for Channel {
    fn from(i: u8) -> Self {
        match i {
            0 => Channel::Ch1,
            1 => Channel::Ch2,
            2 => Channel::Ch3,
            3 => Channel::Ch4,
            4 => Channel::Ch5,
            5 => Channel::Ch6,
            6 => Channel::Ch7,
            7 => Channel::Ch8,
            8 => Channel::Ch9,
            9 => Channel::Ch10,
            10 => Channel::Ch11,
            11 => Channel::Ch12,
            12 => Channel::Ch13,
            13 => Channel::Ch14,
            14 => Channel::Ch15,
            _ => Channel::Ch16,
        }
    }
}

impl Serialize for Channel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let c = *self as u8 + 1;
        serializer.serialize_str(&c.to_string())
    }
}

impl<'de> Deserialize<'de> for Channel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        let channel_no = s.parse::<u8>().unwrap();
        Ok(Channel::from(channel_no - 1))
    }
}

pub trait MidiProcessor {
    fn deal_with(&mut self, message: MidiMsg);
}

