use clap::{App, ArgMatches};
use configuration::Configuration;
use import::import;
use library::Library;

fn parse() -> App<'static, 'static> {
    clap_app!(fowlder =>
        (version: crate_version!())
        (author: crate_authors!())
        (about: "An application for organizing, searching and viewing academic publications")
        (@subcommand import =>
            (about: "Import an additional document into the database")
            (@arg file: +required "Specify the file to import")
            (@arg bibliography: +required "Specify a bibliography used to obtain metadata \
                about the file")
            (@arg entry: -e --entry +takes_value "Specify which bibliography entry to use \
                if there are multiple")
            (@arg move: -m --move "Move the imported file regardless of the confifuration")
            (@arg copy: -c --copy conflicts_with[move] "Copy the imported file regardless of \
                the configuration")
            (@arg tag: -t --tag ... +takes_value "Specify tags used to categorize papers.")
        )
    )
}

pub fn process_args(conf: &Configuration, lib: &mut Library) {
    let matches = parse().get_matches();

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
