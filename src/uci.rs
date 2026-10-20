use crate::attacks::Tables;
use crate::board::{Board, Color};
use crate::search::Searcher;
use std::io::{self, BufRead};

const START: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

pub fn run(){
    let t = Tables::new();
    let mut board = Board::from_fen(START).unwrap();
    let stdin = io::stdin();
    for line in stdin.lock().lines(){
        let line = match line{
            Ok(l) => l,
            Err(_) => break,
        };
        let tok: Vec<&str> = line.split_whitespace().collect();
        if tok.is_empty(){
            continue;
        }
        match tok[0]{
            "uci" =>{
                println!("id name SpicyChess");
                println!("id author BramVH");
                println!("uciok");
            }
            "isready" => println!("readyok"),
            "ucinewgame" => board = Board::from_fen(START).unwrap(),
            "position" =>{
                if let Some(b) = parse_position(&tok, &t){
                    board = b;
                }
            }
            "go" => go(&board, &tok, &t),
            "quit" => break,
            _ =>{}
        }
    }
}

fn parse_position(tok: &[&str], t: &Tables) -> Option<Board>{
    let mut b: Board;
    let i;
    if tok.get(1) == Some(&"startpos"){
        b = Board::from_fen(START).ok()?;
        i = 2;
    } else if tok.get(1) == Some(&"fen"){
        let mut j = 2;
        let mut parts = Vec::new();
        while j < tok.len() && tok[j] != "moves"{
            parts.push(tok[j]);
            j += 1;
        }
        b = Board::from_fen(&parts.join(" ")).ok()?;
        i = j;
    } else {
        return None;
    }
    if tok.get(i) == Some(&"moves"){
        for s in &tok[i + 1..]{
            let legal = b.legal_moves(t);
            match legal.into_iter().find(|m| m.to_uci() == *s){
                Some(m) => b = b.make_move(m),
                None => return None,
            }
        }
    }
    Some(b)
}

fn go(b: &Board, tok: &[&str], t: &Tables){
    let mut depth: i32 = 64;
    let mut movetime: i64 = -1;
    let (mut wtime, mut btime, mut winc, mut binc) = (-1i64, -1i64, 0i64, 0i64);

    let mut i = 1;
    while i < tok.len(){
        let val = tok.get(i + 1).and_then(|s| s.parse::<i64>().ok());
        if let Some(v) = val{
            match tok[i]{
                "depth" => depth = v as i32,
                "movetime" => movetime = v,
                "wtime" => wtime = v,
                "btime" => btime = v,
                "winc" => winc = v,
                "binc" => binc = v,
                _ => {}
            }
        }
        i += 1;
    }

    let (my, inc) = if b.side == Color::White { (wtime, winc) } else { (btime, binc) };
    let limit: u128 = if movetime >= 0{
        movetime as u128
    } else if my >= 0{
        let alloc = my / 30 + inc / 2;
        alloc.max(50).min((my - 50).max(10)) as u128
    } else if depth != 64{
        3_600_000
    } else{
        5000
    };

    let mut s = Searcher::new();
    match s.think(b, depth, limit, t){
        Some(m) => println!("bestmove {}", m.to_uci()),
        None => println!("bestmove 0000"),
    }
}