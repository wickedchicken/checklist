#[macro_use]
extern crate structopt;
#[cfg(test)]
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate tempfile;

use std::collections::BTreeMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use console::Style;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirmation;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
struct CheckListList(BTreeMap<String, CheckList>);

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
struct CheckList(Vec<String>);

impl CheckListList {
    fn from_file(path: &Path) -> Result<CheckListList, &'static str> {
        let file = match File::open(&path) {
            Err(_) => return Err("couldn't open file"),
            Ok(file) => file,
        };
        CheckListList::from_reader(file)
    }

    fn from_reader<R: Read>(input: R) -> Result<CheckListList, &'static str> {
        match serde_yaml::from_reader::<_, CheckListList>(input) {
            Err(_) => Err("couldn't parse yaml"),
            Ok(s) => Ok(s),
        }
    }
}

fn ask_question(prompt: &str) -> Result<bool, &'static str> {
    let mut theme = ColorfulTheme::default();
    theme.no_style = Style::new().red();
    match Confirmation::with_theme(&theme)
        .with_text(&prompt)
        .interact()
    {
        Err(_) => Err("error occured while prompting"),
        Ok(answer) => Ok(answer),
    }
}

fn ask_formatted_question(prefix: &str, prompt: &str) -> Result<bool, &'static str> {
    ask_question(&format!("{}{}?", prefix, prompt))
}

fn question_loop(checklist: &CheckList) -> Result<bool, Box<dyn Error>> {
    let mut seen = false;
    for item in &checklist.0 {
        if !seen {
            seen = true;
        } else {
            println!("Great! Continuing...")
        }
        if !ask_formatted_question(&"Have you: ", &item)? {
            return Ok(false);
        }
    }

    Ok(true)
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "checklist",
    about = "Run through a checklist",
    raw(setting = "structopt::clap::AppSettings::ColoredHelp")
)]
pub struct Opt {
    #[structopt(
        parse(from_os_str),
        default_value = ".checklist.yml",
        long = "checklist",
        help = "location of the checklist YAML"
    )]
    checklist: PathBuf,
}

pub fn run(opts: &Opt) -> Result<(), Box<dyn Error>> {
    let success = Style::new().green();
    let failure = Style::new().red();
    let checklists = CheckListList::from_file(&opts.checklist)?;
    if let Some(checklist) = checklists.0.get("committing") {
        if question_loop(&checklist)? {
            println!("{}", success.apply_to("all clear!"));
        } else {
            println!(
                "{} please fix and start again",
                failure.apply_to("aborting")
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Seek, SeekFrom, Write};

    fn write_to_tempfile(contents: &str) -> File {
        // Write
        let mut tmpfile: File = tempfile::tempfile().unwrap();
        tmpfile.write_all(contents.as_bytes()).unwrap();

        // Seek to start
        tmpfile.seek(SeekFrom::Start(0)).unwrap();

        tmpfile
    }

    #[test]
    fn test_correct_yaml() {
        let tempfile = write_to_tempfile("committing:\n- test");
        assert_eq!(
            CheckListList::from_reader(tempfile),
            Ok(CheckListList(btreemap! {
                String::from("committing") => CheckList(vec![String::from("test")]),
            })),
        )
    }

    #[test]
    fn test_incorrect_yaml() {
        let tempfile = write_to_tempfile("beep beep");
        assert_eq!(
            CheckListList::from_reader(tempfile),
            Err("couldn't parse yaml")
        )
    }
}
