use crate::types::ArticleMeta;
use rand::{rng, Rng};
use rusqlite::{params, Connection, OpenFlags, Row};
use std::{collections::HashSet, path::Path, sync::Mutex};

pub struct Dataset {
    connection: Mutex<Connection>,
}

impl Dataset {
    pub fn open(path: &Path) -> Result<Self, String> {
        let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| format!("The local Wikipedia title database could not be opened: {e}"))?;
        let version: String = connection
            .query_row(
                "SELECT value FROM metadata WHERE key='schema_version'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| format!("Article database is invalid: {e}"))?;
        if version != "1" {
            return Err(format!("Unsupported article database schema: {version}"));
        }
        Ok(Self {
            connection: Mutex::new(connection),
        })
    }

    pub fn eligible_articles(&self, limit: usize) -> Result<Vec<ArticleMeta>, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "Article database lock was poisoned".to_string())?;
        let max_id: u32 = connection
            .query_row("SELECT COALESCE(MAX(id),0) FROM articles", [], |row| {
                row.get(0)
            })
            .map_err(|error| error.to_string())?;
        if max_id == 0 || limit == 0 {
            return Ok(Vec::new());
        }
        let pivot = rng().random_range(0..=max_id);
        let mut output = Vec::with_capacity(limit);
        let mut seen = HashSet::with_capacity(limit);
        collect_eligible(
            &connection,
            "id >= ?1",
            pivot,
            limit,
            &mut output,
            &mut seen,
        )?;
        if output.len() < limit {
            collect_eligible(
                &connection,
                "id < ?1",
                pivot,
                limit - output.len(),
                &mut output,
                &mut seen,
            )?;
        }
        Ok(output)
    }

    pub fn version(&self) -> String {
        let Ok(connection) = self.connection.lock() else {
            return "unknown".into();
        };
        connection
            .query_row(
                "SELECT value FROM metadata WHERE key='dataset_version'",
                [],
                |row| row.get(0),
            )
            .or_else(|_| {
                connection.query_row(
                    "SELECT value FROM metadata WHERE key='dataset_kind'",
                    [],
                    |row| row.get(0),
                )
            })
            .unwrap_or_else(|_| "unknown".into())
    }
}

fn collect_eligible(
    connection: &Connection,
    range: &str,
    pivot: u32,
    limit: usize,
    output: &mut Vec<ArticleMeta>,
    seen: &mut HashSet<u32>,
) -> Result<(), String> {
    let sql = format!("SELECT id,title,normalized_title,is_redirect,is_disambiguation,in_degree,out_degree,topic_mask,community_id,sig0,sig1,sig2,sig3 FROM articles WHERE is_redirect=0 AND is_disambiguation=0 AND out_degree>={} AND {range} ORDER BY id LIMIT ?2", crate::randomizer::MIN_OUT_DEGREE);
    let mut query = connection
        .prepare(&sql)
        .map_err(|error| error.to_string())?;
    let rows = query
        .query_map(params![pivot, limit as u64], article_from_row)
        .map_err(|error| error.to_string())?;
    for value in rows {
        let article = value.map_err(|error| error.to_string())?;
        if seen.insert(article.id) {
            output.push(article);
        }
    }
    Ok(())
}

fn article_from_row(row: &Row<'_>) -> rusqlite::Result<ArticleMeta> {
    Ok(ArticleMeta {
        id: row.get(0)?,
        title: row.get(1)?,
        normalized_title: row.get(2)?,
        is_redirect: row.get::<_, u8>(3)? != 0,
        is_disambiguation: row.get::<_, u8>(4)? != 0,
        in_degree: row.get(5)?,
        out_degree: row.get(6)?,
        topic_mask: row.get(7)?,
        community_id: row.get(8)?,
        graph_signature: [row.get(9)?, row.get(10)?, row.get(11)?, row.get(12)?],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexed_sampler_is_bounded_and_unique() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../data/articles.sqlite");
        let dataset = Dataset::open(&path).expect("development dataset");
        let articles = dataset.eligible_articles(32).expect("eligible sample");
        assert!(articles.len() >= 2 && articles.len() <= 32);
        let ids: HashSet<_> = articles.iter().map(|article| article.id).collect();
        assert_eq!(ids.len(), articles.len());
    }
}
