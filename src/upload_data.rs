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
    /// This is called from ChunkedBody post request. It stops sending data
    /// when it's elapsed given timeout thereby triggering end of body POST'ed
    /// to server.
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let constant_buf_size: u64 = 8192;
        let data_to_send = self.total_data_size - self.current_size;
        let elapsed = self.start_time.elapsed();
        let elapsed_in_millis = (elapsed.as_secs() * 1_000) + (elapsed.subsec_nanos() / 1_000_000) as u64;

        if elapsed_in_millis < self.timeout_in_sec * 1000 {
            if data_to_send > 0 {
                for i in 0..constant_buf_size {
                    buf[i as usize] = 0;
                }
                self.current_size = self.current_size + constant_buf_size;
                return Ok(constant_buf_size as usize);

            } else {
                return Ok(0 as usize);
            }

        } else {
            // Times up - so return an error..
            return Err(Error::new(ErrorKind::Other, "Error sending upload data - times up"));
        }
    }
}

#[cfg(test)]
mod test {

    use super::UploadData;
    use std::io::Read;
    use std::{time, thread};

    #[test]
    fn test_read_data() -> () {
        let total_data = 8192 * 4; // 32KB
        let mut buffered = UploadData::new(total_data, 1);
        for i in 0..4 {
            let mut data_read: Vec<u8> = vec![1; 8192];
            buffered.read(&mut data_read);
        }
        assert!(buffered.current_size == total_data);
    }

    #[test]
    fn test_timed_out_read_data() -> () {
        let total_data = 8192 * 4; // 32KB
        let mut buffered = UploadData::new(total_data, 1);
        for i in 0..4 {
            let mut data_read: Vec<u8> = vec![1; 8192];
            thread::sleep(time::Duration::from_secs(2));
            let read_response = buffered.read(&mut data_read);
            match read_response {
                Ok(_)   => {println!("Read data fine")},
                Err(e)  => {
                    println!("Timed out");
                    break;
                }
            }
        }
        assert!(buffered.current_size == 0);
    }

}