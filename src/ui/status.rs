const CHECK_MARK: &str = "✓";
const CROSS_MARK: &str = "✗";
const ARROW: &str = "→";

pub fn println_failure(message: &str, level: usize) {
    let indent = "  ".repeat(level);
    println!("{indent}{CROSS_MARK} {message}");
}

pub fn println_step(message: &str, level: usize) {
    let indent = "  ".repeat(level);
    println!("{indent}{ARROW} {message}");
}

pub fn println_step_concat(message: &str, level: usize) {
    let indent = "  ".repeat(level);
    println!("{indent}  {message}");
}

pub fn print_step_concat(message: &str, level: usize) {
    let indent = "  ".repeat(level);
    print!("{indent}  {message}");
}

pub fn println_success(message: &str, level: usize) {
    let indent = "  ".repeat(level);
    println!("{indent}{CHECK_MARK} {message}");
}
