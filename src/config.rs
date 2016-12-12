
use std::collections::HashMap;

use xml::reader::{EventReader, XmlEvent};
use xml::attribute::OwnedAttribute;
use hyper;
use hyper::client::Client;
use hyper::client::response::Response;
use hyper::client::RedirectPolicy;
use hyper::client::Body;
use hyper::header::{Headers, UserAgent, Header, ContentLength};

#[derive(Debug, Default)]
pub struct FullConfig {
    pub client: ClientConfig,
    pub server: ServerConfig,
    pub download: DownloadConfig,
    pub upload: UploadConfig
}

#[derive(Debug, Default)]
pub struct ClientConfig {
    pub ip: String,
    pub lat: f32,
    pub lon: f32,
    pub isp: String,
    pub isprating: f32,
    pub ispdlavg: u64,
    pub ispulavg: u64
}

impl GenerateConfig<ClientConfig> for ClientConfig {
    fn from_xml(client_conf: &Vec<OwnedAttribute>) -> ClientConfig {
        let mut ip = String::new();
        let mut lat = 0.0 as f32;
        let mut lon = 0.0 as f32;
        let mut isp = String::new();
        let mut isprating = 0.0 as f32;
        let mut ispdlavg = 0 as u64;
        let mut ispulavg = 0 as u64;

        for attrib in client_conf {
            if attrib.name.to_string() == "ip".to_string() {
                ip = attrib.value.to_string();

            } else if attrib.name.to_string() == "lat".to_string() {
                let lat_string = attrib.value.to_string();
                lat = lat_string.parse::<f32>().unwrap_or(0.0);

            } else if attrib.name.to_string() == "lon".to_string() {
                let lon_string = attrib.value.to_string();
                lon = lon_string.parse::<f32>().unwrap_or(0.0);

            } else if attrib.name.to_string() == "isp".to_string() {
                isp = attrib.value.to_string();

            } else if attrib.name.to_string() == "isprating".to_string() {
                let isprating_string = attrib.value.to_string();
                isprating = isprating_string.parse::<f32>().unwrap_or(0.0);

            } else if attrib.name.to_string() == "ispdlavg".to_string() {
                let ispdlavg_string = attrib.value.to_string();
                ispdlavg = ispdlavg_string.parse::<u64>().unwrap_or(0);

            } else if attrib.name.to_string() == "ispulavg".to_string() {
                let ispulavg_string = attrib.value.to_string();
                ispulavg = ispulavg_string.parse::<u64>().unwrap_or(0);
            }
        }

        ClientConfig {
            ip: ip,
            lat: lat,
            lon: lon,
            isp: isp,
            isprating: isprating,
            ispdlavg: ispdlavg,
            ispulavg: ispulavg
        }
    }
}

#[derive(Debug, Default)]
pub struct ServerConfig {
    pub threadcount: String,
    pub ignoreids: String,
    pub forcepingid: u64
}

#[derive(Debug, Default)]
pub struct DownloadConfig {
    pub testlength: u64,
    pub initialtest: String,
    pub mintestsize: String,
    pub threadsperurl: u64
}

#[derive(Debug, Default)]
pub struct UploadConfig {
    pub testlength: u64,
    pub ratio: u64,
    pub initialtest: String,
    pub mintestsize: String,
    pub threads: u64,
    pub maxchunksize: String,
    pub maxchunkcount: String,
    pub threadsperurl: u64
}


pub trait GenerateConfig<T> {
    fn from_xml(&Vec<OwnedAttribute>) -> T;
}


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


pub fn get_config() -> () {
    let url = "http://www.speedtest.net/speedtest-config.php";
    let mut client = Client::new();
    client.set_redirect_policy(RedirectPolicy::FollowAll);
    let mut headers = Headers::new();
    headers.set(UserAgent("Hyper-speedtest".to_owned()));

    let mut response = client.get(url)
                        .headers(headers)
                        .send();
    let mut client_config = ClientConfig::default();

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
//                                full_config.insert(name.to_string(), attributes);
                                client_config = ClientConfig::from_xml(&attributes);
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
}


fn build_full_config<T: GenerateConfig>(name: String, attribs: &Vec<OwnedAttribute>) -> Option<T> {

    if name.to_string() == "client".to_string() {
        return ClientConfig::from_xml(&attribs);

    }

//        else if name.to_string() == "server-config".to_string() {
//        full_config.insert(name.to_string(), attributes);
//
//    } else if name.to_string() == "download".to_string() {
//        full_config.insert(name.to_string(), attributes);
//
//    } else if name.to_string() == "upload".to_string() {
//        full_config.insert(name.to_string(), attributes);
//    }

}