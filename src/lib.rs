extern crate hyper;
extern crate xml;
extern crate url;
extern crate time as ext_time;
extern crate csv;
#[macro_use]
extern crate clap;
extern crate chrono;
extern crate rand;
extern crate hyper_timeout_connector;

pub mod file_utils;
pub mod geo;
pub mod upload_data;
pub mod config;
pub mod utils;
pub mod time_utils;

use std::io::Read;
use std::io::Write;
use std::collections::{HashMap, BTreeMap};
use std::thread;
use std::time;
use std::time::Instant;
//use std::io::prelude::*;
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
use hyper_timeout_connector::HttpTimeoutConnector;

use config::TestServerConfig;
use utils::compute_speed_in_mbps;


pub fn pick_closest_servers(client_location: (f32, f32),
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
        let client_lat = client_location.0;
        let client_lon = client_location.1;
        let dist = geo::calc_distance_in_km((client_lat, client_lon),
                                            (server.latitude, server.longitude));
//        println!("distance {}", dist);
        distance_map.insert(dist.round() as u64, server);
    }

    let max_servers = 5;
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

type Latency = u64;

pub fn find_best_server_by_ping(test_servers: &Vec<TestServerConfig>)
                            -> (&TestServerConfig, Latency) {
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
            let mut connector = HttpTimeoutConnector::new();
            connector.set_connect_timeout(Some(time::Duration::from_secs(30)));

            let mut client = Client::with_connector(connector);
            client.set_read_timeout(Some(time::Duration::from_secs(10)));
            client.set_write_timeout(Some(time::Duration::from_secs(10)));

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
    (best_server, *latency)
}


pub fn perform_download_test(server_url_str: &str, dimensions: &Vec<u64>) -> (u64, u64, f64) {
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
            let mut connector = HttpTimeoutConnector::new();
            connector.set_connect_timeout(Some(time::Duration::from_secs(30)));

            let mut client = Client::with_connector(connector);
            client.set_read_timeout(Some(time::Duration::from_secs(10)));
            client.set_write_timeout(Some(time::Duration::from_secs(10)));

            client.set_redirect_policy(RedirectPolicy::FollowAll);

            let mut headers = Headers::new();
            headers.set(UserAgent("Hyper-speedtest".to_owned()));
            let mut response = client.get(url.as_str())
                .headers(headers)
                .send();

            match response {
                Ok(mut res)   => {
                    if res.status == hyper::Ok {
                        let mut all_read = false;
                        let mut read_bytes = 0;

                        while !all_read {
                            let elapsed = start.elapsed();
                            if elapsed.as_secs() >= 10 {
                                // Link is slow - so not worth reading any more and quit this thread
                                break;
                            }

                            let mut buf: Vec<u8> = vec![0; 8192];
                            let size = res.read(&mut buf);
                            match size {
                                Ok(s)   => {
                                    read_bytes = read_bytes + s as u64;
                                    if s == 0  {
                                        // break out of loop as all read!
                                        all_read = true;
                                    }
                                },
                                Err(e) => {
                                    all_read = true;
                                }
                            }

                        }
//                        println!("Downloaded = {} in {} seconds", read_bytes, start.elapsed().as_secs());
                        print!(".");
                        io::stdout().flush().ok().expect("");
//                        io::stdout().write_all("\x1b[1K".as_bytes()).unwrap();
                        read_bytes

                    } else {
                        0 as u64
                    }
                }
                Err(res)    => {
                    0 as u64
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


pub fn perform_upload_test(server_url_str: &str,
                       client_conf: &config::UploadConfig,
                       sizes: &Vec<u64>) -> (u64, u64, f64) {
    io::stdout().flush().ok().expect("");
    let mut thread_handles = vec![];
    let start = time::Instant::now();
    let ratio = client_conf.ratio;
    let max_chunk_count = client_conf.maxchunkcount;
    let size_max = (ratio - 1) as usize;
    let upload_sizes = sizes.into_iter().skip(size_max);

    let upload_count = if upload_sizes.len() == 0 {
        1_u64

    } else {
        ((max_chunk_count * 2) / upload_sizes.len() as u64) as u64
    };

    let upload_threads = client_conf.threads;
    let upload_length = client_conf.testlength;

    let mut all_sizes: Vec<u64> = Vec::new();
    for size in upload_sizes {
        for _ in 0..upload_count {
            all_sizes.push(*size);
        }
    }

    let picked_sizes = all_sizes.into_iter().take(max_chunk_count as usize);

    for s in picked_sizes {
        let full_size = s.clone();
        let upload_url = server_url_str.to_string();

        let handle = thread::spawn(move || {
            let mut total_bytes_uploaded = 0;

            let mut connector = HttpTimeoutConnector::new();
            connector.set_connect_timeout(Some(time::Duration::from_secs(30)));

            let mut client = Client::with_connector(connector);
            client.set_redirect_policy(RedirectPolicy::FollowAll);
            client.set_read_timeout(Some(time::Duration::from_secs(5)));
            client.set_write_timeout(Some(time::Duration::from_secs(5)));

            let mut headers = Headers::new();
            headers.set(UserAgent("Hyper-speedtest".to_owned()));

            let mut buffered = upload_data::UploadData::new(full_size, upload_length);
            {
                let mut response = client.post(upload_url.as_str())
                    //.body(Body::BufBody(&buff, full_size as usize))
                    .body(Body::ChunkedBody(&mut buffered))
                    .headers(headers.clone())
                    .send();

                match response {
                    Ok(res)     => {
                        //                    println!("{:?}", res);
                        print!(".");
                        io::stdout().flush().ok().expect("");
                    },
                    Err(e)       => {}
                }
            }
            total_bytes_uploaded = total_bytes_uploaded + buffered.current_size;
            total_bytes_uploaded

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

pub fn parse_url(server_url: &str) -> String {
    let url_obj = Url::parse(server_url).unwrap();
    url_obj.host_str().unwrap().to_string()
}
