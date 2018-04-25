extern crate csv;
#[macro_use]
extern crate clap;
extern crate stest_lib;

mod args;
use std::{thread, time};

use stest_lib::config;
use stest_lib::config::{TestServerConfig, find_ignore_ids};
use stest_lib::file_utils;
use stest_lib::time_utils::get_current_time_as_string;
use stest_lib::{find_best_server_by_ping, perform_download_test, perform_upload_test, pick_closest_servers, parse_url};

const MAX_NUM_RETRIES: u64      = 10;
const ONE_SEC_IN_MILLIS: u64    = 1000;

const CSV_COLUMN_NAMES: &'static str = "test_number,server_url,rx_start,rx_total_bytes,rx_total_millis,rx_speed_mbps,rx_end,tx_start,tx_total_bytes,tx_total_millis,tx_speed_mbps,tx_end";


fn run_test(number_of_tests: u64, file_name: Option<&str>,
            server_country: Option<&str>,
            server_country_code: Option<&str>,
            use_cached_servers: bool) {
    // The speed test config file request returns nothing sometimes, but it looks like a
    // glitch on the server side as similar content-length:0 responses come back when queried
    // using curl as well. To work around it we retry upto MAX_NUM_RETRIES, it should come in
    // via passed in args as well.
    let mut current_count = 0;
    let mut config = config::FullConfig::new();

    while !config.parsing_succeeded && current_count <= MAX_NUM_RETRIES {
        println!("Retrying...");
        config = config::FullConfig::new();
        thread::sleep(time::Duration::from_millis(ONE_SEC_IN_MILLIS));
        current_count += 1;
    }

//    println!("{:?}", config);
    // TODO: Add a check to exit if we cannot retrieve any config

    let mut test_servers: Vec<TestServerConfig> = config::get_all_test_servers(use_cached_servers);
    println!("Total servers available: {:?}", test_servers.len());

    let server_hint_config = config.server;

    // Use find_ignore_ids from config mod
    let ignore_ids = find_ignore_ids(server_hint_config.ignoreids);
//    println!("Ignored ids: {:?}", ignore_ids);

    // ignore servers on ignore list
    // TODO: Pass in argument to switch off ignore servers recommended by speedtest config?
    test_servers.retain(|ref mut server| {
        // If not ignore ids list keep this server
        !ignore_ids.contains(&server.id)
    });

    println!("Total servers available after ignoring: {:?}", test_servers.len());
    println!("");

    let closest_servers: Vec<TestServerConfig> = match server_country {
        Some(sc)    => {
            test_servers.retain(|ref mut server| {
                // If not ignore ids list keep this server
                server.country.to_lowercase() == sc.to_string().to_lowercase()
            });

            if test_servers.len() > 10 {
                println!("Number of servers in {} are {} - it might take a while to find best server", sc, test_servers.len());
            } else {
                println!("Number of servers in {} are {}", sc, test_servers.len());
            }

            test_servers
        },
        None        => {
            match server_country_code {
                Some(scc) => {
                    test_servers.retain(|ref mut server| {
                        // If not ignore ids list keep this server
                        server.country_code.to_lowercase() == scc.to_string().to_lowercase()
                    });

                    if test_servers.len() > 10 {
                        println!("Number of servers in {} are {} - it might take a while to find best server", scc, test_servers.len());
                    } else {
                        println!("Number of servers in {} are {}", scc, test_servers.len());
                    }

                    test_servers
                },
                None => {
                    // both server country and server country code are not set.
                    // look for closest servers - we should add a switch to avoid this distance check
                    let client_conf = &config.client;
                    let client_location = (client_conf.lat, client_conf.lon);
                    let mut closest_servers: Vec<TestServerConfig> = Vec::new();
                    pick_closest_servers(client_location, &test_servers, &mut closest_servers);
                    closest_servers
                }
            }
        }
    };


    println!("Your address {:?} and ISP {:?}", config.client.ip, config.client.isp);
    if closest_servers.len() > 0 {
        // TODO: May be change the server for each test?
        // look for ping latency for all servers (or closest servers)
        let (best_server, _) = find_best_server_by_ping(&closest_servers);
        let mut records = Vec::new();

        let mut col_names = Vec::new();
        for name in CSV_COLUMN_NAMES.split(',') {
            col_names.push(name.to_string());
        }

        let sizes: Vec<u64> = vec![32768, 65536, 131072, 262144, 524288, 1048576, 7340032];
        let dimensions: Vec<u64> = vec![350, 500, 750, 1000, 1500, 2000, 2500, 3000];

        records.push(col_names);

        for i in 0..number_of_tests {
            let current_test = i + 1;
            println!("Performing test {}", current_test);
            let mut record = Vec::new();
            let server_url = parse_url(&best_server.url);
            record.push(current_test.to_string());
            record.push(server_url.to_owned());

            // Start tests against chosen server - these download/upload tests will
            // run in separate threads
            print!("Running download tests...");
            record.push(get_current_time_as_string());
            let (rx_total_bytes, rx_total_millis, rx_speed_in_mbps) = perform_download_test(&server_url, &dimensions);
            record.push(rx_total_bytes.to_string());
            record.push(rx_total_millis.to_string());
            record.push(rx_speed_in_mbps.to_string());
            record.push(get_current_time_as_string());
            println!("");

            print!("Running upload tests...");
            record.push(get_current_time_as_string());
            let (tx_total_bytes, tx_total_millis, tx_speed_in_mbps) = perform_upload_test(
                &best_server.url,
                &config.upload,
                &sizes);
            record.push(tx_total_bytes.to_string());
            record.push(tx_total_millis.to_string());
            record.push(tx_speed_in_mbps.to_string());
            record.push(get_current_time_as_string());
            // run a HTTP server in probably main thread and do the rest in separate thread.
            println!("Done");
            records.push(record);
        }

        match file_name {
            Some(f)     => {
                let mut writer = csv::Writer::from_memory();
                for record in records {
                    writer.encode(record);
                }

                // println!("{}", writer.into_string());
                file_utils::write_to_file(writer.into_string(), f);
                println!("Finished writing to csv file {}", f);
            }
            None        => {}
        }

    } else {
        println!("Cannot find any servers, please note that if you're searching by country name currently it is an exact match.");
    }

}


fn main() {
    println!("");

    let matches = args::parse_args();
    let number_of_tests = matches.value_of("number_tests");
    let csv_file_name = matches.value_of("csv");
    let server_country = matches.value_of("server-country");
    let server_country_code = matches.value_of("server-country-code");
    let use_cached_servers = matches.is_present("use_cached");

    let mut n_tests: u64 = 1;

    if let Some(n) = number_of_tests {
        // Any non-numerical number of tests will default to 1 test
        let num_tests: u64 = n.parse::<u64>().unwrap_or(1);
        n_tests = num_tests;
    }

    println!("Number of tests to run {}", n_tests);
//    println!("CSV file name {:?}", csv_file_name);
//    println!("Server country - {:?} code - {:?}", server_country, server_country_code);
    run_test(n_tests, csv_file_name, server_country, server_country_code, use_cached_servers);
}
