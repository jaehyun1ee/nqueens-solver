use clap::Parser;
use z3::{ast::Bool, Config, Context, Model, SatResult, Solver};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value_t = 8)]
    count: usize,
    #[arg(short, long, default_value_t = true)]
    unique: bool,

}

fn encode_row<'c>(board: &Vec<Vec<Bool<'c>>>, context: &'c Context) -> Bool<'c> {
    let n = board.len();

    // Definedness for row.
    let mut define = Vec::new();
    for r in 0..n {
        let mut row = Vec::new();
        for c in 0..n {
            row.push(board[r][c].clone());
        }
        let row: Vec<&Bool<'_>> = row.iter().collect();
        define.push(Bool::or(&context, row.as_slice()));
    }
    let define: Vec<&Bool<'_>> = define.iter().collect();

    // Uniqueness for row.
    let mut unique = Vec::new();
    for r in 0..n {
        for i in 0..n {
            for j in (i + 1)..n {
                unique.push(Bool::and(&context, &[&board[r][i], &board[r][j]]).not())
            }
        }
    }
    let unique: Vec<&Bool<'_>> = unique.iter().collect(); 

    Bool::and(&context, &[&define[..], &unique[..]].concat())
}

fn encode_column<'c>(board: &Vec<Vec<Bool<'c>>>, context: &'c Context) -> Bool<'c> {
    let n = board.len();

    // Definedness for column.
    let mut define = Vec::new();
    for c in 0..n {
        let mut row = Vec::new();
        for r in 0..n {
            row.push(board[r][c].clone());
        }
        let row: Vec<&Bool<'_>> = row.iter().collect();
        define.push(Bool::or(&context, row.as_slice()));
    }
    let define: Vec<&Bool<'_>> = define.iter().collect();

    // Uniqueness for column.
    let mut unique = Vec::new();
    for c in 0..n {
        for i in 0..n {
            for j in (i + 1)..n {
                unique.push(Bool::and(&context, &[&board[i][c], &board[j][c]]).not());
            }
        }
    }
    let unique: Vec<&Bool<'_>> = unique.iter().collect(); 

    Bool::and(&context, &[&define[..], &unique[..]].concat())
}

fn encode_diagonal<'c>(board: &Vec<Vec<Bool<'c>>>, context: &'c Context) -> Bool<'c> {
    let n = board.len();

    let mut define = Vec::new();

    // Uniqueness for major diagonal.
    for d in 0..n {
        for i in 0..(n - d) {
            for j in (i + 1)..(n - d) {
                define.push(Bool::and(&context, &[&board[i + d][i], &board[j + d][j]]).not());
            }
        }
    }
    for d in 1..n {
        for i in d..n {
            for j in (i + 1)..n {
                define.push(Bool::and(&context, &[&board[i - d][i], &board[j - d][j]]).not());
            }
        }
    }

    // Uniqueness for minor diagonal.
    for s in 0..n {
        for i in 0..=s {
            for j in (i + 1)..=s {
                define.push(Bool::and(&context, &[&board[i][s - i], &board[j][s - j]]).not())
            }
        }
    }
    for s in n..(2 * n - 1) {
        for i in (s - n + 1)..n {
            for j in (i + 1)..n {
                define.push(Bool::and(&context, &[&board[i][s - i], &board[j][s - j]]).not())
            }
        }
    }

    let define: Vec<&Bool<'_>> = define.iter().collect();
    Bool::and(&context, define.as_slice())
}

fn encode_board<'c>(n: usize, context: &'c Context) -> Vec<Vec<Bool<'c>>> {
    let mut board = Vec::new();
    for r in 0..n {
        let mut row = Vec::new();
        for c in 0..n {
            let cell = Bool::new_const(&context, format!("({},{})", r, c)); 
            row.push(cell);
        }
        board.push(row);
    }

    board
}

fn hflip<'c>(queens: &Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    let n = queens.len();
    queens.iter().map(|(r, c)| (n - *r - 1, *c)).collect()
}

