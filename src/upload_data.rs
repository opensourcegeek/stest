use std::io::Read;
use std::time::Instant;
use std::io::Result;
use std::io::{Error, ErrorKind};
use rand;
use rand::{thread_rng, Rng, AsciiGenerator};


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
        let const_buf_size_8kb: u64 = 8 * 1024;
        let buf_size = buf.len() as u64;
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
            return Ok(buf_size as usize);

        } else {
            return Ok(0 as usize);
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
    fn test_random_ascii_generator() -> () {
        let mut generator = thread_rng();

        let chars = generator.gen_ascii_chars().take(8192);
        let mut buf: Vec<u8> = vec![0; 8192];

        let timer = Instant::now();
        for i in 0..8192 {
            buf[i as usize] = chars.nth(i).unwrap_or('0') as u8;
        }
        let el_1 = timer.elapsed();
        let el_millis_1 = (el_1.as_secs() * 1_000) + (el_1.subsec_nanos() / 1_000_000) as u64;
        println!("{:?}", el_millis_1);
        assert!(8192 == buf.len() * std::mem::size_of::<u8>());
    }

}