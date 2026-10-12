CREATE TABLE IF NOT EXISTS dictionary_entries (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            kanji TEXT,
            reading TEXT NOT NULL,
            meaning TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_dictionary_kanji ON dictionary_entries(kanji);
        CREATE INDEX IF NOT EXISTS idx_dictionary_reading ON dictionary_entries(reading);

        CREATE TABLE IF NOT EXISTS decks (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT
        );

        CREATE TABLE IF NOT EXISTS tags (
            id TEXT PRIMARY KEY,
            name TEXT,
            color TEXT,
            category TEXT,
            notes TEXT
        );

        CREATE TABLE IF NOT EXISTS cards (
            id TEXT PRIMARY KEY,
            entry_id INTEGER NOT NULL,
            deck_id TEXT,
            due_date INTEGER NOT NULL,
            stability REAL NOT NULL,
            difficulty REAL NOT NULL,
            reps INTEGER NOT NULL,
            state INTEGER NOT NULL,
            FOREIGN KEY(entry_id) REFERENCES dictionary_entries(id) ON DELETE CASCADE,
            FOREIGN KEY(deck_id) REFERENCES decks(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_cards_entry_id ON cards(entry_id);

        CREATE TABLE IF NOT EXISTS card_tags (
            card_id TEXT NOT NULL,
            tag_id TEXT NOT NULL,
            PRIMARY KEY (card_id, tag_id),
            FOREIGN KEY(card_id) REFERENCES cards(id) ON DELETE CASCADE,
            FOREIGN KEY(tag_id) REFERENCES tags(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS review_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            card_id TEXT NOT NULL,
            rating INTEGER NOT NULL,
            review_datetime INTEGER NOT NULL,
            FOREIGN KEY(card_id) REFERENCES cards(id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS idx_review_logs_card_id ON review_logs(card_id);

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT
        );

        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT
        );

        CREATE TABLE IF NOT EXISTS kanjis (
            character TEXT PRIMARY KEY,
            onyomi TEXT,
            kunyomi TEXT,
            meaning TEXT,
            jlpt_level INTEGER
        );

        CREATE TABLE IF NOT EXISTS kanji_strokes (
            kanji TEXT PRIMARY KEY,
            svg_data TEXT
        );

        CREATE TABLE IF NOT EXISTS word_frequencies (
            term TEXT PRIMARY KEY,
            rank INTEGER
        );

        CREATE TABLE IF NOT EXISTS kanji_components (
            kanji TEXT NOT NULL,
            component TEXT NOT NULL,
            meaning TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_kanji_components_kanji ON kanji_components(kanji);

        CREATE TABLE IF NOT EXISTS user_mnemonics (
            kanji TEXT PRIMARY KEY,
            mnemonic TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS notebook_entries (
            id TEXT PRIMARY KEY,
            title TEXT,
            content_md TEXT,
            created_at DATETIME,
            updated_at DATETIME,
            deck_id TEXT,
            next_review_date DATETIME,
            srs_stage INTEGER DEFAULT 0,
            FOREIGN KEY(deck_id) REFERENCES decks(id) ON DELETE SET NULL
        );

        CREATE TABLE IF NOT EXISTS kana_mastery (
            character TEXT PRIMARY KEY,
            romaji TEXT NOT NULL,
            kana_type TEXT NOT NULL,
            alphabet TEXT,
            category TEXT,
            correct_count INTEGER DEFAULT 0,
            wrong_count INTEGER DEFAULT 0,
            total_attempts INTEGER DEFAULT 0,
            correct_attempts INTEGER DEFAULT 0,
            accuracy_percentage REAL DEFAULT 0.0,
            last_practiced INTEGER DEFAULT 0
        );

        CREATE TABLE IF NOT EXISTS nhk_news_cache (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            title_with_ruby TEXT,
            body_text TEXT NOT NULL,
            body_html TEXT NOT NULL,
            image_url TEXT,
            audio_url TEXT,
            published_at TEXT,
            web_url TEXT,
            cached_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS epub_reading_positions (
            book_hash TEXT PRIMARY KEY,
            cfi TEXT NOT NULL,
            updated_at INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS badges (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            icon TEXT NOT NULL,
            category TEXT NOT NULL,
            target_metric TEXT NOT NULL,
            target_value INTEGER NOT NULL
        );

        CREATE TABLE IF NOT EXISTS user_badges (
            badge_id TEXT PRIMARY KEY,
            unlocked_at INTEGER NOT NULL,
            FOREIGN KEY(badge_id) REFERENCES badges(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS study_sessions (
            id TEXT PRIMARY KEY,
            category TEXT NOT NULL,
            duration_minutes INTEGER NOT NULL CHECK (duration_minutes > 0),
            date TEXT NOT NULL DEFAULT (datetime('now', 'localtime'))
        );

        CREATE TABLE IF NOT EXISTS image_occlusion_cards (
            id TEXT PRIMARY KEY,
            deck_id TEXT NOT NULL DEFAULT 'default',
            image_path TEXT NOT NULL,
            masks_json TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now', 'localtime')),
            due_date INTEGER NOT NULL DEFAULT 0,
            stability REAL NOT NULL DEFAULT 0.0,
            difficulty REAL NOT NULL DEFAULT 0.0,
            reps INTEGER NOT NULL DEFAULT 0,
            state INTEGER NOT NULL DEFAULT 0,
            last_review INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY(deck_id) REFERENCES decks(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS image_occlusion_review_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            card_id TEXT NOT NULL,
            rating INTEGER NOT NULL,
            review_datetime INTEGER NOT NULL,
            FOREIGN KEY(card_id) REFERENCES image_occlusion_cards(id) ON DELETE CASCADE
        );

