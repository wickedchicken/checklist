extern crate duct_sh;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate structopt;
#[cfg(test)]
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use console::Style;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Confirmation;
use duct_sh::sh_dangerous;
use failure::Error;
use indicatif::{ProgressBar, ProgressStyle};
use structopt::clap::AppSettings;

// Increment the version number every time the version changes. I can't figure out how to
// break this out into its own const, see https://github.com/rust-lang/rust/issues/52393.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "schema_version")]
enum VersionedCheckListList {
    #[serde(rename = "3")] // increment here
    Current(CheckListList),
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
struct CheckListList(BTreeMap<String, CheckList>);

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
struct CheckList {
    #[serde(default)]
    environment: BTreeMap<String, String>,
    #[serde(default)]
    automated: Vec<String>,
    #[serde(default)]
    manual: Vec<String>,
}

impl CheckListList {
    /// Creates a CheckListList from a file
    fn from_file(path: &Path) -> Result<CheckListList, Error> {
        let file = match File::open(&path) {
            Err(e) => bail!("couldn't open file {}: {}", path.display(), e),
            Ok(file) => file,
        };
        match CheckListList::from_reader(file) {
            Err(e) => bail!("couldn't open file {}: {}", path.display(), e),
            Ok(s) => Ok(s),
        }
    }

