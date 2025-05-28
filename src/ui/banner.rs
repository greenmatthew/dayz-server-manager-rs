use crate::VERSION;

pub fn print_banner() {
    let banner = include_str!("../../banner.ascii");
    let term_width = term_size::dimensions().map(|(w, _)| w).unwrap_or(80);

    println!(); // Padding before banner

    for line in banner.lines() {
        let line_len = line.chars().count();
        let padding = if term_width > line_len {
            (term_width - line_len) / 2
        } else {
            0
        };
        println!("{}{}", " ".repeat(padding), line);
    }

    println!(); // Margin between banner and title

    // Center the title/version
    let title = format!("DZSM v{VERSION} - DayZ Server Manager");
    let title_len = title.chars().count();
    let padding = if term_width > title_len {
        (term_width - title_len) / 2
    } else {
        0
    };
    println!("{}{}", " ".repeat(padding), title);

    println!(); // Padding after banner
}
