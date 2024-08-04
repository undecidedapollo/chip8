use std::io::{Read, Write};

use chip8_assembler::{
    lexer::{Lexer, Token},
    parser::{ParseResult, Parser},
    resolver::Resolver,
};

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    let input_file = args.get(1).expect("No input file provided");
    let output_file = args.get(2).expect("No output file provided");
    let rest = args.get(3..).unwrap_or(&[]);
    let lex_out = rest.contains(&"-l".to_owned());
    let parser_out = rest.contains(&"-p".to_owned());

    println!("Input file: {}", input_file);
    println!("Output file: {}", output_file);

    let file = std::fs::File::open(input_file)
        .expect(format!("Could not open file {}", input_file).as_str());
    let buffered = std::io::BufReader::new(file);
    let chars = buffered.bytes().filter_map(|b| b.ok()).map(|b| b as char);
    let lexer: Box<dyn Iterator<Item = Token>> = if lex_out {
        Box::new(Lexer::from_iter(chars).map(|token| {
            match token {
                Token::Whitespace('\n') => {
                    println!("{:?}", token);
                }
                _ => {
                    print!("L: {:?}", token);
                }
            }
            token
        }))
    } else {
        Box::new(Lexer::from_iter(chars))
    };
    let parser: Box<dyn Iterator<Item = ParseResult>> = if parser_out {
        Box::new(Parser::from_iter(lexer).map(|parse_result| {
            println!("P: {:?}", parse_result);
            parse_result
        }))
    } else {
        Box::new(Parser::from_iter(lexer))
    };
    let mut resolver = Resolver::from_iter(parser);
    let output = resolver.resolve();
    // let output_bin = parser
    //     .filter_map(|token| match token {
    //         ParseResult::Statement(statement) => {
    //             let opcode = OpCode::try_from(statement.clone());

    //             let op = match opcode {
    //                 Ok(f) => f,
    //                 Err(e) => {
    //                     println!("{:?}", e);
    //                     return None;
    //                 }
    //             };

    //             let res = <(u8, u8)>::from(op);
    //             println!("{:?} {:?} {:?}", statement, opcode, res);

    //             return Some(res);
    //         }
    //         _ => {
    //             println!("{:?}", token);
    //             None
    //         }
    //     })
    //     .flat_map(|f| [f.0, f.1])
    //     .collect::<Vec<u8>>();

    let output_bin = match output {
        Ok(f) => f,
        Err(e) => {
            println!("Assembly error: {:?}", e);
            return;
        }
    };

    let mut file = std::fs::File::create(output_file).expect("Could not create file");
    file.write_all(&output_bin)
        .expect("Could not write to file");
}

// Lex
// Parse
