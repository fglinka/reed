use clap::{App, ArgMatches};
use configuration::Configuration;
use import::import;

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
        )
    )
}

pub fn process_args(conf: &Configuration) {
    let matches = parse().get_matches();

    match matches.subcommand() {
        ("import", Some(sub)) => sub_import(sub, conf),
        _ => (),
    }
}

fn sub_import(sub: &ArgMatches, conf: &Configuration) {
    let file = sub.value_of("file").unwrap();
    let bibliography = sub.value_of("bibliography").unwrap();
    let entry = sub.value_of("entry");
    let force_move = sub.is_present("move");
    let force_copy = sub.is_present("copy");

    import(file, bibliography, entry, force_move, force_copy, conf).unwrap();
}
