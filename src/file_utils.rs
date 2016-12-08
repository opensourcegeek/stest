
use std::fs::File;
use std::io::prelude::*;

pub fn write_to_file(csv_content: String, file_name: &str) -> () {
    let full_file_name = get_full_file_name(file_name);
    let mut f = File::create(full_file_name).expect("Unable to create file");
    f.write_all(csv_content.as_bytes()).expect("Unable to write data to file");
}


fn get_full_file_name(file_name: &str) -> String {
    if file_name.to_string().ends_with(".csv") {
        return format!("{}", file_name);

    }
    // if no csv in file name, return with csv extension
    format!("{}.csv", file_name)
}

#[cfg(test)]
mod tests {
    use super::get_full_file_name;

    #[test]
    fn get_full_file_name_no_csv_extension_test() {
        assert_eq!("abc.csv".to_string(), get_full_file_name("abc"));
    }

    #[test]
    fn get_full_file_name_with_csv_extension_test() {
        assert_eq!("abc.csv".to_string(), get_full_file_name("abc.csv"));
    }
}