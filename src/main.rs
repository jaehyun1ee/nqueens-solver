use z3::{ast::Bool, Config, Context, Model, SatResult, Solver};

const N: usize = 8;

fn encode_row<'c>(board: &Vec<Vec<Bool<'c>>>, context: &'c Context) -> Bool<'c> {
    // Definedness
    let mut define = Vec::new();
    for r in 0..N {
        let mut row = Vec::new();
        for c in 0..N {
            row.push(board[r][c].clone());
        }
        let row: Vec<&Bool<'_>> = row.iter().collect();
        define.push(Bool::or(&context, row.as_slice()));
    }
    let define: Vec<&Bool<'_>> = define.iter().collect();

    // Uniqueness
    let mut unique = Vec::new();
    for r in 0..N {
        for i in 0..N {
            for j in (i + 1)..N {
                unique.push(Bool::and(&context, &[&board[r][i], &board[r][j]]).not())
            }
        }
    }
    let unique: Vec<&Bool<'_>> = unique.iter().collect(); 

    Bool::and(&context, &[&define[..], &unique[..]].concat())
}

fn encode_column<'c>(board: &Vec<Vec<Bool<'c>>>, context: &'c Context) -> Bool<'c> {
    // Definedness
    let mut define = Vec::new();
    for c in 0..N {
        let mut row = Vec::new();
        for r in 0..N {
            row.push(board[r][c].clone());
        }
        let row: Vec<&Bool<'_>> = row.iter().collect();
        define.push(Bool::or(&context, row.as_slice()));
    }
    let define: Vec<&Bool<'_>> = define.iter().collect();

    // Uniqueness
    let mut unique = Vec::new();
    for c in 0..N {
        for i in 0..N {
            for j in (i + 1)..N {
                unique.push(Bool::and(&context, &[&board[i][c], &board[j][c]]).not());
            }
        }
    }
    let unique: Vec<&Bool<'_>> = unique.iter().collect(); 

    Bool::and(&context, &[&define[..], &unique[..]].concat())
}

fn encode_diagonal<'c>(board: &Vec<Vec<Bool<'c>>>, context: &'c Context) -> Bool<'c> {
    let mut define = Vec::new();

    // Definedness for major diagonal
    for d in 0..N {
        for i in 0..(N - d) {
            for j in (i + 1)..(N - d) {
                define.push(Bool::and(&context, &[&board[i + d][i], &board[j + d][j]]).not());
            }
        }
    }
    for d in 1..N {
        for i in d..N {
            for j in (i + 1)..N {
                define.push(Bool::and(&context, &[&board[i - d][i], &board[j - d][j]]).not());
            }
        }
    }

    // Definedness for minor diagonal
    for s in 0..N {
        for i in 0..=s {
            for j in (i + 1)..=s {
                define.push(Bool::and(&context, &[&board[i][s - i], &board[j][s - j]]).not())
            }
        }
    }
    for s in N..(2 * N - 1) {
        for i in (s - N + 1)..N {
            for j in (i + 1)..N {
                define.push(Bool::and(&context, &[&board[i][s - i], &board[j][s - j]]).not())
            }
        }
    }

    let define: Vec<&Bool<'_>> = define.iter().collect();
    Bool::and(&context, define.as_slice())
}

fn encode_board<'c>(context: &'c Context) -> Vec<Vec<Bool<'c>>> {
    let mut board = Vec::new();
    for r in 0..N {
        let mut row = Vec::new();
        for c in 0..N {
            let cell = Bool::new_const(&context, format!("({},{})", r, c)); 
            row.push(cell);
        }
        board.push(row);
    }

    board
}

fn decode_board<'c>(board: &Vec<Vec<Bool<'c>>>, solution: &Model<'c>, context: &'c Context) -> Bool<'c> {
    let mut queens = Vec::new();
    for r in 0..N {
        for c in 0..N {
            let cell = &board[r][c];
            let cell = Bool::as_bool(&solution.eval(cell, true).unwrap()).unwrap();

            if cell {
                queens.push(board[r][c].clone());
            }
        }
    }
    let queens: Vec<&Bool<'_>> = queens.iter().collect();

    Bool::and(&context, queens.as_slice()).not()
}

fn main() {
    let mut config = Config::new();
    config.set_proof_generation(true);
    let context = Context::new(&config);

    let solver = Solver::new(&context);
    solver.push();

    let board = encode_board(&context);
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
                let solution = decode_board(&board, &solution, &context);
                solver.assert(&solution);
                count += 1;
            }
            _ => break 
        }
    }
    println!("Total number of solutions: {}.", count);
}
