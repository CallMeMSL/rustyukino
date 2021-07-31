use regex::Regex;
use roxmltree::Descendants;
use std::fmt;


pub struct SubsPlsChannel {
    pub title: String,
    pub description: String,
    pub items: Vec<FeedItem>,
}


#[derive(Debug, Eq, PartialEq)]
pub enum RssParsingError {
    InvalidRssFeed,
    RssTitleNotFound,
    RssDescriptionNotFound,
    ItemTitleNotFound,
    ItemLinkNotFound,
    ItemGuidNotFound,
    ItemPubDateNotFound,
    ItemCategoryNotFound,
    ItemSizeNotFound,
}

impl fmt::Display for RssParsingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}


fn get_text_in_node_by_name(mut descendants: Descendants, name: &str) -> Option<String> {
    Some(descendants.find(|i| i.tag_name().name() == name)?
        .text()?.to_string())
}

impl SubsPlsChannel {
    pub fn from_xml(xml: &str) -> Result<SubsPlsChannel, RssParsingError> {
        let content = roxmltree::Document::parse(xml)
            .map_err(|_| RssParsingError::InvalidRssFeed)?;
        let mut items: Vec<FeedItem> = Vec::with_capacity(50);

        for i in content.descendants()
            .filter(|n| n.tag_name().name() == "item") {
            let title = get_text_in_node_by_name(i.descendants(), "title")
                .ok_or_else(|| RssParsingError::ItemTitleNotFound)?;
            let link = get_text_in_node_by_name(i.descendants(), "link")
                .ok_or_else(|| RssParsingError::ItemLinkNotFound)?;
            let guid = get_text_in_node_by_name(i.descendants(), "guid")
                .ok_or_else(|| RssParsingError::ItemGuidNotFound)?;
            let pub_date = get_text_in_node_by_name(i.descendants(), "pubDate")
                .ok_or_else(|| RssParsingError::ItemPubDateNotFound)?;
            let category = get_text_in_node_by_name(i.descendants(), "category")
                .ok_or_else(|| RssParsingError::ItemCategoryNotFound)?;
            let file_size = get_text_in_node_by_name(i.descendants(), "size")
                .ok_or_else(|| RssParsingError::ItemSizeNotFound)?;
            items.push(FeedItem { title, link, guid, pub_date, category, file_size });
        }
        Ok(SubsPlsChannel {
            title: content.descendants().find(|n| n.tag_name().name() == "title")
                .ok_or_else(|| RssParsingError::RssTitleNotFound)?.text()
                .ok_or_else(|| RssParsingError::RssTitleNotFound)?.to_string(),
            description: get_text_in_node_by_name(content.descendants(), "description")
                .ok_or_else(|| RssParsingError::RssDescriptionNotFound)?,
            items,
        })
    }
}


pub struct FeedItem {
    pub title: String,
    pub link: String,
    pub guid: String,
    pub pub_date: String,
    pub category: String,
    pub file_size: String,
}


pub fn rss_category_to_show_id(rss_category: &str) -> Option<String> {
    lazy_static::lazy_static! {
            static ref CHECK_INVALID_CHARS: Regex = Regex::new("[^A-Za-z0-9_ ]+").unwrap();
            static ref CHECK_WHITESPACES: Regex = Regex::new(" +").unwrap();
            static ref SKIP_SYMBOLS: [char; 4] = [',', '\'', '(', ')'];
        }

    let name_len = get_last_occurrence_index(rss_category, '-')?;
    let category_stripped: String =
        rss_category.chars()
            .take(name_len)
            .map(|c| c.to_ascii_lowercase())
            .filter(|c| !SKIP_SYMBOLS.contains(c))
            .collect();
    let preprocessed_id = CHECK_INVALID_CHARS.replace_all(&category_stripped, " ").to_string();
    let mut dashed_result: Vec<char> = CHECK_WHITESPACES.replace_all(&preprocessed_id, "-")
        .to_string().chars().collect();
    if dashed_result[0] == '-' {
        dashed_result.remove(0);
    }
    if dashed_result[dashed_result.len() - 1] == '-' {
        dashed_result.remove(dashed_result.len() - 1);
    }
    Some(dashed_result.iter().collect())
}


#[test]
fn rss_title_to_show_id_test() {
    assert_eq!(rss_category_to_show_id("2.43 - Seiin Koukou Danshi Volley-bu - 1080"),
               Some("2-43-seiin-koukou-danshi-volley-bu".to_string()));
    assert_eq!(rss_category_to_show_id("Megami-ryou no Ryoubo-kun. - 1080"),
               Some("megami-ryou-no-ryoubo-kun".to_string()));
    assert_eq!(rss_category_to_show_id("Cheat Kusushi no Slow Life - Isekai ni Tsukurou Drugstore - 1080"),
               Some("cheat-kusushi-no-slow-life-isekai-ni-tsukurou-drugstore".to_string()));
    assert_eq!(rss_category_to_show_id("D_Cide Traumerei the Animation@ - 1080"),
               Some("d_cide-traumerei-the-animation".to_string()));
}


