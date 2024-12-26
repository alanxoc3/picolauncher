mod schema;

use std::path::Path;

use anyhow;
use diesel::prelude::*;
pub use schema::Cart;
use schema::*;

// TODO create initial db migration

pub static DB_PATH: &'static str = "./db.sqlite";

pub struct DB {
    conn: SqliteConnection,
}

impl DB {
    pub fn connect(db_path: &str) -> anyhow::Result<DB> {
        let conn = SqliteConnection::establish(db_path)?;
        Ok(DB { conn })
    }

    pub fn migrate(&mut self) -> anyhow::Result<()> {
        let sql = r#"
            CREATE TABLE IF NOT EXISTS carts (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                author TEXT NOT NULL,
                likes INTEGER NOT NULL,
                tags TEXT NOT NULL,
                lid TEXT NOT NULL,
                download_url TEXT NOT NULL,
                description TEXT NOT NULL,
                thumb_url TEXT NOT NULL,
                filename TEXT NOT NULL,
                favorite BOOLEAN NOT NULL
            );
        "#;

        diesel::sql_query(sql).execute(&mut self.conn)?;

        Ok(())
    }

    pub fn insert_cart(&mut self, cart: &Cart) -> anyhow::Result<()> {
        use crate::db::carts::{dsl::*, id};

        diesel::insert_into(carts)
            .values(cart)
            .on_conflict(id)
            .do_update()
            .set(cart)
            .execute(&mut self.conn)?;

        Ok(())
    }
    // TODO need to avoid sql injections LOL
    pub fn insert_carts(&mut self, new_carts: &Vec<Cart>) -> anyhow::Result<()> {
        use crate::db::carts::dsl::*;

        diesel::insert_into(carts)
            .values(new_carts)
            .execute(&mut self.conn)?;

        Ok(())
    }

    pub fn get_conn(&mut self) -> &mut SqliteConnection {
        &mut self.conn
    }

    pub fn set_favorite(&mut self, cart_id: i32, is_favorite: bool) -> anyhow::Result<bool> {
        use crate::db::carts::{dsl::*, id};

        diesel::update(carts.filter(id.eq(cart_id)))
            .set(favorite.eq(is_favorite))
            .execute(&mut self.conn)?;

        Ok(is_favorite)
    }
}
