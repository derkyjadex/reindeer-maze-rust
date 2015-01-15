use std::fmt;
use std::str::FromStr;

pub type Pos = (usize, usize);

#[derive(Show, Copy)]
pub enum Dir {
    N, E, S, W
}

impl Dir {
    pub fn move_pos(self, (x, y): Pos) -> Pos {
        match self {
            Dir::N => (x, y + 1),
            Dir::E => (x + 1, y),
            Dir::S => (x, y - 1),
            Dir::W => (x - 1, y),
        }
    }
}

impl FromStr for Dir {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "N" | "n" => Some(Dir::N),
            "E" | "e" => Some(Dir::E),
            "S" | "s" => Some(Dir::S),
            "W" | "w" => Some(Dir::W),
            _ => None
        }
    }
}

#[derive(Copy)]
pub enum PresentLocation {
    Unknown,
    Here,
    InDir(Dir),
}

#[derive(Copy)]
pub struct Compass {
    pub north: usize,
    pub east: usize,
    pub south: usize,
    pub west: usize,
    pub present: PresentLocation,
}

impl fmt::String for Compass {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let p = match self.present {
            PresentLocation::Unknown => "?".to_string(),
            PresentLocation::Here => "X".to_string(),
            PresentLocation::InDir(d) => format!("{:?}", d),
        };

        write!(formatter, "N{} E{} S{} W{} P{}",
            self.north, self.east, self.south, self.west, p)
    }
}

