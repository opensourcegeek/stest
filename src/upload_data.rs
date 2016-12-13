
use std::io::Read;
use std::time::Instant;
use std::io::Result;
use std::io::{Error, ErrorKind};

pub struct UploadData {
    pub total_data_size: u64,
    pub timeout_in_sec: u64,
    pub start_time: Instant,
    pub current_size: u64
}

impl UploadData {
    pub fn new(n: u64, timeout_secs: u64) -> UploadData {
        UploadData {
            total_data_size: n,
            timeout_in_sec: timeout_secs,
            start_time: Instant::now(),
            current_size: 0
        }
    }
}


impl Read for UploadData {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
//        println!("Trying to fill in data {:?}", buf.len());
        let constant_buf_size: u64 = 8192;
        let data_to_send = self.total_data_size - self.current_size;
//        println!("Data sent {:?}", data_to_send);
        let elapsed = self.start_time.elapsed();
        let elapsed_in_millis = (elapsed.as_secs() * 1_000) + (elapsed.subsec_nanos() / 1_000_000) as u64;
        if data_to_send > 0 && (elapsed_in_millis <= self.timeout_in_sec * 1000) {
            for i in 0..constant_buf_size {
                buf[i as usize] = 0;
            }
//            println!("Returned data {:?}", buf.len());
            self.current_size = self.current_size + constant_buf_size;
            return Ok(constant_buf_size as usize);

        } else if data_to_send >= 0 {
            return Ok(0 as usize);

        } else {
            return Err(Error::new(ErrorKind::Other, "Error sending upload data"));
        }
    }
}