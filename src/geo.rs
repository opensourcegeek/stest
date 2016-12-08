
pub fn calc_distance_in_km((lat1, lon1): (f32, f32), (lat2, lon2): (f32, f32)) -> f32 {
    let radius_in_km = 6371.0;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin() * (dlat / 2.0).sin() +
         (lat1.to_radians()).cos() *
         (lat2.to_radians()).cos() *
         (dlon / 2.0).sin() * (dlon / 2.0).sin();
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    (radius_in_km * c) as f32
}


#[cfg(test)]
mod tests {
    use super::calc_distance_in_km;

    #[test]
    fn calc_distance_in_km_test() {
        let start = (1.0, 1.0);
        let end = (5.0, 5.0);
        assert!(calc_distance_in_km(start, end) > 0.0);
    }


}