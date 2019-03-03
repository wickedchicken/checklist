use checklist;

use console::style;
use structopt::StructOpt;

use std::process;

fn main() {
    let opts = checklist::Opt::from_args();

    if let Err(e) = checklist::run(&opts) {
        println!("{}: {}", style("Application error").red(), e);

        process::exit(1);
    }
}
