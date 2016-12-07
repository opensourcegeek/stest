extern crate hyper;
extern crate xml;
extern crate url;
extern crate time as ext_time;
extern crate clap;
extern crate csv;
extern crate chrono;

use std::io::Read;
use std::collections::{HashMap, BTreeMap};
use std::thread;
use std::time;
use std::time::Instant;
use std::io::prelude::*;
use std::io;
use std::fs::File;

use hyper::client::Client;
use hyper::client::response::Response;
use hyper::client::RedirectPolicy;
use hyper::client::Body;
use hyper::header::{Headers, UserAgent, Header, ContentLength};
use xml::reader::{EventReader, XmlEvent};
use xml::attribute::OwnedAttribute;
use url::{Url, Host};
use clap::{Arg, App, ArgMatches};

const MAX_NUM_RETRIES: u64      = 10;
const ONE_SEC_IN_MILLIS: u64    = 1000;

const CSV_COLUMN_NAMES: &'static str = "test_number,server_url,rx_start,rx_total_bytes,rx_total_millis,rx_speed_mbps,rx_end,tx_start,tx_total_bytes,tx_total_millis,tx_speed_mbps,tx_end";


#[derive(Debug)]
struct TestServerConfig {
    pub url: String,
    pub latitude: f32,
    pub longitude: f32,
    pub name: String,
    pub country: String,
    pub country_code: String,
    pub id: u64,
    pub url2: String,
    pub host: String
}


#[derive(Debug)]
struct ClientConfig {
    pub ip: String,
    pub lat: f32,
    pub lon: f32,
    pub isp: String,
    pub isprating: f32,
    pub ispdlavg: u64,
    pub ispulavg: u64
}

#[derive(Debug)]
struct ServerConfig {
    pub threadcount: String,
    pub ignoreids: String,
    pub forcepingid: u64
}

#[derive(Debug)]
struct DownloadConfig {
    pub testlength: u64,
    pub initialtest: String,
    pub mintestsize: String,
    pub threadsperurl: u64
}

#[derive(Debug)]
struct UploadConfig {
    pub testlength: u64,
    pub ratio: u64,
    pub initialtest: String,
    pub mintestsize: String,
    pub threads: u64,
    pub maxchunksize: String,
    pub maxchunkcount: String,
    pub threadsperurl: u64
}


