#[macro_use]
extern crate serde_derive;

extern crate serde_yaml;

use std::error::Error;
use std::fs::File;
use std::path::Path;

use std::collections::BTreeMap;

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
        Ok(s) => return s,
    }
}

fn main() {
    let path = Path::new(".checklist.yml");
    let checklists = read_checklists(&path);
    match checklists.0.get("committing") {
        Some(checklist) => print!("{} contains:\n{:#?}", path.display(), checklist),
        None => (),
    }
}
