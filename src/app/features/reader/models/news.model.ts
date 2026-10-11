export interface NhkNewsItem {
  id: string
  title: string
  title_with_ruby: string
  body: string
  body_html: string
  image_url: string | null
  news_web_url: string | null
  published_at: string | null
}
