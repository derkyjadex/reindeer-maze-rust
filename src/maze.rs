use std::sync::mpsc::{channel, Sender, Receiver};
use std::sync::{Arc};
use std::thread::Thread;
use std::collections::HashMap;
use std::iter::{repeat};
use std::rand::random;

use data::{Pos, Dir};
use player::{Player, PlayerHandle};

pub struct MazeInfo {
    pub width: usize,
    pub height: usize,
    pub walls: Vec<Vec<bool>>,
    pub present: Pos,
}

#[derive(Clone)]
pub struct Maze {
    pub info: Arc<MazeInfo>,
    pub sender: Sender<MazeMsg>,
}

pub enum MazeMsg {
    AddPlayer(String, Pos, Sender<(u64, Pos)>),
    RemovePlayer(u64),
    MovePlayer(u64, Pos, Sender<()>),
    GetPlayers(Sender<Vec<Player>>),
}

impl Maze {
    pub fn new(width: usize, height: usize) -> Maze {
        let present = (width / 2, height / 2);

        let (sender, receiver) = channel();

        let info = Arc::new(MazeInfo {
            width: width,
            height: height,
            walls: generate_maze(width, height, present),
            present: present,
        });

        let maze = Maze {
            info: info.clone(),
            sender: sender,
        };

        Thread::spawn(move || processor(receiver));

        maze
    }

    pub fn add_player<'a>(&'a self, name: &str) -> Option<PlayerHandle<'a>> {
        let pos = (random::<usize>() % self.info.width, random::<usize>() % self.info.height);
        println!("Adding player at {:?}", pos);

        let (result_sender, receiver) = channel();
        let msg = MazeMsg::AddPlayer(name.to_string(), pos, result_sender);
        self.sender.send(msg).unwrap();

        receiver.recv().ok().map(|(id, pos)| PlayerHandle {
            id: id,
            maze: self,
            pos: pos,
        })
    }

    pub fn is_valid_location(&self, (x, y): Pos) -> bool {
        x < self.info.width && y < self.info.height && !self.info.walls[x][y]
    }

    pub fn measure_free(&self, mut pos: Pos, d: Dir) -> usize {
        let mut c = 0us;
        loop {
            pos = d.move_pos(pos);
            if !self.is_valid_location(pos) {
                return c;
            }

            c += 1;
        }
    }
}

fn processor(receiver: Receiver<MazeMsg>) {
    let mut last_id = 0u64;
    let mut players = HashMap::new();

    for msg in receiver.iter() {
        match msg {
            MazeMsg::AddPlayer(name, pos, sender) => {
                let player = Player { name: name, pos: pos };
                let id = last_id + 1;
                last_id = id;
                players.insert(id, player);
                sender.send((id, pos)).unwrap();
            },
            MazeMsg::RemovePlayer(id) => {
                players.remove(&id);
            },
            MazeMsg::MovePlayer(id, pos, sender) => {
                match players.get_mut(&id) {
                    Some(player) => {
                        player.pos = pos;
                    },
                    None => (),
                }
                sender.send(()).unwrap();
            },
            MazeMsg::GetPlayers(sender) => {
                sender.send(players.iter()
                    .map(|(_, player)| {
                        player.clone()
                    })
                    .collect())
                    .unwrap();
            },
        }
    }
}

fn generate_maze(width: usize, height: usize, start: Pos) -> Vec<Vec<bool>> {
    let mut walls: Vec<Vec<bool>> = range(0, width)
        .map(|_| repeat(true).take(height).collect())
        .collect();

    let (start_x, start_y) = start;
    walls[start_x][start_y] = false;

    let mut wall_list = vec![];

    if start_x > 0 {
        wall_list.push((start_x - 1, start_y, Dir::W));
    }
    if start_x < width - 1 {
        wall_list.push((start_x + 1, start_y, Dir::E));
    }
    if start_y > 0 {
        wall_list.push((start_x, start_y - 1, Dir::S));
    }
    if start_y < height - 1 {
        wall_list.push((start_x, start_y + 1, Dir::N));
    }

    while wall_list.len() > 0 {
        let i = random::<usize>() % wall_list.len();
        let (wall_x, wall_y, d) = wall_list.remove(i);
        let (new_x, new_y) = d.move_pos((wall_x, wall_y));

        if /*new_x < 0 ||*/ new_x > width - 1 || /*new_y < 0 ||*/ new_y > height - 1 {
            walls[wall_x][wall_y] = false;

        } else if walls[wall_x][wall_y] && walls[new_x][new_y] {
            walls[wall_x][wall_y] = false;
            walls[new_x][new_y] = false;

            if new_x > 0 && walls[new_x - 1][new_y] {
                wall_list.push((new_x - 1, new_y, Dir::W));
            }
            if new_x < width - 1 && walls[new_x + 1][new_y] {
                wall_list.push((new_x + 1, new_y, Dir::E));
            }
            if new_y > 0 && walls[new_x][new_y - 1] {
                wall_list.push((new_x, new_y - 1, Dir::S));
            }
            if new_y < width - 1 && walls[new_x][new_y + 1] {
                wall_list.push((new_x, new_y + 1, Dir::N));
            }
        }
    }

    walls
}
