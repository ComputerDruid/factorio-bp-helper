use std::io::stderr;

use crossterm::event::{DisableBracketedPaste, EnableBracketedPaste, KeyCode, KeyModifiers};

pub fn prompt_blueprint() -> String {
    crossterm::execute!(stderr(), EnableBracketedPaste).unwrap();
    crossterm::terminal::enable_raw_mode().unwrap();
    eprint!("Paste blueprint:\r\n");
    const PROMPT: &str = "🟦❯ ";
    eprint!("{}", PROMPT);
    let mut warned = false;
    let blueprint_string = loop {
        let event = crossterm::event::read().unwrap();
        match event {
            crossterm::event::Event::Key(key_event) => {
                if key_event.code == KeyCode::Char('c')
                    && key_event.modifiers == KeyModifiers::CONTROL
                {
                    eprint!("\r\n");
                    break None;
                }
                if !warned {
                    warned = true;
                    eprint!("\rWARN: Only pasting works.\r\n");
                    eprint!("{}", PROMPT);
                }
            }
            crossterm::event::Event::Paste(s) => {
                eprint!("received.\r\n");
                break Some(s);
            }
            _ => {}
        }
    };
    crossterm::execute!(stderr(), DisableBracketedPaste).unwrap();
    crossterm::terminal::disable_raw_mode().unwrap();
    blueprint_string.unwrap_or_else(|| std::process::exit(1))
}
