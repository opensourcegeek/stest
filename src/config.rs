
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