use crate::cache::Key;

/// Returns a list of dead zones in the form of (offset, size) tuples.
#[allow(unused)]
pub fn compute_dead_zones(keys: &mut [Key], file_size: u64) -> Vec<Key> {
    // Sort the keys based on their offsets.
    keys.sort_by(|a, b| a.0.cmp(&b.0));

    let mut dead_zones = Vec::new();

    let mut last_end = 0;
    for key in keys.iter() {
        let offset = key.0;
        let size = key.1;

        // Check if there's a gap between the end of the last region and the start of the current one.
        if offset > last_end {
            dead_zones.push((last_end, (offset - last_end) as usize));
        }

        last_end = offset + size as u64;
    }

    // Check for a dead zone after the last key until the end of the file.
    if last_end < file_size {
        dead_zones.push((last_end, (file_size - last_end) as usize));
    }

    dead_zones
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_dead_zones() {
        let mut keys = Vec::new();
        keys.push((0, 10));
        keys.push((20, 10));
        keys.push((40, 10));
        keys.push((60, 10));
        keys.push((80, 10));

        let dead_zones = compute_dead_zones(&mut keys, 100);
        assert_eq!(dead_zones.len(), 5);
        assert_eq!(dead_zones[0], (10, 10));
        assert_eq!(dead_zones[1], (30, 10));
        assert_eq!(dead_zones[2], (50, 10));
        assert_eq!(dead_zones[3], (70, 10));
        assert_eq!(dead_zones[4], (90, 10));
    }
}
