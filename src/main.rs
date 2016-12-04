
extern crate hyper;
extern crate xml;

use std::io::Read;
use std::collections::HashMap;

use hyper::client::Client;
use hyper::client::response::Response;
use hyper::client::RedirectPolicy;
use hyper::header::{Headers, UserAgent, Header, ContentLength};
use xml::reader::{EventReader, XmlEvent};
use xml::attribute::OwnedAttribute;

const MAX_NUM_RETRIES: u64 = 10;

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
struct FullClientConfig {
    pub client_config: ClientConfig,
//    pub server_config: ServerConfig,
//    pub upload_config: UploadConfig,
//    pub download_config: DownloadConfig
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

//fn do_http_get(url: String) -> Result<Response> {
//    let client = Client::new();
//    client.set_redirect_policy(RedirectPolicy::FollowAll);
//    let mut headers = Headers::new();
//    headers.set(UserAgent("Python-urllib/1.17".to_owned()));
//
//    let mut response = client.get(url)
//                        .headers(headers)
//                        .send();
//    response
//}

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
    let mut full_client_config: FullClientConfig;
    let mut client_config: ClientConfig;


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
        "www.speedtest.net/speedtest-servers-static.php",
        "http://c.speedtest.net/speedtest-servers-static.php",
        "www.speedtest.net/speedtest-servers.php",
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


//fn pick_closest_servers() {
//
//}


fn main() {
    // if we don't get data back use some looping mechanism to retry upto 5 times before
    // giving up??
    let config = get_config_map();
    println!("{:?}", config);

    let test_servers = get_all_test_servers();
    println!("Total servers available: {:?}", test_servers.len());

    // look for closest servers


    // look for ping latency for all servers (or closest servers)

    // with the best server chosen start performing tests. These download/upload tests will
    // run in separate threads

    // run a HTTP server in probably main thread and do the rest in separate thread.

}