fn get_config_map() -> HashMap<String, Vec<OwnedAttribute>> {
    let url = "http://www.speedtest.net/speedtest-config.php";
    let mut client = Client::new();
    client.set_redirect_policy(RedirectPolicy::FollowAll);
    let mut headers = Headers::new();
    headers.set(UserAgent("Hyper-speedtest".to_owned()));

    let mut response = client.get(url)
                        .headers(headers)
                        .send();

    let mut full_config: HashMap<String, Vec<OwnedAttribute>> = HashMap::new();
//    let mut client_config: ClientConfig;
//    println!("{:?}", response);

    match response {
        Ok(res)    => {
            let all_headers_wrapped = &res.headers.to_owned();
            let default_content_len = ContentLength(0);
            let content_length: &ContentLength = all_headers_wrapped.get().unwrap_or(&default_content_len);
//            println!("{:?}", content_length);
            let no_content_length: u64 = 0;
            if res.status == hyper::Ok && content_length.0 > no_content_length {

                let parser = EventReader::new(res);

                for e in parser {
                    match e {
                        Ok(XmlEvent::StartElement { name, attributes, .. }) => {

                            if name.to_string() == "client".to_string() {
                                full_config.insert(name.to_string(), attributes);
//                                let mut ip: String = String::new();
//                                let mut lat: f32 = 10000.0;
//                                let mut lon: f32 = 10000.0;
//                                let mut isp: String = String::new();
//                                let mut isprating: f32 = 0.0;
//                                let mut ispdlavg: u64 = 0;
//                                let mut ispulavg: u64 = 0;
//
//                                for attribute in attributes {
//                                    let att_name = attribute.name.to_string();
//                                    let att_value = &attribute.value;
//
//                                    if att_name == "ip".to_string() {
//                                        ip = att_value.to_string();
//                                    }
//
//                                    if att_name == "lat".to_string() {
//                                        lat = att_value.parse::<f32>().unwrap();
//                                    }
//
//                                    if att_name == "lon".to_string() {
//                                        lon = att_value.parse::<f32>().unwrap();
//                                    }
//
//                                    if att_name == "isp".to_string() {
//                                        isp = att_value.to_string();
//                                    }
//
//                                    if att_name == "ispdlavg".to_string() {
//                                        ispdlavg = att_value.parse::<u64>().unwrap();
//                                    }
//
//                                    if att_name == "ispulavg".to_string() {
//                                        ispulavg = att_value.parse::<u64>().unwrap();
//                                    }
//
//                                    if att_name == "isprating".to_string() {
//                                        isprating = att_value.parse::<f32>().unwrap();
//                                    }
//
//                                }
//
//                                client_config = ClientConfig {
//                                    ip: ip,
//                                    lat: lat,
//                                    lon: lon,
//                                    isp: isp,
//                                    isprating: isprating,
//                                    ispdlavg: ispdlavg,
//                                    ispulavg: ispulavg
//                                };

                            } else if name.to_string() == "server-config".to_string() {
                                full_config.insert(name.to_string(), attributes);

                            } else if name.to_string() == "download".to_string() {
                                full_config.insert(name.to_string(), attributes);

                            } else if name.to_string() == "upload".to_string() {
                                full_config.insert(name.to_string(), attributes);
                            }
                        }
                        Err(e) => {
//                            println!("Error parsing configuration XML");
                        }
                        _ => {}
                    }
                }
//                println!("{:?}", full_config);

            } else {

//                println!("Cannot retrieve config data from server");
            }
        },
        Err(e)      => {
//            println!("Error fetching config file - please try again");
        }

    }

    full_config
}


fn get_all_test_servers() -> Vec<TestServerConfig> {
    let urls = vec![
        "http://www.speedtest.net/speedtest-servers-static.php",
        "http://c.speedtest.net/speedtest-servers-static.php",
        "http://www.speedtest.net/speedtest-servers.php",
        "http://c.speedtest.net/speedtest-servers.php"
    ];

    let mut all_test_servers: Vec<TestServerConfig> = Vec::new();

    for url in urls {
        let mut client = Client::new();
        client.set_redirect_policy(RedirectPolicy::FollowAll);

        let mut headers = Headers::new();
        headers.set(UserAgent("Hyper-speedtest".to_owned()));
        let mut response = client.get(url)
                                .headers(headers)
                                .send();

        match response {
            Ok(res)    => {
                let all_headers_wrapped = &res.headers.to_owned();
                let content_length: &ContentLength = all_headers_wrapped.get().unwrap();
//                println!("{:?}", content_length);
                let no_content_length: u64 = 0;
                if res.status == hyper::Ok && content_length.0 > no_content_length {
                    let parser = EventReader::new(res);
                    for e in parser {
                        match e {
                            Ok(XmlEvent::StartElement { name, attributes, .. }) => {
                                if name.to_string() == "server".to_string() {
//                                    println!("{:?}", attributes);
                                    let mut url: String = String::new();
                                    let mut latitude: f32 = 0.0;
                                    let mut longitude: f32 = 0.0;
                                    let mut name: String = String::new();
                                    let mut country: String = String::new();
                                    let mut country_code: String = String::new();
                                    let mut id: u64 = 0;
                                    let mut url2: String = String::new();
                                    let mut host: String = String::new();

                                    for attribute in attributes {
                                        let att_name = attribute.name.to_string();
                                        let att_value = &attribute.value;
                                        
                                        if att_name == "url".to_string() {
                                            url = att_value.to_string();
                                        }

                                        if att_name == "lat".to_string() {
                                            latitude = att_value.parse::<f32>().unwrap();
                                        }

                                        if att_name == "lon".to_string() {
                                            longitude = att_value.parse::<f32>().unwrap();
                                        }

                                        if att_name == "name".to_string() {
                                            name = att_value.to_string();
                                        }

                                        if att_name == "country".to_string() {
                                            country = att_value.to_string();
                                        }

                                        if att_name == "cc".to_string() {
                                            country_code = att_value.to_string();
                                        }

                                        if att_name == "id".to_string() {
                                            id = att_value.parse::<u64>().unwrap();
                                        }

                                        if att_name == "url2".to_string() {
                                            url2 = att_value.to_string();
                                        }

                                        if att_name == "host".to_string() {
                                            host = att_value.to_string();
                                        }
                                    }

                                    all_test_servers.push(TestServerConfig {
                                        url: url,
                                        id: id,
                                        url2: url2,
                                        host: host,
                                        country: country,
                                        country_code: country_code,
                                        latitude: latitude,
                                        longitude: longitude,
                                        name: name
                                    })

                                }
                            }
                            Err(e) => {
                                println!("Error parsing configuration XML");
                                continue;
                            }
                            _ => {}
                        }
                    }

                    // We got all data so no need to loop through all of the urls.
                    break;

                } else {
                    println!("Cannot retrieve config data from server");
                }
            },
            Err(e)      => {
                println!("Error fetching config file - please try again {}", url);
            }

        }

    }
    all_test_servers
}


