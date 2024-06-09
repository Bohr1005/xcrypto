use crate::chat::Position;
use log::*;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Row, Sqlite};
use std::sync::Arc;
use std::{borrow::Borrow, collections::HashMap};

type Positions = HashMap<String, Position>;
pub struct PositionDB {
    conn: Arc<Pool<Sqlite>>,
    positions: HashMap<u16, Positions>,
}

impl PositionDB {
    pub async fn new(db: &str) -> anyhow::Result<Self> {
        let conn = Arc::new(
            SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(
                    SqliteConnectOptions::new()
                        .create_if_missing(true)
                        .filename(db),
                )
                .await?,
        );

        let mut session_positions = HashMap::default();
        let query = "SELECT name FROM sqlite_master WHERE type='table';";
        let rows = sqlx::query(query).fetch_all(conn.borrow()).await?;

        for row in rows {
            let session_id: String = row.get(0);
            let session_id: u16 = session_id.parse()?;
            let positions = Self::load(conn.clone(), session_id).await?;

            session_positions.insert(session_id, positions);
        }
        Ok(Self {
            conn,
            positions: session_positions,
        })
    }

    async fn load(
        conn: Arc<Pool<Sqlite>>,
        session_id: u16,
    ) -> anyhow::Result<HashMap<String, Position>> {
        let mut positions = HashMap::new();

        let query = format!("SELECT * FROM \"{}\" WHERE net <> 0", session_id);
        let rows: Vec<Position> = sqlx::query_as(&query).fetch_all(conn.borrow()).await?;

        for row in rows {
            info!("Session {} {:?}", session_id, row);
            positions.insert(row.symbol.clone(), row);
        }

        Ok(positions)
    }

    pub fn update(&self, session_id: u16, position: Position) {
        let conn = self.conn.clone();

        let query = format!(
            "REPLACE INTO \"{}\" (symbol, net) VALUES ($1, $2)",
            session_id
        );

        tokio::spawn(async move {
            match sqlx::query(&query)
                .bind(position.symbol.clone())
                .bind(position.net)
                .execute(conn.borrow())
                .await
            {
                Ok(_) => info!("Update {:?}", position),
                Err(e) => error!("{}", e),
            }
        });
    }

    pub fn get_positions(&self, session_id: u16) -> Option<&Positions> {
        self.positions.get(&session_id)
    }

    pub async fn create_table(&self, session_id: u16) -> anyhow::Result<()> {
        let query = format!(
            "CREATE TABLE IF NOT EXISTS \"{}\" (symbol TEXT PRIMARY KEY NOT NULL,  
            net REAL NOT NULL )",
            session_id
        );
        sqlx::query(&query).execute(self.conn.borrow()).await?;
        Ok(())
    }

    pub async fn drop_table(&self, session_id: u16) -> anyhow::Result<()> {
        let query = format!("DROP TABLE IF EXISTS \"{}\"", session_id);
        sqlx::query(&query).execute(self.conn.borrow()).await?;
        Ok(())
    }
}
