
use clap::{Arg, App, ArgMatches, SubCommand};

pub fn parse_args<'a>() -> ArgMatches<'a> {
    let matches = App::new("stest (speedtest cli)")
        .version("0.3.0")
        .author("opensourcegeek. <3.pravin@gmail.com>")
        .about("A command line utility to run speedtest similar to http://speedtest.net")
        .arg(Arg::with_name("number_tests")
            .short("n")
            .long("number-tests")
            .value_name("number_tests")
            .help("Sets number of tests to run")
            .takes_value(true))
        .arg(Arg::with_name("csv")
            .short("c")
            .long("csv")
            .value_name("csv")
            .help("Set name of csv file")
            .takes_value(true))
        .arg(Arg::with_name("server_country")
            .short("sc")
            .long("server-country")
            .value_name("server_country")
            .help("This will scan servers only from given country - it might take a while before it finds the best server")
            .takes_value(true))
        .subcommand(SubCommand::with_name("server")
                .about("Available test servers can be searched for")
                .arg(Arg::with_name("list")
                    .short("l")
                    .long("list")
                    .value_name("list")
                    .help("prints all servers")))
        .get_matches();
    matches
}
