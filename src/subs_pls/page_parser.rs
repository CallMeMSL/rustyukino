use regex::Regex;
use reqwest;
use crate::subs_pls::db;
use serde::{Deserialize, Serialize};
use easy_scraper::Pattern;
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct Show {
    pub id: String,
    pub name: String,
    pub image_url: String,
    pub synopsis: String,
    pub air_time: AirTime,
}

#[derive(Clone)]
pub struct AirTime {
    pub is_airing: bool,
    pub est_week_day: i32,
    pub est_h: i32,
    pub est_m: i32,
}

impl AirTime {
    pub fn to_string(&self) -> String {
        if !self.is_airing { return "".to_string(); };
        format!("{}, {}", self.to_weekday_string(), self.to_clock_stamp())
    }
    pub fn to_clock_stamp(&self) -> String {
        if !self.is_airing { return "".to_string(); };
        format!("{:02}:{:02}", self.est_h, self.est_m)
    }

    pub fn to_weekday_string(&self) -> String {
        if !self.is_airing { return "".to_string(); };
        let weekdays = AirTime::weekdays();
        format!("{}", weekdays[self.est_week_day as usize])
    }

    pub fn weekdays() -> [&'static str; 7] {
        ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"]
    }
}


#[derive(Serialize, Deserialize)]
struct ScheduleShow {
    title: String,
    page: String,
    image_url: String,
    time: String,
}

#[derive(Serialize, Deserialize)]
struct ScheduleContainer {
    tz: String,
    schedule: BTreeMap<String, Vec<ScheduleShow>>,
}

#[derive(Debug, PartialEq)]
pub enum AddFailure {
    AlreadyAdded,
    InvalidUrl,
    ShowNotAvailable,
    NameNotFound,
    DatabaseError,
}

#[test]
fn test_is_valid_url() {
    assert!(is_valid_url("https://subsplease.org/shows/yami-shibai-9/"));
    assert!(is_valid_url("https://subsplease.org/shows/seirei-gensouki/"));
    assert!(is_valid_url("https://subsplease.org/shows/d_cide-traumerei-the-animation/"));
    assert!(!is_valid_url("https://google.com/"));
    assert!(!is_valid_url("lol https://subsplease.org/shows/d_cide-traumerei-the-animation/"));
    assert!(!is_valid_url("http://subsplease.org/shows/detective-conan/")); //http no good
    assert!(!is_valid_url("https://susbplease.org/shows/detective-conan/")); //typo in link
    assert!(!is_valid_url("https://subsplease.org/shws/detective-conan/")); //another one
    assert!(!is_valid_url("https://subsplease.org/shows//")); //empty
}


pub fn is_valid_url(url: &str) -> bool {
    lazy_static::lazy_static! {
            static ref CHECK_URL: Regex = Regex::new("\\Ahttps://subsplease.org/shows/[A-Za-z0-9_-]+/").unwrap();
        }
    CHECK_URL.is_match(url)
}


/// Here a user can add a Show to its watchlist. If the show is not in the db,
/// an entry will be generated
pub async fn add_show(user_id: i64, identifier: &str) -> Result<Show, AddFailure> {
    let is_url_ident = is_valid_url(identifier);
    if is_url_ident {
        let show_id = &identifier[29..identifier.len() - 1];
        if !db::is_show_saved(show_id).await.map_err(|_| AddFailure::DatabaseError)? {
            let show = scrape_show(show_id)
                .await.ok_or_else(|| AddFailure::ShowNotAvailable)?;
            db::insert_show(&show).await.map_err(|_| AddFailure::DatabaseError)?;
            let db_interaction = add_user_show(user_id, show_id).await;
            db_interaction.map(|_| show)
        } else {
            let show = db::get_show_from_show_id(show_id).await.map_err(|_| AddFailure::DatabaseError)?;
            let db_interaction = add_user_show(user_id, show_id).await;
            db_interaction.map(|_| show)
        }
    } else if !is_url_ident && identifier.contains("http") {
        Err(AddFailure::InvalidUrl)
    } else {
        Err(AddFailure::NameNotFound) //TODO: fuzzy name search
    }
}

