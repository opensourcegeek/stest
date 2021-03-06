use std::io::Read;
use std::time::Instant;
use std::io::Result;
use std::io::{Error, ErrorKind};
use std::cmp;

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
        let elapsed = self.start_time.elapsed();
        let elapsed_in_millis = (elapsed.as_secs() * 1_000) + (elapsed.subsec_nanos() / 1_000_000) as u64;

        if elapsed_in_millis >= self.timeout_in_sec * 1000 {
            // Times up - so return an error..
            return Err(Error::new(ErrorKind::Other, "Error sending upload data - times up"));
        }
        // Always chunk by 8K or less
        let const_buf_size_8kb: u64 = 8 * 1024;
        let mut buf_size = buf.len() as u64;
        buf_size = cmp::min(buf_size, const_buf_size_8kb);
        let chars = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz".chars().cycle();
        let data_to_send = self.total_data_size - self.current_size;

        if data_to_send > 0 {
//            let timer = Instant::now();
            for (idx, val) in chars.take(buf_size as usize).enumerate() {
                buf[idx as usize] = val as u8;
            }
//            let loop_elapsed = timer.elapsed();
//            println!("Loop end - {:?}", ((loop_elapsed.as_secs() * 1_000) + (loop_elapsed.subsec_nanos() / 1_000_000) as u64));
            self.current_size = self.current_size + buf_size;
            Ok(buf_size as usize)

        } else {
            Ok(0 as usize)
        }

    }
}

#[cfg(test)]
mod test {

    use super::UploadData;
    use std::io::Read;
    use std::{time, thread};
    use std::time::Instant;
    use rand::{thread_rng, Rng};
    use std;

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

    #[test]
    fn test_smaller_buff_read_data() -> () {
        let total_data = 8192 * 4; // 32KB
        let mut buffered = UploadData::new(total_data, 1);
        let mut num_cycles = 0;
        loop {
            let mut data_read: Vec<u8> = vec![1; 1024];
            let read_response = buffered.read(&mut data_read);
            match read_response {
                Ok(_)   => {
                    println!("Read data fine");
                    num_cycles = num_cycles + 1;
                    if buffered.current_size == total_data {
                        break;
                    }
                },
                Err(e)  => {
                    println!("Timed out");
                    break;
                }
            }
        }
        assert!(buffered.current_size == total_data);
        assert!(num_cycles == 32);
    }

    #[test]
    fn test_bigger_buff_read_data() -> () {
        let total_data = 8192 * 4; // 32KB
        let mut buffered = UploadData::new(total_data, 1);
        let mut num_cycles = 0;
        loop {
            // Try reading 32K in one go - but still only 8K is read each time
            let mut data_read: Vec<u8> = vec![1; total_data as usize];
            let read_response = buffered.read(&mut data_read);
            match read_response {
                Ok(_)   => {
                    println!("Read data fine");
                    num_cycles = num_cycles + 1;
                    if buffered.current_size == total_data {
                        break;
                    }
                },
                Err(e)  => {
                    println!("Timed out");
                    break;
                }
            }
        }
        println!("Num cycles {:?}", num_cycles);
        assert!(buffered.current_size == total_data);
        assert!(4 == num_cycles);
    }

}