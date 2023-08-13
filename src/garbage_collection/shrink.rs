use crate::cache::Key;

pub type Key2Key = (Key, Key); // old key to new key transformation

pub fn compact_links(keys: &mut [(String, Key)]) -> Vec<(String, Key2Key)> {
    // Sort the keys based on their offsets for consistency.
    keys.sort_by(|a, b| a.1 .0.cmp(&b.1 .0));

    let mut transformations = Vec::new();

    let mut offset = 0;
    for (key, index) in keys.iter_mut() {
        let size = index.1;
        let new_index = (offset, size);
        transformations.push((key.clone(), (*index, new_index)));
        *index = new_index;
        offset += size as u64;
    }

    transformations
}

pub fn compact_file(
    keys: Vec<(String, Key)>,
    file_content: &[u8],
) -> (Vec<(String, Key)>, Vec<u8>) {
    let transformations = compact_links(&mut keys.clone());

    let mut new_file_content = Vec::new();
    let mut new_keys = Vec::new();

    // the String remains the same, but the Key changes
    for (key, (old_index, new_index)) in &transformations {
        let (old_offset, old_size) = *old_index;
        let old_start = old_offset as usize;
        let old_end = old_start + old_size;

        new_file_content.extend_from_slice(&file_content[old_start..old_end]);
        new_keys.push((key.clone(), *new_index));
    }

    (new_keys, new_file_content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compact_links() {
        let mut keys = Vec::new();
        keys.push(("abc".to_string(), (0, 10)));
        keys.push(("def".to_string(), (20, 10)));
        keys.push(("ghi".to_string(), (40, 10)));
        keys.push(("jkl".to_string(), (60, 10)));
        keys.push(("mno".to_string(), (80, 10)));

        let transformations = compact_links(&mut keys);
        assert_eq!(transformations.len(), 5);
        assert_eq!(transformations[0], ("abc".to_string(), ((0, 10), (0, 10))));
        assert_eq!(
            transformations[1],
            ("def".to_string(), ((20, 10), (10, 10)))
        );
        assert_eq!(
            transformations[2],
            ("ghi".to_string(), ((40, 10), (20, 10)))
        );
        assert_eq!(
            transformations[3],
            ("jkl".to_string(), ((60, 10), (30, 10)))
        );
        assert_eq!(
            transformations[4],
            ("mno".to_string(), ((80, 10), (40, 10)))
        );
    }

    #[test]
    fn test_compact_file() {
        let keys = vec![
            ("abc".to_string(), (0, 10)),
            ("def".to_string(), (20, 10)),
            ("ghi".to_string(), (40, 10)),
            ("jkl".to_string(), (60, 10)),
            ("mno".to_string(), (80, 10)),
        ];

        let file = b"0123456789abcdefghijABCDEFGHIJklmnopqrstKLMNOPQRSTabcdefghijAasdhjuiyasshjkewrbj012asd123a";
        let (keys, file) = compact_file(keys, file);

        assert_eq!(keys.len(), 5);
        assert_eq!(keys[0], ("abc".to_string(), (0, 10)));
        assert_eq!(keys[1], ("def".to_string(), (10, 10)));
        assert_eq!(keys[2], ("ghi".to_string(), (20, 10)));
        assert_eq!(keys[3], ("jkl".to_string(), (30, 10)));
        assert_eq!(keys[4], ("mno".to_string(), (40, 10)));

        assert_eq!(file.len(), 50);
        let expected_file = b"0123456789ABCDEFGHIJKLMNOPQRSTAasdhjuiya012asd123a";
        assert_eq!(file, expected_file);
    }
}
