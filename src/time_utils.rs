use chrono;

pub fn get_current_time_as_string() -> String {
    chrono::Local::now().to_string()
}

fn get_elapsed_in_millis() {

}