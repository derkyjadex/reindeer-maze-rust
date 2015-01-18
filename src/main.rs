#![allow(unstable)]

extern crate "reindeer-maze" as reindeer_maze;

use std::io::{TcpListener, TcpStream, Acceptor, Listener, BufferedReader};
use std::thread::Thread;

use reindeer_maze::data::{Dir, PresentLocation};
use reindeer_maze::maze::{Maze};

fn handle_client(mut stream: TcpStream, maze: Maze) {
    let mut reader = BufferedReader::new(stream.clone());

    stream.write_line("Welcome to the reindeer maze! What is your team name?").unwrap();
    let name = reader.read_line().unwrap();
    let name = name.trim();

    let mut player = maze.add_player(name).unwrap();

    println!("{} joined", name);

    let compass = player.get_compass();
    write!(&mut stream, "{}\n", compass).unwrap();

    for line in reader.lines() {
        let msg = line.unwrap();
        let msg = msg.trim();

        let d = match msg.parse::<Dir>() {
            None => {
                stream.write_line("Bad command, plase try again").unwrap();
                continue;
            },
            Some(d) => d
        };

        let pos = d.move_pos(player.pos);
        player.set_pos(pos);

        let compass = player.get_compass();
        match compass.present {
            PresentLocation::Here => {
                println!("{} found the present", name);
            },
            _ => {},
        }

        write!(&mut stream, "{}\n", compass).unwrap();
    }

    player.remove();

    println!("{} disconnected", name);
}

fn main() {
    println!("Starting up...");

    let listener = TcpListener::bind("127.0.0.1:3000");
    let mut acceptor = listener.listen();

    let maze = Maze::new(50, 50);

    println!("Start listening...");
    for stream in acceptor.incoming() {
        match stream {
            Ok(stream) => {
                let maze = maze.clone();
                Thread::spawn(move || {
                    handle_client(stream, maze)
                });
            },
            Err(e) => {
                panic!("{}", e);
            },
        }
    }
}

