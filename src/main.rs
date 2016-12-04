extern crate hyper;
extern crate xml;

use std::io::Read;
use std::collections::HashMap;
use std::{thread, time};

use hyper::client::Client;
use hyper::client::response::Response;
use hyper::client::RedirectPolicy;
use hyper::header::{Headers, UserAgent, Header, ContentLength};
use xml::reader::{EventReader, XmlEvent};
use xml::attribute::OwnedAttribute;

const MAX_NUM_RETRIES: u64      = 10;
const ONE_SEC_IN_MILLIS: u64    = 1000;


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
    headers.set(UserAgent("Python-urllib/1.17".to_owned()));

    let mut response = client.get(url)
                        .headers(headers)
                        .send();

    let mut full_config: HashMap<String, Vec<OwnedAttribute>> = HashMap::new();
//    let mut client_config: ClientConfig;
//    println!("{:?}", response);

    match response {
        Ok(res)    => {
            let all_headers_wrapped = &res.headers.to_owned();
            let content_length: &ContentLength = all_headers_wrapped.get().unwrap();
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
                            println!("Error parsing configuration XML");
                        }
                        _ => {}
                    }
                }
//                println!("{:?}", full_config);

            } else {

                println!("Cannot retrieve config data from server");
            }
        },
        Err(e)      => {
            println!("Error fetching config file - please try again");
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
        headers.set(UserAgent("curl/7.40.0".to_owned()));
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
                        ref all_test_servers: &Vec<TestServerConfig>)
    -> () {

    for server in *all_test_servers {
        let client_lat = client_location.0.unwrap();
        let client_lon = client_location.1.unwrap();
        let dist = calc_distance_in_km((client_lat, client_lon), (server.latitude, server.longitude));
        println!("distance {}", dist);
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


fn main() {
    // The speed test config file request returns nothing sometimes, but it looks like a
    // glitch on the server side as similar content-length:0 responses come back when queried
    // using curl as well. To work around it we retry upto MAX_NUM_RETRIES, it should come in
    // via passed in args as well.
    // TODO: Pass in MAX_NUM_RETRIES from command line
    let mut got_config = false;
    let current_count = 0;
    let mut config = get_config_map();
    while !config.contains_key("client") && current_count <= MAX_NUM_RETRIES {
        config = get_config_map();
        thread::sleep(time::Duration::from_millis(ONE_SEC_IN_MILLIS));
    }
//    println!("{:?}", config);
    // TODO: Add a check to exit if we cannot retrieve any config

    let mut test_servers = get_all_test_servers();
    println!("Total servers available: {:?}", test_servers.len());

    let server_hint_config = config.get("server-config").unwrap();
    let ignore_ids = find_ignore_ids(&server_hint_config);
    println!("Ignored ids: {:?}", ignore_ids);

    // ignore servers on ignore list
    // TODO: Pass in argument to switch off ignore servers recommended by speedtest config?
    test_servers.retain(|ref mut server| {
        // If not ignore ids list keep this server
        !ignore_ids.contains(&server.id)
    });

    println!("Total servers available after ignoring: {:?}", test_servers.len());

    // look for closest servers - we should add a switch to avoid this distance check
    let client_conf = config.get("client").unwrap();
    let client_location = get_client_location(&client_conf);
    let closest = pick_closest_servers(client_location, &test_servers);

    // look for ping latency for all servers (or closest servers)

    // with the best server chosen start performing tests. These download/upload tests will
    // run in separate threads

    // run a HTTP server in probably main thread and do the rest in separate thread.

}