fn vflip<'c>(queens: &Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    let n = queens.len();
    queens.iter().map(|(r, c)| (*r, n - *c - 1)).collect()
}

fn maflip<'c>(queens: &Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    queens.iter().map(|(r, c)| (*c, *r)).collect()
}

fn miflip<'c>(queens: &Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    let n = queens.len();
    queens.iter().map(|(r, c)| (n - *c - 1, n - *r - 1)).collect()
}

fn flip<'c>(queens: &Vec<(usize, usize)>, fs: &Vec<Box<dyn Fn(&Vec<(usize, usize)>) -> Vec<(usize, usize)>>>) -> Vec<(usize, usize)> {
    let mut flipped = queens.clone();
    for f in fs {
        flipped = f(&flipped);
    }
    flipped
}

fn decode_board<'c>(unique: bool, board: &Vec<Vec<Bool<'c>>>, solution: &Model<'c>, context: &'c Context) -> Bool<'c> {
    let n = board.len();

    // Collect queen's positions.
    let mut queens = Vec::new();
    for r in 0..n {
        for c in 0..n {
            let cell = &board[r][c];
            let cell = Bool::as_bool(&solution.eval(cell, true).unwrap()).unwrap();

            if cell {
                queens.push((r, c));
            }
        }
    }

    let flips: Vec<Vec<Box<dyn Fn(&Vec<(usize, usize)>) -> Vec<(usize, usize)>>>> = if unique { 
        vec![
            // No flip.
            vec![],
            // One flip.
            vec![ Box::new(hflip) ], vec![ Box::new(vflip) ],
            vec![ Box::new(maflip) ], vec![ Box::new(miflip) ],
            vec![ Box::new(hflip), Box::new(vflip) ],
            // Two flips.
            vec![ Box::new(maflip), Box::new(miflip) ],
            vec![ Box::new(hflip), Box::new(maflip) ],
            vec![ Box::new(hflip), Box::new(miflip) ],
            vec![ Box::new(vflip), Box::new(maflip) ],
            vec![ Box::new(vflip), Box::new(miflip) ],
            // Three flips.
            vec![ Box::new(hflip), Box::new(vflip), Box::new(maflip) ],
            vec![ Box::new(hflip), Box::new(vflip), Box::new(miflip) ],
            vec![ Box::new(hflip), Box::new(maflip), Box::new(miflip) ],
            vec![ Box::new(vflip), Box::new(maflip), Box::new(miflip) ],
            // All flips.
            vec![ Box::new(hflip), Box::new(vflip), Box::new(maflip), Box::new(miflip) ],
        ]
    } else {
        vec![ vec![] ]
    };

    let mut solutions = Bool::from_bool(&context, true);
    for fs in flips {
        let flipped: Vec<Bool<'_>> = flip(&queens, &fs).iter().map(|(r, c)| board[*r][*c].clone()).collect();
        let flipped: Vec<&Bool<'_>> = flipped.iter().collect(); 
        let flipped = Bool::and(&context, flipped.as_slice()).not();
        solutions = Bool::and(&context, &[&solutions, &flipped]);
    }

    solutions
}

fn main() {
    let args = Args::parse();
    let n = args.count;
    let unique = args.unique;

    let mut config = Config::new();
    config.set_proof_generation(true);
    let context = Context::new(&config);

    let solver = Solver::new(&context);
    solver.push();

    let board = encode_board(n, &context);
    let row = encode_row(&board, &context);
    let column = encode_column(&board, &context);
    let diagonal = encode_diagonal(&board, &context);

    let problem = Bool::and(&context, &[&row, &column, &diagonal]);
    solver.assert(&problem);

    let mut count: usize = 0;
    loop {
        match solver.check() {
            SatResult::Sat => {
                let solution = solver.get_model().unwrap();
                let solution = decode_board(unique, &board, &solution, &context);
                solver.assert(&solution);
                count += 1;
            }
            _ => break 
        }
    }
    println!("Total number of solutions: {}.", count);
}
