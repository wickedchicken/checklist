use checklist;

use console::style;
use structopt::StructOpt;

use std::process;

fn main() {
    let opts = checklist::Opt::from_args();

    match checklist::run(&opts) {
        Err(e) => {
            println!("{}: {}", style("Application error").red(), e);
            process::exit(1);
        }
        Ok(status) => {
            process::exit(status);
        }
    }
}