    /// Creates a CheckListList from a Reader
    fn from_reader<R: Read>(input: R) -> Result<CheckListList, Error> {
        match serde_yaml::from_reader::<_, VersionedCheckListList>(input) {
            Err(e) => bail!("couldn't parse yaml: {}", e),
            Ok(s) => match s {
                VersionedCheckListList::Current(s) => Ok(s),
            },
        }
    }
}

/// Asks a yes/no question to the user, returning the response
fn ask_question(prompt: &str) -> Result<bool, Error> {
    let mut theme = ColorfulTheme::default();
    theme.no_style = Style::new().red();
    Ok(Confirmation::with_theme(&theme)
        .with_text(&prompt)
        .interact()?)
}

/// Asks a formatted yes/no question to the user, returning the response
fn ask_formatted_question(prefix: &str, prompt: &str) -> Result<bool, Error> {
    ask_question(&format!("{}{}?", prefix, prompt))
}

/// Prompt the user with a list of yes/no questions, stopping if any are false
fn question_loop(checklist: &CheckList) -> Result<i32, Error> {
    let success = Style::new().green();
    let failure = Style::new().red();
    let mut seen = false;
    for item in &checklist.manual {
        if !seen {
            seen = true;
        } else {
            println!("Great! Continuing...")
        }
        if !ask_formatted_question(&"Have you: ", &item)? {
            println!("\nmanual tests: {}", failure.apply_to("failed"));
            return Ok(1);
        }
    }

    println!("\nmanual tests: {}", success.apply_to("passed"));
    Ok(0)
}

/// Run a list of commands, stopping if any fail
fn shell_loop(checklist: &CheckList) -> Result<i32, Error> {
    let success = Style::new().green();
    let failure = Style::new().red();
    let sty = ProgressStyle::default_bar()
        .template("{bar:40.green/white} {pos:>2}/{len:7} {wide_msg:.blue}");
    let b = ProgressBar::new(checklist.automated.len() as u64);
    b.set_style(sty);
    let progress_bar = scopeguard::guard(b, |b| {
        b.finish_and_clear();
    });
    for item in &checklist.automated {
        progress_bar.set_message(&item);
        let mut command = sh_dangerous(item)
            .stdout_capture()
            .stderr_capture();
        for (key, value) in checklist.environment.iter() {
            command = command.env(key, value);
        }
        let command_res = command.unchecked().run()?;
        if !command_res.status.success() {
            progress_bar.finish_and_clear();
            println!("\nautomated tests: {}", failure.apply_to("failed"));
            println!("{} running: {}\n", failure.apply_to("error"), item);
            io::stdout().write_all(&command_res.stdout)?;
            io::stderr().write_all(&command_res.stderr)?;
            return Ok(command_res.status.code().unwrap());
        }
        progress_bar.inc(1);
    }

    progress_bar.finish_and_clear();
    println!("\nautomated tests: {}", success.apply_to("passed"));
    Ok(0)
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "checklist",
    about = "Run through a checklist",
    global_settings(&[AppSettings::ColoredHelp]),
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

/// Run automated tests and ask if manual tests have been done
pub fn run(opts: &Opt) -> Result<i32, Error> {
    let success = Style::new().green();
    let failure = Style::new().red();
    let checklists = CheckListList::from_file(&opts.checklist)?;
    if let Some(checklist) = checklists.0.get("committing") {
        if shell_loop(&checklist)? == 0 && question_loop(&checklist)? == 0 {
            println!("{}", success.apply_to("all clear!"));
        } else {
            println!(
                "{} please fix and start again",
                failure.apply_to("aborting")
            );
            return Ok(1);
        }
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::prelude::*;

    #[test]
    fn test_correct_yaml() {
        let t = assert_fs::TempDir::new().unwrap();
        let temp = scopeguard::guard(t, |t| {
            t.close().unwrap();
        });
        temp.child(".checklist.yml")
            .write_str(
                "schema_version: 3\ncommitting:\n  environment: {}\n  automated: []\n  manual:\n    - test",
            )
            .unwrap();
        assert_eq!(
            CheckListList::from_file(temp.child(".checklist.yml").path()).unwrap(),
            CheckListList(btreemap! {
                String::from("committing") => CheckList{
                    environment: Default::default(),
                    manual: vec![String::from("test")],
                    automated: vec![],
                },
            }),
        );
    }

    #[test]
    fn test_defaults() {
        let t = assert_fs::TempDir::new().unwrap();
        let temp = scopeguard::guard(t, |t| {
            t.close().unwrap();
        });
        temp.child(".checklist.yml")
            .write_str("schema_version: 3\ncommitting:\n  manual: []")
            .unwrap();
        assert_eq!(
            CheckListList::from_file(temp.child(".checklist.yml").path()).unwrap(),
            CheckListList(btreemap! {
                String::from("committing") => CheckList{
                    environment: Default::default(),
                    manual: vec![],
                    automated: vec![],
                },
            }),
        );
    }

    #[test]
    fn test_incorrect_yaml() {
        let t = assert_fs::TempDir::new().unwrap();
        let temp = scopeguard::guard(t, |t| {
            t.close().unwrap();
        });
        temp.child(".checklist.yml").write_str("beep beep").unwrap();
        assert!(CheckListList::from_file(temp.child(".checklist.yml").path()).is_err())
    }

    #[test]
    fn test_incorrect_schema_version() {
        let t = assert_fs::TempDir::new().unwrap();
        let temp = scopeguard::guard(t, |t| {
            t.close().unwrap();
        });
        temp.child(".checklist.yml")
            .write_str("schema_version: bananas\ncommitting:\n- test")
            .unwrap();
        assert!(CheckListList::from_file(temp.child(".checklist.yml").path()).is_err())
    }

    #[test]
    fn test_invalid_filename() {
        let t = assert_fs::TempDir::new().unwrap();
        let temp = scopeguard::guard(t, |t| {
            t.close().unwrap();
        });
        assert!(CheckListList::from_file(temp.child("does_not_exist").path()).is_err())
    }

    #[test]
    fn test_return_code() {
        let c = CheckList {
            environment: Default::default(),
            automated: vec!["true".to_string()],
            manual: vec![],
        };
        assert_eq!(shell_loop(&c).unwrap(), 0);
    }

    #[test]
    fn test_return_code_fail() {
        let c = CheckList {
            environment: Default::default(),
            automated: vec!["false".to_string()],
            manual: vec![],
        };
        assert_eq!(shell_loop(&c).unwrap(), 1);
    }
}
