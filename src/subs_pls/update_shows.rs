use crate::subs_pls::db;
use crate::subs_pls::page_parser::scrape_show;

pub async fn update_shows() {
    let res = db::get_all_show_ids().await;
    match res {
        Ok(ids) => {
            for id in ids {
                let show = scrape_show(&id).await;
                match show {
                    None => {
                        println!("Error updating show {}", id);
                        break;
                    },
                    Some(s) => {
                        let update_res = db::update_show(&s).await;
                        match update_res {
                            Ok(_) => {},
                            Err(_) => println!("Error updating show {}", s.id)
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_secs(10))
            }
        }
        Err(e) => println!("DB Error updating shows: {}", e.to_string())
    }
}