/// return 0 if char not in text
fn get_last_occurrence_index(text: &str, chr: char) -> Option<usize> {
    let mut index: i32 = -1;
    for (i, c) in text.chars().enumerate() {
        if c == chr { index = i as i32 }
    }
    if index == -1 { None } else { Some(index as usize) }
}

#[test]
fn gloi_test() {
    assert_eq!(get_last_occurrence_index("", '@'), None);
    assert_eq!(get_last_occurrence_index("a@la@basta", '@'), Some(4));
    assert_eq!(get_last_occurrence_index("Megami-ryou no Ryoubo-kun. - 1080", '-'), Some(27));
    assert_eq!(get_last_occurrence_index("k!!!!!!!", 'k'), Some(0));
}


#[test]
fn parse_test() {
    let ex_1 = r##"
            <rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom" xmlns:subsplease="https://subsplease.org/rss">
                <channel>
                    <title>SubsPlease RSS</title>
                    <description>RSS feed for SubsPlease releases (1080p)</description>
                    <link>https://subsplease.org</link>
                    <atom:link href="http://subsplease.org/rss" rel="self" type="application/rss+xml"/>
                    <item>
                        <title>[SubsPlease] Yami Shibai 9 - 02 (1080p) [C68BD8C2].mkv</title>
                        <link>test.rs</link>
                        <guid isPermaLink="false">TT75FZ2BERWLLJR2EF764LDMU53XASEV</guid>
                        <pubDate>Sun, 18 Jul 2021 19:31:11 +0000</pubDate>
                        <category>Yami Shibai 9 - 1080</category>
                        <subsplease:size>269.71 MiB</subsplease:size>
                    </item>
                    <item>
                        <title>[SubsPlease] Kingdom S3 - 14 (1080p) [E0FDE25E].mkv</title>
                        <link>test.rs</link>
                        <guid isPermaLink="false">LCSAY3AP3K5YDP3HZKSDDRLVQTNHR4WZ</guid>
                        <pubDate>Sun, 18 Jul 2021 18:58:32 +0000</pubDate>
                        <category>Kingdom S3 - 1080</category>
                        <subsplease:size>1.09 GiB</subsplease:size>
                    </item>
                </channel>
            </rss>
        "##;
    let correct_feed = SubsPlsChannel::from_xml(ex_1).unwrap();
    assert_eq!(correct_feed.title, "SubsPlease RSS");
    assert_eq!(correct_feed.description, "RSS feed for SubsPlease releases (1080p)");
    assert_eq!(correct_feed.items.len(), 2);
    assert_eq!(correct_feed.items[0].category, "Yami Shibai 9 - 1080");
    assert_eq!(correct_feed.items[1].file_size, "1.09 GiB");

    let ex_2 = r##"
            <rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom" xmlns:subsplease="https://subsplease.org/rss">
                <channel>
                    <description>RSS feed for SubsPlease releases (1080p)</description>
                    <link>https://subsplease.org</link>
                    <atom:link href="http://subsplease.org/rss" rel="self" type="application/rss+xml"/>
                </channel>
            </rss>
        "##;

    let title_missing = SubsPlsChannel::from_xml(ex_2);
    assert_eq!(title_missing.err().unwrap(), RssParsingError::RssTitleNotFound);

    let ex_3 = r##"
            <rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom" xmlns:subsplease="https://subsplease.org/rss">
                <channel>
                    <title>SubsPlease RSS</title>
                    <description>RSS feed for SubsPlease releases (1080p)</description>
                    <link>https://subsplease.org</link>
                    <atom:link href="http://subsplease.org/rss" rel="self" type="application/rss+xml"/>
                    <item>
                        <title>[SubsPlease] Yami Shibai 9 - 02 (1080p) [C68BD8C2].mkv</title>
                        <guid isPermaLink="false">TT75FZ2BERWLLJR2EF764LDMU53XASEV</guid>
                        <pubDate>Sun, 18 Jul 2021 19:31:11 +0000</pubDate>
                        <category>Yami Shibai 9 - 1080</category>
                        <subsplease:size>269.71 MiB</subsplease:size>
                    </item>
                </channel>
            </rss>
        "##;

    let item_missing_link = SubsPlsChannel::from_xml(ex_3);
    assert_eq!(item_missing_link.err().unwrap(), RssParsingError::ItemLinkNotFound);
}
