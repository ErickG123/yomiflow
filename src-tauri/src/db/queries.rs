use rusqlite::{Connection, Result, params};

pub fn bulk_insert_tags(
    conn: &mut Connection,
    tags: Vec<crate::models::TagEntry>,
) -> Result<(), rusqlite::Error> {
    let tx = conn.transaction()?;
    {
        let mut stmt =
            tx.prepare("INSERT OR REPLACE INTO tags (id, category, notes) VALUES (?1, ?2, ?3)")?;

        for tag in tags {
            stmt.execute(params![tag.id, tag.category, tag.notes])?;
        }
    }
    tx.commit()?;
    Ok(())
}

pub fn bulk_insert_kanjis(
    conn: &mut Connection,
    kanjis: Vec<crate::models::KanjiEntry>,
) -> Result<(), rusqlite::Error> {
    let tx = conn.transaction()?;
    {
        let mut stmt = tx.prepare(
            "INSERT OR REPLACE INTO kanjis (character, onyomi, kunyomi, meaning, jlpt_level) VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;

        for kanji in kanjis {
            stmt.execute(params![
                kanji.character,
                kanji.onyomi,
                kanji.kunyomi,
                kanji.meaning,
                kanji.jlpt_level
            ])?;
        }
    }
    tx.commit()?;
    Ok(())
}
