use std::io::{self, Write};

pub fn read_line(question: String) -> String {
    print!("{}", question);
    io::stdout().flush().unwrap();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();

    choice.trim().to_string()
}
