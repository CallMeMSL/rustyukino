use std::env;
use tokio_postgres::{Client, Error, NoTls};
use crate::subs_pls::page_parser::{Show, AirTime};


async fn connect_db() -> Result<Client, Error> {
    let host = env::var("DB_IP").unwrap();
    let user = env::var("DB_USER").unwrap();
    let db_name = env::var("DB_NAME").unwrap();
    let (client, connection) =
        tokio_postgres::connect(&format!("host={} user={} dbname={}", host, user, db_name),
                                NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    Ok(client)
}

pub async fn get_user_ids_for_show_id(show_id: &str) -> Result<Vec<i64>, Error> {
    let client = connect_db().await?;
    let mut user_ids = Vec::new();
    for row in client.query("select user_id from user_shows where show_id = $1", &[&show_id])
        .await.or_else(|_| Ok(Vec::new()))? {
        let id = row.get(0);
        user_ids.push(id);
    }
    Ok(user_ids)
}

pub async fn is_show_saved(show_id: &str) -> Result<bool, Error> {
    let client = connect_db().await?;
    let res = client.query("select * from shows where id = $1", &[&show_id])
        .await?;
    Ok(!res.is_empty())
}

pub async fn insert_show(show: &Show) -> Result<(), Error> {
    let client = connect_db().await?;
    client.query("insert into shows values ($1, $2, $3, $4, $5, $6, $7, $8)",
                 &[&show.id, &show.name, &show.image_url, &show.synopsis,
                     &show.air_time.is_airing, &show.air_time.est_week_day,
                     &show.air_time.est_h, &show.air_time.est_m]).await?;
    Ok(())
}

pub async fn update_show(show: &Show) -> Result<(), Error> {
    let client = connect_db().await?;
    client.query("update shows set image_url = $2, synopsis = $3,
                 is_airing = $4, est_week_day = $5, est_h = $6, est_m = $7 where id = $1",
                 &[&show.id, &show.image_url, &show.synopsis, &show.air_time.is_airing,
                 &show.air_time.est_week_day, &show.air_time.est_h, &show.air_time.est_m]).await?;
    Ok(())
}

pub async fn get_all_show_ids() -> Result<Vec<String>, Error> {
    let client = connect_db().await?;
    let rows = client.query("select id from shows", &[]).await?;
    Ok(rows.iter().map(|r| r.get(0) ).collect())
}

pub async fn get_show_from_show_id(show_id: &str) -> Result<Show, Error> {
    let client = connect_db().await?;
    let row = client.query_one("select * from shows where id = $1", &[&show_id]).await?;

    let id: &str = row.get(0);
    let name: &str = row.get(1);
    let image_url: &str = row.get(2);
    let synopsis: &str = row.get(3);
    let is_airing: bool = row.get(4);
    let est_week_day: i32 = row.get(5);
    let est_h: i32 = row.get(6);
    let est_m: i32 = row.get(7);

    Ok(Show {
        id: id.to_string(),
        name: name.to_string(),
        image_url: image_url.to_string(),
        synopsis: synopsis.to_string(),
        air_time: AirTime {
            is_airing,
            est_week_day,
            est_h,
            est_m,
        },
    })
}

pub async fn get_shows_for_user(user_id: i64) -> Result<Vec<Show>, Error> {
    let client = connect_db().await?;
    let rows = client.query("select * from shows inner join user_shows us \
        on shows.id = us.show_id where us.user_id = $1", &[&user_id]).await?;
    let mut shows = Vec::with_capacity(rows.len());
    for row in rows {
        let id: &str = row.get(0);
        let name: &str = row.get(1);
        let image_url: &str = row.get(2);
        let synopsis: &str = row.get(3);
        let is_airing: bool = row.get(4);
        let est_week_day: i32 = row.get(5);
        let est_h: i32 = row.get(6);
        let est_m: i32 = row.get(7);

        shows.push(Show {
            id: id.to_string(),
            name: name.to_string(),
            image_url: image_url.to_string(),
            synopsis: synopsis.to_string(),
            air_time: AirTime {
                is_airing,
                est_week_day,
                est_h,
                est_m,
            },
        })
    }
    Ok(shows)
}

pub async fn get_show_from_name(show_name: &str) -> Result<Show, Error> {
    let client = connect_db().await?;
    let row = client.query_one("select * from shows where name = $1", &[&show_name]).await?;

    let id: &str = row.get(0);
    let name: &str = row.get(1);
    let image_url: &str = row.get(2);
    let synopsis: &str = row.get(3);
    let is_airing: bool = row.get(4);
    let est_week_day: i32 = row.get(5);
    let est_h: i32 = row.get(6);
    let est_m: i32 = row.get(7);

    Ok(Show {
        id: id.to_string(),
        name: name.to_string(),
        image_url: image_url.to_string(),
        synopsis: synopsis.to_string(),
        air_time: AirTime {
            is_airing,
            est_week_day,
            est_h,
            est_m,
        },
    })
}

pub async fn does_user_show_exist(user_id: i64, show_id: &str) -> Result<bool, Error> {
    let client = connect_db().await?;
    let is_empty = client.query("select * from user_shows where show_id = $1 and user_id = $2",
                                &[&show_id, &user_id]).await?.is_empty();
    Ok(!is_empty)
}

pub async fn insert_user_show(user_id: i64, show_id: &str) -> Result<(), Error> {
    let client = connect_db().await?;
    client.query("insert into user_shows values ($1, $2)", &[&user_id, &show_id]).await?;
    Ok(())
}

pub async fn delete_user_show(user_id: i64, show_id: &str) -> Result<(), Error> {
    let client = connect_db().await?;
    client.query("delete from user_shows where user_id = $1 and show_id = $2", &[&user_id, &show_id]).await?;
    Ok(())
}


pub async fn is_user_registered(user_id: i64) -> Result<bool, Error> {
    let client = connect_db().await?;
    let res = client.query("select * from users where id = $1", &[&user_id]).await?;
    Ok(!res.is_empty())
}

pub async fn insert_user(user_id: i64) -> Result<(), Error> {
    let client = connect_db().await?;
    client.query("insert into users values ($1)", &[&user_id]).await?;
    Ok(())
}

pub async fn remove_user(user_id: i64) -> Result<(), Error> {
    let client = connect_db().await?;
    client.query("delete from user_shows where user_id = $1", &[&user_id]).await?;
    client.query("delete from users where id = $1", &[&user_id]).await?;
    Ok(())
}


pub struct RssIdDbCommunicator {
    client: Client,
}

impl RssIdDbCommunicator {
    pub async fn new() -> RssIdDbCommunicator {
        RssIdDbCommunicator { client: connect_db().await.unwrap() }
    }
    pub async fn get_guid(&self) -> String {
        self.client
            .query_one("select value from program_state where id = 'last_rss_guid'", &[])
            .await.expect("retrieving last rss guid failed").get(0)
    }
    pub async fn save_guid(&self, guid: &str) -> Result<(), Error> {
        self.client.query("update program_state set value = $1 where id = 'last_rss_guid'", &[&guid])
            .await?;
        Ok(())
    }
}




