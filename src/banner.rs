use std::{
    io::{self, Write},
    thread,
    time::Duration,
};

const BOLE_BANNER: &str = r#"
           .-.
          /   \
         /  .  \
        /__/ \__\
          || ||
        __||_||__
       /  BOLE   \
      /___________\
         \  |  /
          \ | /
           \|/
            V
    "#;

/// Stream banner with Matrix-style reveal effect
pub fn stream_banner() {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let matrix_chars = ['█', '▓', '▒', '░', '▀', '▄', '■'];

    let chars: Vec<char> = BOLE_BANNER.chars().collect();

    // Show matrix characters
    for (i, &ch) in chars.iter().enumerate() {
        if ch == '\n' {
            let _ = handle.write_all(b"\n");
        } else if ch == ' ' {
            let _ = handle.write_all(b" ");
        } else {
            let matrix_char = matrix_chars[i % matrix_chars.len()];
            let _ = handle.write_all(matrix_char.to_string().as_bytes());
        }
        let _ = handle.flush();

        if ch != '\n' && ch != ' ' {
            thread::sleep(Duration::from_millis(5));
        }
    }

    thread::sleep(Duration::from_millis(60));

    // Clear and show real banner
    print!("\r");
    for _ in 0..BOLE_BANNER.lines().count().saturating_sub(1) {
        print!("\x1b[A"); // Move cursor up
    }
    let _ = handle.flush();

    // Reveal actual characters
    for ch in BOLE_BANNER.chars() {
        let _ = handle.write_all(ch.to_string().as_bytes());
        let _ = handle.flush();

        if ch != '\n' && ch != ' ' {
            thread::sleep(Duration::from_millis(10));
        }
    }
}
