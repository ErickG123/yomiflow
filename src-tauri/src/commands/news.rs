use crate::models::NhkNewsItem;
use std::time::Duration;

pub fn strip_html_tags(html: &str) -> String {
    let mut result = String::with_capacity(html.len());
    let mut inside_tag = false;
    for c in html.chars() {
        if c == '<' {
            inside_tag = true;
        } else if c == '>' {
            inside_tag = false;
        } else if !inside_tag {
            result.push(c);
        }
    }
    result.trim().to_string()
}

pub fn sanitize_body_html(html: &str) -> String {
    // Keep useful ruby tags, remove script, style, audio, links tables
    let mut cleaned = html.to_string();
    if let Some(pos) = cleaned.find("<audio") {
        cleaned.truncate(pos);
    }
    if let Some(pos) = cleaned.find("<table") {
        cleaned.truncate(pos);
    }
    cleaned.trim().to_string()
}

#[tauri::command]
pub async fn fetch_nhk_easy_news(
    state: tauri::State<'_, crate::AppState>,
) -> Result<Vec<NhkNewsItem>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    let now_ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    // Primary source: nhkeasier.com (mirrors and cleans NHK News Web Easy articles daily)
    let resp_res = client.get("https://nhkeasier.com/").send().await;

    let mut fetched_items = Vec::new();

    if let Ok(resp) = resp_res
        && resp.status().is_success()
        && let Ok(html) = resp.text().await
    {
        // Parse each <article lang=ja> ... </article>
        let mut search_pos = 0;
        while let Some(start_idx) = html[search_pos..].find("<article") {
            let actual_start = search_pos + start_idx;
            let end_tag = "</article>";
            if let Some(end_offset) = html[actual_start..].find(end_tag) {
                let actual_end = actual_start + end_offset + end_tag.len();
                let article_chunk = &html[actual_start..actual_end];
                search_pos = actual_end;

                // Extract Title from <h3>...</h3>
                let title_with_ruby = article_chunk.find("<h3>").and_then(|h3_start| {
                    article_chunk[h3_start..].find("</h3>").map(|h3_end| {
                        article_chunk[h3_start + 4..h3_start + h3_end]
                            .trim()
                            .to_string()
                    })
                });

                let title = title_with_ruby
                    .as_ref()
                    .map(|t| strip_html_tags(t))
                    .unwrap_or_else(|| "ニュース".to_string());

                // Extract Image from <img src="..."
                let image_url = if let Some(img_start) = article_chunk.find("<img") {
                    let img_part = &article_chunk[img_start..];
                    if let Some(src_start) = img_part.find("src=\"") {
                        let after_src = &img_part[src_start + 5..];
                        if let Some(quote_end) = after_src.find('\"') {
                            let path = &after_src[..quote_end];
                            if path.starts_with("http") {
                                Some(path.to_string())
                            } else {
                                Some(format!("https://nhkeasier.com{}", path))
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Extract audio URL from <audio ... src="..."> or <source src="...">
                let audio_url = if let Some(audio_start) = article_chunk.find("<audio") {
                    let audio_part = &article_chunk[audio_start..];
                    let audio_tag_end = audio_part.find("</audio>").unwrap_or(audio_part.len());
                    let audio_snippet = &audio_part[..audio_tag_end];

                    if let Some(src_pos) = audio_snippet.find("src=\"") {
                        let after_src = &audio_snippet[src_pos + 5..];
                        if let Some(quote_end) = after_src.find('\"') {
                            let path = &after_src[..quote_end];
                            if path.starts_with("http") {
                                Some(path.to_string())
                            } else {
                                Some(format!("https://nhkeasier.com{}", path))
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Extract published_at date from <time datetime="...">
                let published_at = if let Some(time_start) = article_chunk.find("<time datetime=\"")
                {
                    let after_time = &article_chunk[time_start + 16..];
                    after_time.find('\"').map(|q| after_time[..q].to_string())
                } else {
                    None
                };

                // Extract story ID or permalink
                let id = if let Some(story_pos) = article_chunk.find("/story/") {
                    let after = &article_chunk[story_pos + 7..];
                    if let Some(slash_end) = after.find('/') {
                        after[..slash_end].to_string()
                    } else {
                        format!("story-{}", fetched_items.len() + 1)
                    }
                } else {
                    format!("story-{}", fetched_items.len() + 1)
                };

                // Extract paragraphs <p>...</p>
                let mut paragraphs_html = Vec::new();
                let mut p_search = 0;
                while let Some(p_start) = article_chunk[p_search..].find("<p>") {
                    let actual_p_start = p_search + p_start;
                    if let Some(p_end) = article_chunk[actual_p_start..].find("</p>") {
                        let p_text = &article_chunk[actual_p_start + 3..actual_p_start + p_end];
                        paragraphs_html.push(format!("<p>{}</p>", p_text.trim()));
                        p_search = actual_p_start + p_end + 4;
                    } else {
                        break;
                    }
                }

                let body_html = paragraphs_html.join("\n");
                let body_text = strip_html_tags(&body_html);

                if !title.is_empty() && !body_text.is_empty() {
                    fetched_items.push(NhkNewsItem {
                        id,
                        title,
                        title_with_ruby,
                        body_text,
                        body_html: sanitize_body_html(&body_html),
                        image_url,
                        audio_url,
                        published_at,
                        web_url: Some("https://www3.nhk.or.jp/news/easy/".to_string()),
                    });
                }
            } else {
                break;
            }
        }
    }

    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // If online fetch succeeded, upsert into SQLite nhk_news_cache
    if !fetched_items.is_empty() {
        let mut stmt = conn
            .prepare(
                "INSERT INTO nhk_news_cache (
                    id, title, title_with_ruby, body_text, body_html, image_url, audio_url, published_at, web_url, cached_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                ON CONFLICT(id) DO UPDATE SET
                    title = ?2,
                    title_with_ruby = ?3,
                    body_text = ?4,
                    body_html = ?5,
                    image_url = ?6,
                    audio_url = COALESCE(?7, audio_url),
                    published_at = ?8,
                    web_url = ?9,
                    cached_at = ?10",
            )
            .map_err(|e| e.to_string())?;

        for item in &fetched_items {
            let _ = stmt.execute(rusqlite::params![
                item.id,
                item.title,
                item.title_with_ruby,
                item.body_text,
                item.body_html,
                item.image_url,
                item.audio_url,
                item.published_at,
                item.web_url,
                now_ts,
            ]);
        }

        return Ok(fetched_items);
    }

    // Offline fallback: Read cached articles from nhk_news_cache
    let mut stmt = conn
        .prepare(
            "SELECT id, title, title_with_ruby, body_text, body_html, image_url, audio_url, published_at, web_url
             FROM nhk_news_cache
             ORDER BY cached_at DESC, published_at DESC
             LIMIT 30",
        )
        .map_err(|e| e.to_string())?;

    let cached_articles = stmt
        .query_map([], |row| {
            Ok(NhkNewsItem {
                id: row.get(0)?,
                title: row.get(1)?,
                title_with_ruby: row.get(2)?,
                body_text: row.get(3)?,
                body_html: row.get(4)?,
                image_url: row.get(5)?,
                audio_url: row.get(6)?,
                published_at: row.get(7)?,
                web_url: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<NhkNewsItem>, rusqlite::Error>>()
        .map_err(|e| e.to_string())?;

    if cached_articles.is_empty() {
        return Err("Nenhuma notícia disponível no momento e sem cache offline".to_string());
    }

    Ok(cached_articles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_html_tags() {
        let input = "<ruby>会社<rt>かいしゃ</rt></ruby>のコンピューター";
        let output = strip_html_tags(input);
        assert_eq!(output, "会社かいしゃのコンピューター");
    }

    #[test]
    fn test_sanitize_body_html() {
        let input = "<p>テスト</p><audio src='...'/><table class='links'></table>";
        let output = sanitize_body_html(input);
        assert_eq!(output, "<p>テスト</p>");
    }
}
