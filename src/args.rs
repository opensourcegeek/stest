
use clap::{Arg, App, ArgMatches, ArgGroup};
//use clap::*;


pub fn parse_args<'a>() -> ArgMatches<'a> {
    App::new("stest (speedtest cli)")
        .version(crate_version!())
        .author(crate_authors!())
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
        .arg(Arg::with_name("use_cached")
            .short("u")
            .long("use-cached")
            .help("Use a cached copy of servers"))
        .args_from_usage(
            "-s --server-country [server_country] 'This will scan servers only from given country name - it might take a while before it finds the best server'
             -o --server-country-code [server_country_code]  'This will scan servers only from given country code - it might take a while before it finds the best server'")
        .group(ArgGroup::with_name("server-filter")
          .args(&["server-country", "server-country-code"]))
        //        .subcommand(SubCommand::with_name("server")
        //                .about("Available test servers can be searched for")
        //                .arg(Arg::with_name("list")
        //                    .short("l")
        //                    .long("list")
        //                    .value_name("list")
        //                    .help("prints all servers")))
        .get_matches()
}

