use std::{error::Error, fs::File, io::Write};

use serde::{Serialize, Deserialize};

use crate::{chord::Chord, drum::Kit, text::Text};


#[derive(Serialize, Deserialize)]
pub struct Setup {
    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub path: String,

    pub midi_port_id: usize,
    pub global_transpose: i32,
    pub chords: Vec<Chord>,
    pub kit: Kit,
    pub texts: Vec<Text>,
}

impl Setup {
    pub fn load(path: &str) -> Result<Self, Box<dyn Error>> {
        let text = std::fs::read_to_string(path).expect("can't find setup json");
        let mut setup: Setup = serde_json::from_str(&text)?;
        setup.path = path.to_string();
        Ok(setup)
    }
    pub fn maybe_write(&self) -> Result<usize, Box<dyn Error>> {
        let mut buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
        self.serialize(&mut ser).unwrap();
        let text = String::from_utf8(buf).unwrap();
        let mut output = File::create(&self.path)?;
        write!(output, "{}", text)?;
        Ok(text.len())
    }
}