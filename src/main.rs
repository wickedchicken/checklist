#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use std::collections::BTreeMap;
use std::error::Error;
use std::fs::File;
use std::path::Path;

use dialoguer::Confirmation;

#[derive(Debug, Serialize, Deserialize)]
struct CheckListList(BTreeMap<String, CheckList>);

#[derive(Debug, Serialize, Deserialize)]
struct CheckList(Vec<String>);

fn read_checklists(path: &Path) -> CheckListList {
    let display = path.display();
    let file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, why.description()),
        Ok(file) => file,
    };
    match serde_yaml::from_reader::<_, CheckListList>(file) {
        Err(why) => panic!("couldn't read {}: {}", display, why.description()),
        Ok(s) => s,
    }
}

fn ask_question(prompt: &str) -> bool {
    Confirmation::new()
        .with_text(&prompt)
        .interact()
        .expect("Could not prompt, bailing!")
}

fn ask_formatted_question(prefix: &str, prompt: &str) -> bool {
    ask_question(&format!("{}{}?", prefix, prompt))
}

fn main() {
    let path = Path::new(".checklist.yml");
    let checklists = read_checklists(&path);
    if let Some(checklist) = checklists.0.get("committing") {
        for item in &checklist.0 {
            if ask_formatted_question(&"Have you: ", &item) {
                println!("Great! Continuing...")
            } else {
                println!("nevermind then :(");
            }
        }
    }
}
