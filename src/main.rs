/*mod attacks;
mod board;
mod movegen;
use attacks::Tables;
use board::Board;
use movegen::perft;

fn main() {
    let t = Tables::new();

    let tests: [(&str, &str, [u64; 4]); 2] = [
        (
            "start position",
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            [20, 400, 8902, 197281],
        ),
        (
            "kiwipete",
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
            [48, 2039, 97862, 4085603],
        ),
    ];

    for &(name, fen, expected) in tests.iter() {
        let b = Board::from_fen(fen).unwrap();
        println!("{}", name);
        for d in 1..=4u32 {
            let n = perft(&b, d, &t);
            let ok = if n == expected[(d - 1) as usize] { "OK" } else { "BAD" };
            println!("  depth {}: {} (expected {}) {}", d, n, expected[(d - 1) as usize], ok);
        }
    }
}*/

mod attacks;
mod board;
mod eval;
mod movegen;
mod search;
mod uci;

fn main(){
    uci::run();
}