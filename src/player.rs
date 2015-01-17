use std::sync::mpsc::channel;

use data::{Pos, Dir, PresentLocation, Compass};
use maze::{MazeMsg, MazeHandle};

#[derive(Clone)]
pub struct Player {
    pub name: String,
    pub pos: Pos,
}

pub struct PlayerHandle<'a> {
    pub id: u64,
    pub maze: &'a MazeHandle<'a>,
    pub pos: Pos,
}

impl<'a> PlayerHandle<'a> {
    pub fn remove(&self) {
        let msg = MazeMsg::RemovePlayer(self.id);
        self.maze.sender.send(msg).unwrap();
    }

    pub fn set_pos(&mut self, pos: Pos) {
        let (sender, receiver) = channel();
        let msg = MazeMsg::MovePlayer(self.id, pos, sender);
        self.maze.sender.send(msg).unwrap();

        receiver.recv().unwrap();
        self.pos = pos;
    }

    pub fn get_compass(&self) -> Compass {
        let (x, y) = self.pos;

        let present_info = match self.maze.info.present {
            (p_x, p_y) if p_x == x && p_y == y => Some((None, 0)),
            (p_x, p_y) if p_x == x && p_y > y => Some((Some(Dir::N), p_y - y)),
            (p_x, p_y) if p_x == x && p_y < y => Some((Some(Dir::S), y - p_y)),
            (p_x, p_y) if p_y == y && p_x > x => Some((Some(Dir::E), p_x - x)),
            (p_x, p_y) if p_y == y && p_x < x => Some((Some(Dir::W), x - p_x)),
            _ => None
        };

        let present = match present_info {
            Some((None, _)) => PresentLocation::Here,
            Some((Some(dir), dist)) if self.maze.measure_free(self.pos, dir) >= dist =>
                PresentLocation::InDir(dir),
            _ => PresentLocation::Unknown,
        };

        Compass {
            north: self.maze.measure_free(self.pos, Dir::N),
            east: self.maze.measure_free(self.pos, Dir::E),
            south: self.maze.measure_free(self.pos, Dir::S),
            west: self.maze.measure_free(self.pos, Dir::W),
            present: present,
        }
    }
}
