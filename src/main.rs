use checklist;
use structopt::StructOpt;

use std::process;

fn main() {
    let opts = checklist::Opt::from_args();

    if let Err(e) = checklist::run(&opts) {
        println!("Application error: {}", e);

        process::exit(1);
    }
}
