
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
    pub upload: UploadConfig,
    pub parsing_succeeded: bool
}

impl FullConfig {
    pub fn new() -> FullConfig {
        let url = "http://www.speedtest.net/speedtest-config.php";
        let mut client = Client::new();
        client.set_redirect_policy(RedirectPolicy::FollowAll);
        let mut headers = Headers::new();
        headers.set(UserAgent("Hyper-speedtest".to_owned()));

        let mut response = client.get(url)
                            .headers(headers)
                            .send();
        let mut full_config = FullConfig::default();
        full_config.parsing_succeeded = false;

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
                                    let client_config = build_config::<ClientConfig>(ClientConfig::default(), &attributes);
                                    full_config.client = client_config;

                                } else if name.to_string() == "server-config".to_string() {
                                    let server_config = build_config::<ServerConfig>(ServerConfig::default(), &attributes);
                                    full_config.server = server_config;

                                } else if name.to_string() == "download".to_string() {
                                    let download_config = build_config::<DownloadConfig>(DownloadConfig::default(), &attributes);
                                    full_config.download = download_config;

                                } else if name.to_string() == "upload".to_string() {
                                    let upload_config = build_config::<UploadConfig>(UploadConfig::default(), &attributes);
                                    full_config.upload = upload_config;
                                }
                            }
                            Err(e) => {
    //                            println!("Error parsing configuration XML");
                            }
                            _ => {}
                        }
                    }
                    full_config.parsing_succeeded = true;
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
    pub ignoreids: String
}


impl GenerateConfig<ServerConfig> for ServerConfig {
    fn from_xml(client_conf: &Vec<OwnedAttribute>) -> ServerConfig {
        let mut threadcount = String::new();
        let mut ignoreids = String::new();

        for attrib in client_conf {
            if attrib.name.to_string() == "threadcount".to_string() {
                threadcount = attrib.value.to_string();

            } else if attrib.name.to_string() == "ignoreids".to_string() {
                ignoreids = attrib.value.to_string();
            }
        }

        ServerConfig {
            threadcount: threadcount,
            ignoreids: ignoreids
        }
    }
}



#[derive(Debug, Default)]
pub struct DownloadConfig {
    pub testlength: u64,
    pub initialtest: String,
    pub mintestsize: String,
    pub threadsperurl: u64
}


impl GenerateConfig<DownloadConfig> for DownloadConfig {
    fn from_xml(client_conf: &Vec<OwnedAttribute>) -> DownloadConfig {
        let mut testlength: u64 = 0;
        let mut initialtest = String::new();
        let mut mintestsize = String::new();
        let mut threadsperurl: u64 = 0;

        for attrib in client_conf {
            if attrib.name.to_string() == "initialtest".to_string() {
                initialtest = attrib.value.to_string();

            } else if attrib.name.to_string() == "testlength".to_string() {
                let testlength_string = attrib.value.to_string();
                testlength = testlength_string.parse::<u64>().unwrap_or(0);

            } else if attrib.name.to_string() == "threadsperurl".to_string() {
                let threadsperurl_string = attrib.value.to_string();
                threadsperurl = threadsperurl_string.parse::<u64>().unwrap_or(0);

            } else if attrib.name.to_string() == "mintestsize".to_string() {
                mintestsize = attrib.value.to_string();

            }
        }

        DownloadConfig {
            testlength: testlength,
            initialtest: initialtest,
            mintestsize: mintestsize,
            threadsperurl: threadsperurl
        }
    }
}


#[derive(Debug, Default)]
pub struct UploadConfig {
    pub testlength: u64,
    pub ratio: u64,
    pub initialtest: String,
    pub mintestsize: String,
    pub threads: u64,
    pub maxchunksize: String,
    pub maxchunkcount: u64,
    pub threadsperurl: u64
}


impl GenerateConfig<UploadConfig> for UploadConfig {
    fn from_xml(client_conf: &Vec<OwnedAttribute>) -> UploadConfig {
        let mut testlength: u64 = 0;
        let mut ratio: u64 = 0;
        let mut initialtest = String::new();
        let mut mintestsize = String::new();
        let mut threads: u64 = 0;
        let mut maxchunksize = String::new();
        let mut maxchunkcount: u64 = 0;
        let mut threadsperurl: u64 = 0;

        for attrib in client_conf {
            if attrib.name.to_string() == "testlength".to_string() {
                let testlength_string = attrib.value.to_string();
                testlength = testlength_string.parse::<u64>().unwrap_or(0);

            } else if attrib.name.to_string() == "ratio".to_string() {
                let ratio_string = attrib.value.to_string();
                ratio = ratio_string.parse::<u64>().unwrap_or(0);

            } else if attrib.name.to_string() == "initialtest".to_string() {
                initialtest = attrib.value.to_string();

            } else if attrib.name.to_string() == "mintestsize".to_string() {
                mintestsize = attrib.value.to_string();

            } else if attrib.name.to_string() == "threads".to_string() {
                let threads_string = attrib.value.to_string();
                threads = threads_string.parse::<u64>().unwrap_or(0);

            } else if attrib.name.to_string() == "maxchunksize".to_string() {
                maxchunksize = attrib.value.to_string();

            } else if attrib.name.to_string() == "maxchunkcount".to_string() {
                let maxchunkcount_string = attrib.value.to_string();
                maxchunkcount = maxchunkcount_string.parse::<u64>().unwrap_or(0);

            }  else if attrib.name.to_string() == "threadsperurl".to_string() {
                let threadsperurl_string = attrib.value.to_string();
                threadsperurl = threadsperurl_string.parse::<u64>().unwrap_or(0);

            }
        }

        UploadConfig {
            testlength: testlength,
            ratio: ratio,
            initialtest: initialtest,
            mintestsize: mintestsize,
            threads: threads,
            maxchunksize: maxchunksize,
            maxchunkcount: maxchunkcount,
            threadsperurl: threadsperurl
        }
    }
}


pub trait GenerateConfig<T> {
    fn from_xml(&Vec<OwnedAttribute>) -> T;
}


fn build_config<T: GenerateConfig<T> + Default>(type_: T, attribs: &Vec<OwnedAttribute>) -> T {
    T::from_xml(&attribs)
}