fn calc_distance_in_km((lat1, lon1): (f32, f32), (lat2, lon2): (f32, f32)) -> f32 {
    let radius_in_km = 6371.0;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin() * (dlat / 2.0).sin() +
         (lat1.to_radians()).cos() *
         (lat2.to_radians()).cos() *
         (dlon / 2.0).sin() * (dlon / 2.0).sin();
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    (radius_in_km * c) as f32
}


fn pick_closest_servers(client_location: (Option<f32>, Option<f32>),
                        all_test_servers: &Vec<TestServerConfig>,
                        result: &mut Vec<TestServerConfig>)
    -> () {
    // NB: It is important to maintain resul type to be Vec<TestServerConfig> as if
    //     user switches this picking closest servers off, you can still just pass
    //     on servers without having to manipulate types!

    // if we have 2 servers with exact same distance - we are just ignoring the first one.
    // btreemap sorts keys so no extra sorting required
    let mut distance_map: BTreeMap<u64, &TestServerConfig> = BTreeMap::new();

    for server in all_test_servers {
        let client_lat = client_location.0.unwrap();
        let client_lon = client_location.1.unwrap();
        let dist = calc_distance_in_km((client_lat, client_lon), (server.latitude, server.longitude));
//        println!("distance {}", dist);
        distance_map.insert(dist.round() as u64, server);
    }

    let max_servers = 10;
    let mut count = 0;

    for (_, v) in distance_map.iter() {
        count = count + 1;
        result.push(TestServerConfig {
            url: v.url.clone(),
            latitude: v.latitude.clone(),
            longitude: v.longitude.clone(),
            name: v.name.clone(),
            country: v.country.clone(),
            country_code: v.country_code.clone(),
            id: v.id.clone(),
            url2: v.url2.clone(),
            host: v.host.clone()
        });
        if count >= max_servers {
            break;
        }
    }

}


fn find_ignore_ids(ref client_conf: &Vec<OwnedAttribute>) -> Vec<u64> {
    let mut ignored_ids: Vec<u64> = Vec::new();

    for attrib in *client_conf {
        if attrib.name.to_string() == "ignoreids".to_string() {
            let ids_str: String = attrib.value.to_string();
            ignored_ids = ids_str.split(",").map(|x| {
                x.parse::<u64>().unwrap()
            }).collect();
            break;
        }
    }
    ignored_ids
}


