use clap::{App, ArgMatches};
use configuration::Configuration;
use import::import;
use library::Library;

pub fn process_args(conf: &Configuration, lib: &mut Library) {
    let app_yaml = load_yaml!("cli_en.yml");
    let matches = App::from_yaml(app_yaml).get_matches();

    match matches.subcommand() {
        ("import", Some(sub)) => sub_import(sub, lib, conf),
        _ => (),
    }
}

fn sub_import(sub: &ArgMatches, lib: &mut Library, conf: &Configuration) {
    let file = sub.value_of("file").unwrap();
    let bibliography = sub.value_of("bibliography").unwrap();
    let id = sub.value_of("entry");
    let force_move = sub.is_present("move");
    let force_copy = sub.is_present("copy");
    let tags: Vec<String> = sub
        .values_of("tag")
        .map_or_else(|| vec![], |t| t.map(String::from).collect());

    match import(file, bibliography, id, force_move, force_copy, tags, conf) {
        Ok(entry) => {
            let paths: String = (&entry)
                .file_paths()
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    if i == 0 {
                        p.clone()
                    } else {
                        format!("; {}", p)
                    }
                })
                .collect();
            println!("Successfully imported file to {}.", &paths);
            lib.add_entry(entry);
        }
        Err(err) => {
            eprintln!("Failed to import file: {}.", err);
        }
    }
}
