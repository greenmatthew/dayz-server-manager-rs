use super::status::print_step_concat;

use anyhow::{Result};
use std::io::{self, Write};

pub fn prompt_yes_no(prompt: &str, default: bool, level: usize) -> Result<bool> {
    let options = if default { "(Y/n)" } else { "(y/N)" };
    
    println!();
    print_step_concat(&format!("{prompt} {options}: "), level);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();

    Ok(match input.as_str() {
        "" => default,
        "y" | "yes" => true,
        "n" | "no" => false,
        _ => {
            println!("Please enter 'y' or 'n'");
            return prompt_yes_no(prompt, default, level);
        }
    })
}
