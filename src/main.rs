#![allow(dead_code, unused_imports, unused_variables, unstable)]
use std::io::{TcpListener, TcpStream, Acceptor, Listener, BufferedReader};
use std::thread::Thread;
use std::fmt::{Show, Formatter, Error};
use std::rand::random;
use std::iter::{count, repeat};
use std::sync::{Arc, Condvar};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::rc::Rc;
use std::collections::HashMap;

#[derive(Show, Copy)]
enum Dir {
    N, E, S, W
}

#[derive(Copy)]
enum PresentLocation {
    Unknown,
    Here,
    InDir(Dir),
}

struct Player {
    name: String,
    pos: (usize, usize),
}

struct PlayerInfo {
    name: String,
    pos: (usize, usize),
}

struct MazeInfo {
    width: usize,
    height: usize,
    walls: Vec<Vec<bool>>,
    present: (usize, usize),
}

struct Maze {
    info: Arc<MazeInfo>,
    sender: Sender<MazeMsg>,
}

struct Compass {
    north: usize,
    east: usize,
    south: usize,
    west: usize,
    present: PresentLocation,
}

enum MazeMsg {
    AddPlayer(String, (usize, usize), Sender<(u64, (usize, usize))>),
    RemovePlayer(u64),
    MovePlayer(u64, (usize, usize), Sender<()>),
    GetPlayers(Sender<Vec<PlayerInfo>>),
}

impl std::fmt::String for Compass {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), Error> {
        let p = match self.present {
            PresentLocation::Unknown => "?".to_string(),
            PresentLocation::Here => "X".to_string(),
            PresentLocation::InDir(d) => format!("{:?}", d),
        };

        write!(formatter, "N{} E{} S{} W{} P{}",
              self.north, self.east, self.south, self.west, p)
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
                sender.send((id, (0, 0))).unwrap();
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
                        PlayerInfo {
                            name: player.name.clone(),
                            pos: player.pos,
                        }
                    })
                    .collect())
                    .unwrap();
            },
        }
    }
}

impl Maze {
    fn new(width: usize, height: usize) -> Arc<Maze> {
        let present = (width / 2, height / 2);

        let (sender, receiver) = channel();

        let info = Arc::new(MazeInfo {
            width: width,
            height: height,
            walls: generate_maze(width, height, present),
            present: present,
        });

        let maze = Arc::new(Maze {
            info: info.clone(),
            sender: sender,
        });

        Thread::spawn(move || processor(receiver));

        maze
    }
}

fn add_player_to_maze(sender: &Sender<MazeMsg>, name: &str) -> Option<(u64, (usize, usize))> {
    let (result_sender, receiver) = channel();
    let msg = MazeMsg::AddPlayer(name.to_string(), (0, 0), result_sender);
    sender.send(msg).unwrap();

    receiver.recv().ok()
}

fn remove_player(sender: &Sender<MazeMsg>, id: u64) {
    let msg = MazeMsg::RemovePlayer(id);
    sender.send(msg).unwrap();
}

fn set_player_pos(sender: &Sender<MazeMsg>, id: u64, pos: (usize, usize)) {
    let (result_sender, receiver) = channel();
    let msg = MazeMsg::MovePlayer(id, pos, result_sender);
    sender.send(msg).unwrap();

    receiver.recv().unwrap();
}

fn move_pos((x, y): (usize, usize), d: Dir) -> (usize, usize) {
    match d {
        Dir::N => (x, y + 1),
        Dir::E => (x + 1, y),
        Dir::S => (x, y - 1),
        Dir::W => (x - 1, y),
    }
}

fn is_valid_maze_location(maze: &MazeInfo, (x, y): (usize, usize)) -> bool {
    x < maze.width && y < maze.height && !maze.walls[x][y]
}

fn measure_free(maze: &MazeInfo, mut pos: (usize, usize), d: Dir) -> usize {
    let mut c = 0us;
    loop {
        pos = move_pos(pos, d);
        if !is_valid_maze_location(maze, pos) {
            return c;
        }

        c += 1;
    }
}

fn get_player_compass(maze: &MazeInfo, pos: (usize, usize)) -> Compass {
    let (x, y) = pos;

    let present_info = match maze.present {
        (p_x, p_y) if p_x == x && p_y == y => Some((None, 0)),
        (p_x, p_y) if p_x == x && p_y > y => Some((Some(Dir::N), p_y - y)),
        (p_x, p_y) if p_x == x && p_y < y => Some((Some(Dir::S), y - p_y)),
        (p_x, p_y) if p_y == y && p_x > x => Some((Some(Dir::E), p_x - x)),
        (p_x, p_y) if p_y == y && p_x < x => Some((Some(Dir::W), x - p_x)),
        _ => None
    };

    let present = match present_info {
        Some((None, _)) => PresentLocation::Here,
        Some((Some(dir), dist)) if measure_free(maze, pos, dir) >= dist =>
            PresentLocation::InDir(dir),
        _ => PresentLocation::Unknown,
    };

    Compass {
        north: measure_free(maze, pos, Dir::N),
        east: measure_free(maze, pos, Dir::E),
        south: measure_free(maze, pos, Dir::S),
        west: measure_free(maze, pos, Dir::W),
        present: present,
    }
}

fn generate_maze(width: usize, height: usize, start: (usize, usize)) -> Vec<Vec<bool>> {
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
        let (new_x, new_y) = move_pos((wall_x, wall_y), d);

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

fn parse_msg(msg: &str) -> Option<Dir> {
    match msg {
        "N" | "n" => Some(Dir::N),
        "E" | "e" => Some(Dir::E),
        "S" | "s" => Some(Dir::S),
        "W" | "w" => Some(Dir::W),
        _ => None
    }
}

fn handle_client(mut stream: TcpStream, maze: Arc<MazeInfo>, sender: Sender<MazeMsg>) {
    let mut reader = BufferedReader::new(stream.clone());

    stream.write_line("Welcome to the reindeer maze! What is your team name?").unwrap();
    let name = reader.read_line().unwrap();
    let name = name.trim();

    let (id, mut pos) = match add_player_to_maze(&sender, name) {
        None => panic!(),
        Some(info) => info,
    };

    println!("{} joined", name);

    let compass = get_player_compass(&*maze, pos);
    println!("{} is at {:?}", name, pos);
    write!(&mut stream, "{}\n", compass).unwrap();

    for line in reader.lines() {
        let msg = line.unwrap();
        let msg = msg.trim();

        let d = match parse_msg(msg) {
            None => {
                stream.write_line("Bad command, plase try again").unwrap();
                continue;
            },
            Some(d) => d
        };

        pos = move_pos(pos, d);
        set_player_pos(&sender, id, pos);

        let compass = get_player_compass(&*maze, pos);
        println!("{} is at {:?}", name, pos);
        match compass.present {
            PresentLocation::Here => {
                println!("{} found the present", name);
            },
            _ => {},
        }

        write!(&mut stream, "{}\n", compass).unwrap();
    }

    remove_player(&sender, id);

    println!("{} disconnected", name);
}

fn main() {
    println!("Starting up...");

    let listener = TcpListener::bind("127.0.0.1:3000");
    let mut acceptor = listener.listen();

    let maze = Arc::new(Maze::new(5, 5));

    println!("Start listening...");
    for stream in acceptor.incoming() {
        match stream {
            Ok(stream) => {
                let sender = maze.sender.clone();
                let info = maze.info.clone();
                Thread::spawn(move || {
                    handle_client(stream, info, sender)
                });
            },
            Err(e) => {
                panic!("{}", e);
            },
        }
    }
}

