mod release_parser {
    use regex::Regex;

    #[test]
    fn rss_title_to_show_id_test() {
        assert_eq!(rss_category_to_show_id("2.43 - Seiin Koukou Danshi Volley-bu - 1080"),
                   Some("2-43-seiin-koukou-danshi-volley-bu".to_string()));
        assert_eq!(rss_category_to_show_id("Megami-ryou no Ryoubo-kun. - 1080"),
                   Some("megami-ryou-no-ryoubo-kun".to_string()));
        assert_eq!(rss_category_to_show_id("Cheat Kusushi no Slow Life - Isekai ni Tsukurou Drugstore - 1080"),
                   Some("cheat-kusushi-no-slow-life-isekai-ni-tsukurou-drugstore".to_string()));
    }


    fn rss_category_to_show_id(rss_category: &str) -> Option<String> {
        lazy_static::lazy_static! {
            static ref CHECK_INVALID_CHARS: Regex = Regex::new("[^A-Za-z0-9 ]+").unwrap();
            static ref CHECK_WHITESPACES: Regex = Regex::new(" +").unwrap();
            static ref SKIP_SYMBOLS : [char; 4] = [',', '\'', '(', ')'];
        }

        let name_len = get_last_occurrence_index(rss_category, '-')?;
        let category_stripped: String =
            rss_category.chars()
                .take(name_len)
                .map(|c| c.to_ascii_lowercase())
                .filter(|c| !SKIP_SYMBOLS.contains(c))
                .collect();
        let preprocessed_id = CHECK_INVALID_CHARS.replace_all(&category_stripped, " ").to_string();
        let mut dashed_result :Vec<char>= CHECK_WHITESPACES.replace_all(&preprocessed_id, "-")
            .to_string().chars().collect();
        if dashed_result[0] == '-' {
            dashed_result.remove(0);
        }
        if dashed_result[dashed_result.len()-1] == '-' {
            dashed_result.remove(dashed_result.len()-1);
        }
        Some(dashed_result.iter().collect())
    }


    #[test]
    fn gloi_test() {
        assert_eq!(get_last_occurrence_index("", '@'), None);
        assert_eq!(get_last_occurrence_index("a@la@basta", '@'), Some(4));
        assert_eq!(get_last_occurrence_index("Megami-ryou no Ryoubo-kun. - 1080", '-'), Some(27));
        assert_eq!(get_last_occurrence_index("k!!!!!!!", 'k'), Some(0));
    }

    /// return 0 if char not in text
    fn get_last_occurrence_index(text: &str, chr: char) -> Option<usize> {
        let mut index: i32 = -1;
        for (i, c) in text.chars().enumerate() {
            if c == chr { index = i as i32 }
        }
        if index == -1 { None } else { Some(index as usize) }
    }
}