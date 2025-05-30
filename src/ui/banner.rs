use crate::{VERSION, AUTHORS};

pub fn print_banner() {
    let banner = include_str!("../../banner.ascii");
    let term_width = term_size::dimensions().map_or(80, |(w, _)| w);

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

    // Parse and display authors
    let authors_vec: Vec<&str> = AUTHORS.split(':').map(str::trim).collect();
    let authors_text = if authors_vec.len() == 1 {
        format!("Author: {}", authors_vec[0])
    } else {
        format!("Authors: {}", authors_vec.join(", "))
    };
    
    let authors_len = authors_text.chars().count();
    let authors_padding = if term_width > authors_len {
        (term_width - authors_len) / 2
    } else {
        0
    };
    println!("{}{}", " ".repeat(authors_padding), authors_text);

    println!(); // Padding after banner
}
