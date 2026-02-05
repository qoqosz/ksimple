use ksimple::{Runtime, run_batch, run_repl};
const BANNER: &str = "k/simple in Rust";

fn main() {
    let mut runtime = Runtime::new();
    let args: Vec<String> = std::env::args().collect();

    match args.as_slice() {
        [_] => {
            println!("{}", BANNER);
            run_repl(&mut runtime);
        }
        [_, file_path] => run_batch(&mut runtime, file_path),
        _ => {
            eprintln!("Usage: ksimple [FILE]");
            std::process::exit(1);
        }
    }
}