fn get_client_location(ref client_conf: &Vec<OwnedAttribute>) -> (Option<f32>, Option<f32>) {
    let mut latitude: Option<f32> = Option::None;
    let mut longitude: Option<f32> = Option::None;
    for attrib in  *client_conf {
        if attrib.name.to_string() == "lat".to_string() {
            latitude = Option::Some(attrib.value.parse::<f32>().unwrap());

        } else if attrib.name.to_string() == "lon".to_string() {
            longitude = Option::Some(attrib.value.parse::<f32>().unwrap());
        }
    }
    (latitude, longitude)
}


fn find_best_server_by_ping(test_servers: &Vec<TestServerConfig>)
    -> &TestServerConfig {
    let mut server_responses: BTreeMap<u64, &TestServerConfig> = BTreeMap::new();

    for s in test_servers {
        let server_url = Url::parse(s.url.as_str()).unwrap();
        let server_url_str = server_url.host_str().unwrap();
//        println!("{}", server_url_str);
        let latency_url = format!("http://{}/speedtest/latency.txt", server_url_str);
        let latency_url_str = latency_url.as_str();
//        println!("{}", latency_url_str);

        let mut total: u64 = 0;

        for i in 0..3 {
            let start = Instant::now();
            let mut client = Client::new();
            client.set_redirect_policy(RedirectPolicy::FollowAll);
            let mut headers = Headers::new();
            headers.set(UserAgent("Hyper-speedtest".to_owned()));
            let mut response = client.get(latency_url_str)
                                .headers(headers)
                                .send();

            match response {
                Ok(resp)    => {
//                    println!("{:?}", resp);

                    if resp.status == hyper::Ok {
                        let elapsed = start.elapsed();
                        let elapsed_as_millis = (elapsed.as_secs() * 1_000) + (elapsed.subsec_nanos() / 1_000_000) as u64;
//                        println!("Time taken span {:?}", elapsed_as_millis);
                        total = total + elapsed_as_millis;

                    } else {
                        // Assuming this server isn't too good - so weighing out
                        total = total + 360000 as u64;
                    }

                },
                Err(e)      => {
//                    println!("Failed to get response on ping");
                    // Failure responses are weighed 360000 = 1hr in millis
                    total = total + 3600000 as u64;
                }
            }
        }

        let latency_avg = total / 3;
        server_responses.insert(latency_avg, s);
    }

    let (latency, best_server) = server_responses.iter().next().unwrap();
    println!("The chosen server is {:?} with HTTP 'ping' latency {:?}ms", best_server.name, latency);
    best_server
}


fn compute_speed_in_mbps(total_bytes: u64, total_time_in_millis: u64) -> f64 {
    let speed = (total_bytes as f64 * 8.0) / (total_time_in_millis as f64 / 1000.0);
    let speed_in_mbps = speed / (1000.0 * 1000.0);
    speed_in_mbps
}


