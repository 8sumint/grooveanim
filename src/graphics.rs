use serde::{Serialize, Deserialize, Deserializer, Serializer};
use std::error::Error;
use image::Pixel;


#[derive(Clone, Copy)]
pub struct RGB {
    r: u8,
    g: u8,
    b: u8,
}

impl RGB {
    pub fn new(r:u8, g:u8, b:u8) -> Self {
        RGB { r, g, b }
    }

    pub fn default() -> Self {
        RGB {r:0,g:0,b:0}
    }
    
    pub fn irl(&self) -> u32 {
        ((self.r as u32) << 16) + ((self.g as u32) << 8) + (self.b as u32)
    }

    pub fn dimmed(&self, factor: f32) -> Self {
        let f = (factor + 0.2).clamp(0.0, 1.0);
        let r = (self.r as f32 * f) as u8;
        let g = (self.g as f32 * f) as u8;
        let b = (self.b as f32 * f) as u8;
        RGB { r, g, b }
    }

    pub fn fff() -> Self {
        RGB::new(255,255,255)
    }

    pub fn color(arg: &str) -> RGB {
        match arg {
            "red" => RGB::new(255,0,0),
            _ => panic!("unknown color {}", arg)
        }
    }
}

impl Serialize for RGB {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex = format!("{:02X}{:02X}{:02X}", self.r, self.g, self.b);
        serializer.serialize_str(&format!("#{}", hex))
    }
}

impl<'de> Deserialize<'de> for RGB {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        let s = s.trim_start_matches('#');
        match s.len() {
            3 => {
                let r = u8::from_str_radix(&s[0..1].repeat(2), 16).map_err(serde::de::Error::custom)?;
                let g = u8::from_str_radix(&s[1..2].repeat(2), 16).map_err(serde::de::Error::custom)?;
                let b = u8::from_str_radix(&s[2..3].repeat(2), 16).map_err(serde::de::Error::custom)?;
                Ok(RGB { r,g,b })
            }
            6 => {
                let r = u8::from_str_radix(&s[0..2], 16).map_err(serde::de::Error::custom)?;
                let g = u8::from_str_radix(&s[2..4], 16).map_err(serde::de::Error::custom)?;
                let b = u8::from_str_radix(&s[4..6], 16).map_err(serde::de::Error::custom)?;
                Ok(RGB { r,g,b })
            }
            _ => Err(serde::de::Error::custom("Invalid color format")),
        }
    }
}

pub struct Bitmap {
    pub path: String,
    pub data: Vec<RGB>,
    pub xsize: u32,
    pub ysize: u32,
}
impl Bitmap {
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let image = image::open(path)?;
        let image = image.as_rgb8().expect("failed to load img as rgb8");
        let xsize = image.width();
        let ysize = image.height();
        let mut data: Vec<RGB> = vec![RGB::default(); (xsize * ysize) as usize];
        for (i, px) in image.pixels().enumerate() {
            let combined = RGB {r: px.channels()[0], g: px.channels()[1], b: px.channels()[2]};
            data[i] = combined;
        }
        Ok(Bitmap {
            path: path.to_string(),
            data,
            xsize,
            ysize
        })
    }
    pub fn color(&self, x: u32, y: u32, dimmed: Option<f32>) -> u32 {
        let buf_pos = ((y as usize) * self.xsize as usize) + x as usize;
        if buf_pos >= self.data.len() {
            panic!("meow {} no {} {}", self.data.len(), x, y);
        } else {
            match dimmed {
                Some(f) => self.data[buf_pos].dimmed(f).irl(),
                None => self.data[buf_pos].irl()
            }
        }
    }

    pub fn first_nonzero_color(&self) -> RGB {
        for c in &self.data {
            if c.irl() != 0 {
                return *c
            }
        }
        RGB::new(255,0,255)
    }

    pub fn from_data(data: &[u8]) -> Result<Self, Box<dyn Error>> {
        let image = image::load_from_memory(data)?;
        let image = image.as_rgb8().expect("failed to load img as rgb8");
        let xsize = image.width();
        let ysize = image.height();
        let mut data: Vec<RGB> = vec![RGB::default(); (xsize * ysize) as usize];
        for (i, px) in image.pixels().enumerate() {
            let combined = RGB {r: px.channels()[0], g: px.channels()[1], b: px.channels()[2]};
            data[i] = combined;
        }
        Ok(Bitmap {
            path: "".to_string(),
            data,
            xsize,
            ysize
        })
    }

    pub fn rotated(self) -> Self {
        let ysize = self.xsize;
        let xsize = self.ysize;
        let mut new_data = vec![RGB::default(); self.data.len()];

        for y in 0..self.ysize {
            for x in 0..self.xsize {
                let orig_buf_pos = ((y as usize) * self.xsize as usize) + x as usize;
                let buf_pos = ((x as usize) * xsize as usize) + y as usize;
                new_data[buf_pos] = self.data[orig_buf_pos];
            }
        }
        Self {
            path: self.path,
            ysize,
            xsize,
            data: new_data,
        }
    }

}
impl Serialize for Bitmap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.path)
    }
}

impl<'de> Deserialize<'de> for Bitmap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let path = String::deserialize(deserializer)?;
        Bitmap::load_from_file(&path).map_err(serde::de::Error::custom)
    }
}