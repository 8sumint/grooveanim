use serde::{Serialize, Deserialize, Deserializer, Serializer};
use midi_msg::{Channel, MidiMsg};


#[derive(Serialize, Deserialize, Clone)]
pub struct Note {
    pub pitch: u8,
    pub velocity: u8,
    pub age: u32
}

pub fn unfrom_channel<S>(channel: &Channel, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer
{
    let c = *channel as u8 + 1;
    serializer.serialize_str(&c.to_string())
}

pub fn from_channel<'de, D>(deserializer: D) -> Result<Channel, D::Error>
where
    D: Deserializer<'de>
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    match s {
        "1"  => Ok(Channel::Ch1),
        "2"  => Ok(Channel::Ch2),
        "3"  => Ok(Channel::Ch3),
        "4"  => Ok(Channel::Ch4),
        "5"  => Ok(Channel::Ch5),
        "6"  => Ok(Channel::Ch6),
        "7"  => Ok(Channel::Ch7),
        "8"  => Ok(Channel::Ch8),
        "9"  => Ok(Channel::Ch9),
        "10" => Ok(Channel::Ch10),
        "11" => Ok(Channel::Ch11),
        "12" => Ok(Channel::Ch12),
        "13" => Ok(Channel::Ch13),
        "14" => Ok(Channel::Ch14),
        "15" => Ok(Channel::Ch15),
        "16" => Ok(Channel::Ch16),
        &_ => Err(serde::de::Error::custom("invalid channel"))
    }
}

pub trait MidiProcessor {
    fn deal_with(&mut self, message: MidiMsg);
}