fn perform_download_test(server_url_str: &str, sizes: &Vec<u64>, dimensions: &Vec<u64>) -> (u64, u64, f64) {
    let mut urls: Vec<String> = Vec::new();
    let mut counter = 0;

    for dim in dimensions {
        // 4 threads per URL
        for _ in 0..4 {
            let url = format!("http://{}/speedtest/random{}x{}.jpg?x={}.{}", server_url_str,
                          dim, dim, ext_time::precise_time_s(), counter);
            counter = counter + 1;
            urls.push(url);
        }
    }

//    println!("{:?}", urls.len());
    let mut thread_handles = vec![];
    let start = time::Instant::now();

    for url in urls {
        let handle = thread::spawn(move || {
            let mut client = Client::new();
            client.set_redirect_policy(RedirectPolicy::FollowAll);

            let mut headers = Headers::new();
            headers.set(UserAgent("Hyper-speedtest".to_owned()));
            let mut response = client.get(url.as_str())
                                .headers(headers)
                                .send();

            match response {
                Ok(mut res)   => {
                    if res.status == hyper::Ok {
                        let mut buf: Vec<u8> = Vec::new();
                        let size = res.read_to_end(&mut buf);
                        let downloaded_bytes = size.unwrap() as u64;
//                        println!("Downloaded = {}", downloaded_bytes);
                        print!(".");
                        io::stdout().flush().ok().expect("");
                        return downloaded_bytes;

                    } else {
                        return 0 as u64;
                    }
                }
                Err(res)    => {
                    return 0 as u64;
                }
            }

        });
        thread_handles.push(handle);
    }

    let mut total_download_bytes = 0;

    for h in thread_handles {
        let file_size = h.join();
        total_download_bytes = total_download_bytes + file_size.unwrap_or(0);
    }
    print!("Done\n");
    io::stdout().flush().ok().expect("");

    let elapsed = start.elapsed();
    let elapsed_as_millis = (elapsed.as_secs() * 1_000) + (elapsed.subsec_nanos() / 1_000_000) as u64;
    println!("Downloaded {} bytes in {}ms",
             total_download_bytes,
             elapsed_as_millis);
    let speed_in_mbps = compute_speed_in_mbps(total_download_bytes, elapsed_as_millis);
    println!("Download speed: {} Mbps", speed_in_mbps);
    (total_download_bytes, elapsed_as_millis, speed_in_mbps)
}


fn perform_upload_test(server_url_str: &str, sizes: &Vec<u64>) -> (u64, u64, f64) {
    let mut thread_handles = vec![];
    let start = time::Instant::now();

    for s in sizes {
        let full_size = s.clone();
        let upload_url = server_url_str.to_string();
        let handle = thread::spawn(move || {
            let mut buff: Vec<u8> = vec![0; full_size as usize];
            let mut client = Client::new();
            client.set_redirect_policy(RedirectPolicy::FollowAll);

            let mut headers = Headers::new();
            headers.set(UserAgent("Hyper-speedtest".to_owned()));
            let mut response = client.post(upload_url.as_str())
                                .body(Body::BufBody(&buff, full_size as usize))
                                .headers(headers)
                                .send();

            match response {
                Ok(res)     => {
//                    println!("{:?}", res);
                    print!(".");
                    io::stdout().flush().ok().expect("");
                    full_size
                },
                Err(e)       => {
                    print!(".");
                    io::stdout().flush().ok().expect("");
//                    println!("{:?}", e);
                    0
                }
            }
        });
        thread_handles.push(handle);
    }

    let mut total_upload_bytes = 0;
    for h in thread_handles {
        let uploaded = h.join();
        total_upload_bytes = total_upload_bytes + uploaded.unwrap_or(0);
    }
    print!("Done\n");
    io::stdout().flush().ok().expect("");
    let elapsed = start.elapsed();
    let elapsed_as_millis = (elapsed.as_secs() * 1_000) + (elapsed.subsec_nanos() / 1_000_000) as u64;
    println!("Uploaded {} bytes in {}ms", total_upload_bytes, elapsed_as_millis);
    let speed_in_mbps = compute_speed_in_mbps(total_upload_bytes, elapsed_as_millis);
    println!("Upload speed: {} Mbps", speed_in_mbps);
    (total_upload_bytes, elapsed_as_millis, speed_in_mbps)

}


