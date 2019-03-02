use checklist;

use std::path::Path;
use std::process;

fn main() {
    let path = Path::new(".checklist.yml");

    if let Err(e) = checklist::run(&path) {
        println!("Application error: {}", e);

        process::exit(1);
    }
}
