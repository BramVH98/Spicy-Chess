#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Color {
    White = 0,
    Black = 1,
}

pub const PAWN: usize = 0;
pub const KNIGHT: usize = 1;
pub const BISHOP: usize = 2;
pub const ROOK: usize = 3;
pub const QUEEN: usize = 4;
pub const KING: usize = 5;

#[derive(Clone)]
pub struct Board {
    pub pieces: [u64; 6],
    pub color: [u64; 2],
    pub side: Color,
    pub castling: u8,
    pub ep: Option<u8>,
    pub halfmove: u32,
    pub fullmove: u32,
}

impl Board {
    pub fn from_fen(fen: &str) -> Result<Board, String> {
        let parts: Vec<&str> = fen.split_whitespace().collect();
        if parts.len() < 4 {
            return Err("bad FEN".to_string());
        }

        let mut b = Board {
            pieces: [0; 6],
            color: [0; 2],
            side: Color::White,
            castling: 0,
            ep: None,
            halfmove: 0,
            fullmove: 1,
        };

        let mut rank: i32 = 7;
        let mut file: i32 = 0;
        for c in parts[0].chars(){
            match c {
                '/' => {
                    rank -= 1;
                    file = 0;
                }
                '1'..='8' => {
                    file += c.to_digit(10).unwrap() as i32;
                }
                _ => {
                    let color = if c.is_uppercase() { 0 } else { 1 };
                    let piece = match c.to_ascii_lowercase() {
                        'p' => PAWN,
                        'n' => KNIGHT,
                        'b' => BISHOP,
                        'r' => ROOK,
                        'q' => QUEEN,
                        'k' => KING,
                        _ => return Err("bad piece".to_string()),
                    };
                    if rank < 0 || file > 7{
                        return Err("bad board".to_string());
                    }
                    let bit = 1u64 << (rank * 8 + file);
                    b.pieces[piece] |= bit;
                    b.color[color] |= bit;
                    file += 1;
                }
        }
    }


    b.side = if parts[1] == "w" { Color::White } else { Color::Black };

    for c in parts[2].chars(){
        match c {
            'K' => b.castling |= 1,
            'Q' => b.castling |= 2,
            'k' => b.castling |= 4,
            'q' => b.castling |= 8,
            _ => {}
        }
    }

    if parts[3] != "-" {
        let bytes = parts[3].as_bytes();
        if bytes.len() == 2{
            b.ep = Some((bytes[1] - b'1') * 8 + (bytes[0] - b'a'));
        }
    }

    if parts.len() > 4 {
        b.halfmove = parts[4].parse().unwrap_or(0);
    }
    if parts.len() > 5 {
        b.fullmove = parts[5].parse().unwrap_or(1);
    }
    Ok(b)
}

pub fn print(&self) {
    let letters = ['p', 'n', 'b', 'r', 'q', 'k'];
    for rank in (0..8).rev(){
        let mut line = format!("{} ", rank + 1);
        for file in 0..8{
            let bit = 1u64 << (rank * 8 + file);
            let mut ch = '.';
            for p in 0..6{
                if self.pieces[p] & bit != 0{
                    ch = letters[p];
                    if self.color[0] & bit != 0{
                        ch = ch.to_ascii_uppercase();
                    }
                }
            }
            line.push(ch);
            line.push(' ');
        }
        println!("{}", line);
    }
    println!("  a b c d e f g h");
}

}

