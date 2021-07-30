use crate::subs_pls::db;
use crate::subs_pls::page_parser::{Show, AddFailure, add_show, is_valid_url, AirTime};
use std::collections::HashSet;
use reduce::Reduce;


pub async fn is_user_registered(user_id: i64) -> Result<bool, ()> {
    let res = db::is_user_registered(user_id).await;
    match res {
        Ok(b) => Ok(b),
        Err(e) => {
            println!("Error checking user registration: {}", e.to_string());
            Err(())
        }
    }
}

pub async fn register_user(user_id: i64) -> Result<(), ()> {
    let res = db::insert_user(user_id).await;
    match res {
        Ok(()) => Ok(()),
        Err(e) => {
            println!("Error inserting user: {}", e.to_string());
            Err(())
        }
    }
}

pub async fn unregister_user(user_id: i64) -> Result<(), ()> {
    let res = db::remove_user(user_id).await;
    match res {
        Ok(()) => Ok(()),
        Err(e) => {
            println!("Error Removing User: {}", e.to_string());
            Err(())
        }
    }
}

pub async fn add_user_show(user_id: i64, identifier: &str) -> Result<Show, AddFailure> {
    add_show(user_id, identifier).await
}

pub enum RemoveFailure {
    InvalidIdentifier,
    ShowNotFound,
    DBError,
}

pub async fn remove_non_airing(user_id: i64) -> Result<Vec<Show>, RemoveFailure> {
    let user_shows = db::get_shows_for_user(user_id)
        .await.map_err(|_| RemoveFailure::DBError)?;
    let mut removed_shows = Vec::new();
    for show in user_shows.iter() {
        if !show.air_time.is_airing {
            remove_user_show(user_id, &format!("https://subsplease.org/shows/{}/", show.id)).await?;
            removed_shows.push(show.clone());
        }
    }
    Ok(removed_shows)
}

pub struct ShowTable {
    pub release_times: Vec<String>,
    //size m
    pub days: Vec<String>,
    // size n
    pub shows: Vec<Vec<String>>,
    // size nxm
    pub non_airing: Vec<String>,
}

impl ShowTable {
    pub fn get_printable_table(&self) -> Result<Vec<(String, String)>, ()> {
        let mut res = Vec::with_capacity(self.days.len() + 1);
        for (i, day) in self.days.iter().enumerate() {
            let shows_on_day = self.shows.get(i).ok_or_else(|| ())?;
            let timeslot_strings = shows_on_day.iter()
                .zip(&self.release_times)
                .filter(|(name, _)| name != &&"".to_string())
                .map(|(name, time)|
                    format!("{} - {}", time, name))
                .fold("".to_string(), |x, y| format!("{}\n{}", x, y));
            res.push((day.to_string(), timeslot_strings))
        }
        if !self.non_airing.is_empty() {
            let mut non_airing = String::new();
            let stop = self.non_airing.len() - 1;
            for (i, na) in self.non_airing.iter().enumerate() {
                non_airing.push_str(&*na.to_string());

                if i < stop {
                    non_airing.push(',');
                    non_airing.push(' ');
                }
            }
            res.push(("Not currently airing:".to_string(), non_airing));
        }
        Ok(res)
    }
}

pub async fn generate_schedule(user_id: i64) -> Result<ShowTable, ()> {
    let user_shows = db::get_shows_for_user(user_id)
        .await.map_err(|_| ())?;
    let airing_shows: Vec<&Show> = user_shows.iter().filter(|&s| s.air_time.is_airing).collect();
    let non_airing_shows: Vec<&Show> = user_shows.iter().filter(|&s| !s.air_time.is_airing).collect();

    let air_times: HashSet<(i32, i32)> = airing_shows
        .iter()
        .map(|s| (s.air_time.est_h, s.air_time.est_m))
        .collect();


    let mut week_days = Vec::new();
    for day in 1..=7 {
        let mut day_vec = Vec::new();
        for &(h, m) in air_times.iter() {
            let show_names = airing_shows
                .iter()
                .filter(|&&s|
                    s.air_time.est_week_day == day &&
                        s.air_time.est_h == h &&
                        s.air_time.est_m == m)
                .map(|s| s.name.to_string())
                .reduce(|a, b| format!("{}, {}", a, b));
            match show_names {
                Some(s) => day_vec.push(s),
                None => day_vec.push("".to_string())
            };
        }
        week_days.push(day_vec);
    }


    Ok(ShowTable {
        release_times: air_times.iter().map(|&(h, m)| format!("{:02}:{:02}", h, m)).collect(),
        days: AirTime::weekdays().iter().map(|s| s.to_string()).collect(),
        shows: week_days,
        non_airing: non_airing_shows.iter().map(|s| s.name.to_owned()).collect(),
    })
}

pub async fn remove_user_show(user_id: i64, identifier: &str) -> Result<(), RemoveFailure> {
    if is_valid_url(identifier) {
        let show_id = &identifier[29..identifier.len() - 1];
        if db::does_user_show_exist(user_id, show_id).await.map_err(|_| RemoveFailure::DBError)? {
            db::delete_user_show(user_id, show_id)
                .await.map_err(|_| RemoveFailure::DBError)?;
            Ok(())
        } else { Err(RemoveFailure::ShowNotFound) }
    } else {
        if identifier.contains("http") {
            Err(RemoveFailure::InvalidIdentifier)
        } else {
            let res = db::get_show_from_name(identifier).await;
            match res {
                Err(_) => Err(RemoveFailure::ShowNotFound),
                Ok(show) => {
                    db::delete_user_show(user_id, &show.id)
                        .await.map_err(|_| RemoveFailure::DBError)?;
                    Ok(())
                }
            }
        }
    }
}

