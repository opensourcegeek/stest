
use std::collections::HashMap;

use xml::reader::{EventReader, XmlEvent};
use xml::attribute::OwnedAttribute;
use hyper;
use hyper::client::Client;
use hyper::client::response::Response;
use hyper::client::RedirectPolicy;
use hyper::client::Body;
use hyper::header::{Headers, UserAgent, Header, ContentLength};

pub fn get_config_map() -> HashMap<String, Vec<OwnedAttribute>> {
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