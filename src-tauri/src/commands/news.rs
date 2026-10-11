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
pub async fn fetch_nhk_easy_news() -> Result<Vec<NhkNewsItem>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(12))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

    // Primary source: nhkeasier.com (which mirrors and cleans NHK News Web Easy articles daily)
    let resp = client
        .get("https://nhkeasier.com/")
        .send()
        .await
        .map_err(|e| format!("Network error fetching NHK news: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!(
            "NHK news server responded with status: {}",
            resp.status()
        ));
    }

    let html = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response body: {}", e))?;

    let mut items = Vec::new();

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

            // Extract published_at date from <time datetime="...">
            let published_at = if let Some(time_start) = article_chunk.find("<time datetime=\"") {
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
                    format!("story-{}", items.len() + 1)
                }
            } else {
                format!("story-{}", items.len() + 1)
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
                items.push(NhkNewsItem {
                    id,
                    title,
                    title_with_ruby,
                    body_text,
                    body_html: sanitize_body_html(&body_html),
                    image_url,
                    audio_url: None,
                    published_at,
                    web_url: Some("https://www3.nhk.or.jp/news/easy/".to_string()),
                });
            }
        } else {
            break;
        }
    }

    Ok(items)
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
