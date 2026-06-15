mod format_tokens;
mod input;
mod segments;
mod util;

use std::io::Read;

fn main() {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf).unwrap_or(0);

    let input = match input::parse_input(&buf) {
        Some(i) => i,
        None => return,
    };

    let output = format!(
        "{}{}{}{}{} 🦀",
        segments::token_segment(&input),
        segments::directory_segment(&input),
        segments::git_branch_segment(),
        segments::venv_segment(),
        segments::model_segment(&input),
    );

    print!("{}", output);
}