async fn add_user_show(user_id: i64, show_id: &str) -> Result<(), AddFailure> {
    let is_already_added = db::does_user_show_exist(user_id, show_id).await
        .map_err(|_| AddFailure::DatabaseError)?;
    if is_already_added {
        return Err(AddFailure::AlreadyAdded);
    }
    db::insert_user_show(user_id, show_id).await
        .map_err(|_| AddFailure::DatabaseError)?;
    Ok(())
}

pub async fn scrape_show(show_id: &str) -> Option<Show> {
    let weekdays = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
    let page_data = reqwest::get(format!("https://subsplease.org/shows/{}/", show_id))
        .await.ok()?.text().await.ok()?;
    let (image_url, synopsis, name) = get_image_synopsis_and_name(&page_data).await?;
    let schedule_data = reqwest::get("https://subsplease.org/api/?f=schedule&tz=Europe/Berlin")
        .await.ok()?.text().await.ok()?;
    let schedule_c: ScheduleContainer = serde_json::from_str(&schedule_data).unwrap();
    let (mut is_airing, mut est_week_day, mut est_h, mut est_m) = (false, -1, -1, -1);
    for (i, &day) in weekdays.iter().enumerate() {
        let shows_today = schedule_c.schedule.get(day)?;
        let op_show = shows_today.iter().find(|&s| s.page == show_id);
        match op_show {
            Some(s) => {
                is_airing = true;
                est_week_day = i as i32;
                let parse_time: Vec<i32> = s.time
                    .split(":")
                    .map(|p| p.parse().unwrap_or_default())
                    .collect();
                est_h = parse_time[0];
                est_m = parse_time[1];
                break;
            }
            None => {}
        };
    };
    Some(Show {
        id: show_id.to_string(),
        name,
        image_url,
        synopsis,
        air_time: AirTime { is_airing, est_week_day, est_h, est_m },
    })
}

async fn get_image_synopsis_and_name(data: &str) -> Option<(String, String, String)> {
    let im_pattern = Pattern::new(r##"<img class="img-responsive img-center" src="{{url}}" />"##).ok()?;
    let synopsis_pattern = Pattern::new(
        r##"<div class="series-syn">
                        <p>{{synopsis}}</p>
                     </div>"##).ok()?;
    let name_pattern = Pattern::new(r##"<h1 class="entry-title">{{name}}</h1>"##).ok()?;
    let image_url = im_pattern.matches(data).get(0)?.get("url")?.to_string();
    let synopsis = synopsis_pattern.matches(data).get(0)?.get("synopsis")?.to_string();
    let name = name_pattern.matches(data).get(0)?.get("name")?.to_string();
    Some((format!("https://subsplease.org{}", image_url), synopsis, name))
}


#[tokio::test]
async fn test_show_scrape() {
    let one_piece = scrape_show("one-piece").await.unwrap();
    assert!(one_piece.air_time.is_airing);
    assert_eq!("One Piece", one_piece.name);
    assert!(one_piece.air_time.est_m < 60);
    assert!(one_piece.air_time.est_h < 24);
}


#[tokio::test]
async fn test_image_synopsis_and_name() {
    let page_data = reqwest::get("https://subsplease.org/shows/re-zero-kara-hajimeru-isekai-seikatsu/")
        .await.unwrap().text().await.unwrap();
    let (image_url, synopsis, name) = get_image_synopsis_and_name(&page_data).await.unwrap();
    assert_eq!(image_url, "https://subsplease.org/wp-content/uploads/2021/01/79410.jpg");
    assert_eq!(&synopsis[..10], "Natsuki Su");
    assert_eq!(name, "Re Zero kara Hajimeru Isekai Seikatsu");
}