fn run_test(number_of_tests: u64, file_name: Option<&str>, server_country: Option<&str>) {
    // The speed test config file request returns nothing sometimes, but it looks like a
    // glitch on the server side as similar content-length:0 responses come back when queried
    // using curl as well. To work around it we retry upto MAX_NUM_RETRIES, it should come in
    // via passed in args as well.
    let mut got_config = false;
    let current_count = 0;
    let mut config = get_config_map();
    while !config.contains_key("client") && current_count <= MAX_NUM_RETRIES {
        config = get_config_map();
        thread::sleep(time::Duration::from_millis(ONE_SEC_IN_MILLIS));
    }

//    println!("{:?}", config);
    // TODO: Add a check to exit if we cannot retrieve any config

    let mut test_servers: Vec<TestServerConfig> = get_all_test_servers();
    println!("Total servers available: {:?}", test_servers.len());

    let server_hint_config = config.get("server-config").unwrap();
    let ignore_ids = find_ignore_ids(&server_hint_config);
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
            // look for closest servers - we should add a switch to avoid this distance check
            let client_conf = config.get("client").unwrap();
            let client_location = get_client_location(&client_conf);
            let mut closest_servers: Vec<TestServerConfig> = Vec::new();
            pick_closest_servers(client_location, &test_servers, &mut closest_servers);
            closest_servers
        }
    };

    if closest_servers.len() > 0 {
        // TODO: May be change the server for each test?
        // look for ping latency for all servers (or closest servers)
        let best_server = find_best_server_by_ping(&closest_servers);
        let mut records = Vec::new();

        let mut col_names = Vec::new();
        for name in CSV_COLUMN_NAMES.split(',') {
            col_names.push(name.to_string());
        }

        records.push(col_names);

        for i in 0..number_of_tests {
            let current_test = i + 1;
            println!("Performing test {}", current_test);
            let sizes: Vec<u64> = vec![32768, 65536, 131072, 262144, 524288, 1048576, 7340032];
            let dimensions: Vec<u64> = vec![350, 500, 750, 1000, 1500, 2000, 2500, 3000];
            let mut record = Vec::new();
            let server_url = Url::parse(best_server.url.as_str()).unwrap();
            let server_url_str = server_url.host_str().unwrap();
            record.push(current_test.to_string());
            record.push(server_url_str.to_string());

            // Start tests against chosen server - these download/upload tests will
            // run in separate threads
            print!("Running download tests...");
            record.push(chrono::Local::now().to_string());
            let (rx_total_bytes, rx_total_millis, rx_speed_in_mbps) = perform_download_test(server_url_str, &sizes, &dimensions);
            record.push(rx_total_bytes.to_string());
            record.push(rx_total_millis.to_string());
            record.push(rx_speed_in_mbps.to_string());
            record.push(chrono::Local::now().to_string());
            println!("");

            print!("Running upload tests...");
            record.push(chrono::Local::now().to_string());
            let (tx_total_bytes, tx_total_millis, tx_speed_in_mbps) = perform_upload_test(best_server.url.as_str(), &sizes);
            record.push(tx_total_bytes.to_string());
            record.push(tx_total_millis.to_string());
            record.push(tx_speed_in_mbps.to_string());
            record.push(chrono::Local::now().to_string());
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
                write_to_file(writer.into_string(), f);
                println!("Finished writing to csv file {}", f);
            }
            None        => {}
        }

    } else {
        println!("Cannot find any servers, please note that if you're searching by country name currently it is an exact match.");
    }

}


fn write_to_file(csv_content: String, file_name: &str) -> () {
    let full_file_name;
    if file_name.to_string().ends_with(".csv") {
        full_file_name = format!("{}", file_name);

    } else {
        full_file_name = format!("{}.csv", file_name);
    }

    let mut f = File::create(full_file_name).expect("Unable to create file");
    f.write_all(csv_content.as_bytes()).expect("Unable to write data to file");
}


fn parse_args<'a>() -> ArgMatches<'a> {
    let matches = App::new("stest (speedtest cli)")
        .version("0.2.0")
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
        .get_matches();
    matches
}


fn main() {
    println!("");

    let matches = parse_args();
    let number_of_tests = matches.value_of("number_tests");
    let csv_file_name = matches.value_of("csv");
    let server_country = matches.value_of("server_country");
    let mut n_tests: u64 = 1;

    match number_of_tests {
        Some(n)     => {
            // Any non-numerical number of tests will default to 1 test
            let num_tests: u64 = n.parse::<u64>().unwrap_or(1);
            n_tests = num_tests;
        },
        None        => {}
    }

    println!("Number of tests to run {}", n_tests);
//    println!("CSV file name {:?}", csv_file_name);

    run_test(n_tests, csv_file_name, server_country);

}
