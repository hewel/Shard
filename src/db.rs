//! SQLite database module for persisting snippets.
//!
//! Supports three snippet types: Color, Code, and Text.
//! Each operation opens a fresh connection to avoid Send/Sync issues with async tasks.

use crate::snippet::{CodeData, ColorData, Snippet, SnippetContent, SnippetKind, TextData};
use rusqlite::{params, Connection, Result as SqlResult};
use std::path::PathBuf;

/// Schema version for migrations.
const SCHEMA_VERSION: i32 = 3;

/// Get the path to the database file.
pub fn get_database_path() -> Result<PathBuf, String> {
    if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "Shard") {
        let data_dir = proj_dirs.data_dir();
        Ok(data_dir.join("shard.db"))
    } else {
        // Fallback to current directory
        Ok(PathBuf::from("shard.db"))
    }
}

/// Open a connection and ensure schema exists.
fn open_connection() -> Result<Connection, String> {
    let path = get_database_path()?;

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("IO error: {}", e))?;
    }

    let conn = Connection::open(&path).map_err(|e| format!("Database error: {}", e))?;

    // Run migrations
    run_migrations(&conn)?;

    Ok(conn)
}

/// Run database migrations.
fn run_migrations(conn: &Connection) -> Result<(), String> {
    // Create meta table for tracking schema version
    conn.execute(
        "CREATE TABLE IF NOT EXISTS meta (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Meta table error: {}", e))?;

    // Get current schema version
    let current_version: i32 = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'schema_version'",
            [],
            |row| {
                let v: String = row.get(0)?;
                Ok(v.parse().unwrap_or(0))
            },
        )
        .unwrap_or(0);

    // Run migrations based on version
    if current_version < 1 {
        migrate_v1(conn)?;
    }

    if current_version < 2 {
        migrate_v2(conn)?;
    }

    if current_version < 3 {
        migrate_v3(conn)?;
    }

    // Update schema version
    conn.execute(
        "INSERT OR REPLACE INTO meta (key, value) VALUES ('schema_version', ?1)",
        params![SCHEMA_VERSION.to_string()],
    )
    .map_err(|e| format!("Version update error: {}", e))?;

    Ok(())
}

/// Migration v1: Create initial snippets table.
fn migrate_v1(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS snippets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            kind TEXT NOT NULL,
            label TEXT NOT NULL,
            position INTEGER NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP,
            
            -- Color-specific (NULL for non-color)
            r INTEGER,
            g INTEGER,
            b INTEGER,
            a REAL,
            
            -- Code-specific (NULL for non-code)
            code TEXT,
            language TEXT,
            
            -- Text-specific (NULL for non-text)
            text_content TEXT
        )",
        [],
    )
    .map_err(|e| format!("Snippets table error: {}", e))?;

    Ok(())
}

/// Migration v2: Migrate data from old colors table if exists.
fn migrate_v2(conn: &Connection) -> Result<(), String> {
    // Check if old colors table exists
    let has_colors_table: bool = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='colors'",
            [],
            |row| {
                let count: i32 = row.get(0)?;
                Ok(count > 0)
            },
        )
        .unwrap_or(false);

    if has_colors_table {
        // Migrate colors to snippets
        conn.execute(
            "INSERT INTO snippets (kind, label, position, r, g, b, a, created_at)
             SELECT 'color', label, position, r, g, b, a, created_at FROM colors
             WHERE NOT EXISTS (
                 SELECT 1 FROM snippets WHERE kind = 'color' AND r = colors.r AND g = colors.g AND b = colors.b
             )",
            [],
        )
        .map_err(|e| format!("Migration error: {}", e))?;

        // Optionally drop old table (commented out for safety)
        // conn.execute("DROP TABLE colors", []).ok();
    }

    Ok(())
}

/// Migration v3: Create palettes and palette_snippets tables.
fn migrate_v3(conn: &Connection) -> Result<(), String> {
    // Create palettes table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS palettes (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )
    .map_err(|e| format!("Palettes table error: {}", e))?;

    // Create junction table for many-to-many relationship
    conn.execute(
        "CREATE TABLE IF NOT EXISTS palette_snippets (
            palette_id INTEGER NOT NULL,
            snippet_id INTEGER NOT NULL,
            PRIMARY KEY (palette_id, snippet_id),
            FOREIGN KEY (palette_id) REFERENCES palettes(id) ON DELETE CASCADE,
            FOREIGN KEY (snippet_id) REFERENCES snippets(id) ON DELETE CASCADE
        )",
        [],
    )
    .map_err(|e| format!("Palette_snippets table error: {}", e))?;

    Ok(())
}

/// Load all snippets from the database, ordered by position (newest first).
pub fn load_snippets() -> Result<Vec<Snippet>, String> {
    let conn = open_connection()?;

    let mut stmt = conn
        .prepare(
            "SELECT id, kind, label, position, r, g, b, a, code, language, text_content 
             FROM snippets ORDER BY position DESC",
        )
        .map_err(|e| format!("Query error: {}", e))?;

    let snippets = stmt
        .query_map([], |row| {
            let id: i64 = row.get(0)?;
            let kind_str: String = row.get(1)?;
            let label: String = row.get(2)?;
            let position: i64 = row.get(3)?;

            let content = match kind_str.as_str() {
                "color" => {
                    let r: i32 = row.get(4)?;
                    let g: i32 = row.get(5)?;
                    let b: i32 = row.get(6)?;
                    let a: f64 = row.get(7)?;
                    SnippetContent::Color(ColorData::new(r as u8, g as u8, b as u8, a as f32))
                }
                "code" => {
                    let code: String = row.get(8)?;
                    let language: String = row.get(9)?;
                    SnippetContent::Code(CodeData::new(code, language))
                }
                "text" => {
                    let text: String = row.get(10)?;
                    SnippetContent::Text(TextData::new(text))
                }
                _ => {
                    // Fallback to text for unknown types
                    SnippetContent::Text(TextData::new(label.clone()))
                }
            };

            Ok(Snippet {
                id,
                label,
                content,
                position,
            })
        })
        .map_err(|e| format!("Query error: {}", e))?
        .collect::<SqlResult<Vec<_>>>()
        .map_err(|e| format!("Query error: {}", e))?;

    Ok(snippets)
}

/// Load snippets filtered by kind.
#[allow(dead_code)]
pub fn load_snippets_by_kind(kind: SnippetKind) -> Result<Vec<Snippet>, String> {
    let conn = open_connection()?;

    let mut stmt = conn
        .prepare(
            "SELECT id, kind, label, position, r, g, b, a, code, language, text_content 
             FROM snippets WHERE kind = ?1 ORDER BY position DESC",
        )
        .map_err(|e| format!("Query error: {}", e))?;

    let snippets = stmt
        .query_map(params![kind.as_db_str()], |row| {
            let id: i64 = row.get(0)?;
            let kind_str: String = row.get(1)?;
            let label: String = row.get(2)?;
            let position: i64 = row.get(3)?;

            let content = match kind_str.as_str() {
                "color" => {
                    let r: i32 = row.get(4)?;
                    let g: i32 = row.get(5)?;
                    let b: i32 = row.get(6)?;
                    let a: f64 = row.get(7)?;
                    SnippetContent::Color(ColorData::new(r as u8, g as u8, b as u8, a as f32))
                }
                "code" => {
                    let code: String = row.get(8)?;
                    let language: String = row.get(9)?;
                    SnippetContent::Code(CodeData::new(code, language))
                }
                "text" => {
                    let text: String = row.get(10)?;
                    SnippetContent::Text(TextData::new(text))
                }
                _ => SnippetContent::Text(TextData::new(label.clone())),
            };

            Ok(Snippet {
                id,
                label,
                content,
                position,
            })
        })
        .map_err(|e| format!("Query error: {}", e))?
        .collect::<SqlResult<Vec<_>>>()
        .map_err(|e| format!("Query error: {}", e))?;

    Ok(snippets)
}

/// Get the next position value.
fn get_next_position(conn: &Connection) -> Result<i64, String> {
    let max_position: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(position), 0) FROM snippets",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("Query error: {}", e))?;

    Ok(max_position + 1)
}

/// Insert a new snippet and return the snippet with its new ID.
pub fn insert_snippet(snippet: Snippet) -> Result<Snippet, String> {
    let conn = open_connection()?;
    let position = get_next_position(&conn)?;

    match &snippet.content {
        SnippetContent::Color(color) => {
            conn.execute(
                "INSERT INTO snippets (kind, label, position, r, g, b, a) 
                 VALUES ('color', ?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    snippet.label,
                    position,
                    color.r as i32,
                    color.g as i32,
                    color.b as i32,
                    color.a
                ],
            )
            .map_err(|e| format!("Insert error: {}", e))?;
        }
        SnippetContent::Code(code) => {
            conn.execute(
                "INSERT INTO snippets (kind, label, position, code, language) 
                 VALUES ('code', ?1, ?2, ?3, ?4)",
                params![snippet.label, position, code.code, code.language],
            )
            .map_err(|e| format!("Insert error: {}", e))?;
        }
        SnippetContent::Text(text) => {
            conn.execute(
                "INSERT INTO snippets (kind, label, position, text_content) 
                 VALUES ('text', ?1, ?2, ?3)",
                params![snippet.label, position, text.text],
            )
            .map_err(|e| format!("Insert error: {}", e))?;
        }
    }

    let id = conn.last_insert_rowid();
    Ok(Snippet {
        id,
        position,
        ..snippet
    })
}

/// Update a snippet's label.
#[allow(dead_code)]
pub fn update_label(id: i64, label: String) -> Result<(i64, String), String> {
    let conn = open_connection()?;
    conn.execute(
        "UPDATE snippets SET label = ?1 WHERE id = ?2",
        params![label, id],
    )
    .map_err(|e| format!("Update error: {}", e))?;
    Ok((id, label))
}

/// Update a color snippet.
pub fn update_color(
    id: i64,
    r: u8,
    g: u8,
    b: u8,
    a: f32,
    label: String,
) -> Result<Snippet, String> {
    let conn = open_connection()?;
    conn.execute(
        "UPDATE snippets SET r = ?1, g = ?2, b = ?3, a = ?4, label = ?5 WHERE id = ?6 AND kind = 'color'",
        params![r as i32, g as i32, b as i32, a, label, id],
    )
    .map_err(|e| format!("Update error: {}", e))?;

    // Get the updated snippet
    get_snippet_by_id(id)?.ok_or_else(|| "Snippet not found".to_string())
}

/// Update a code snippet.
pub fn update_code(
    id: i64,
    code: String,
    language: String,
    label: String,
) -> Result<Snippet, String> {
    let conn = open_connection()?;
    conn.execute(
        "UPDATE snippets SET code = ?1, language = ?2, label = ?3 WHERE id = ?4 AND kind = 'code'",
        params![code, language, label, id],
    )
    .map_err(|e| format!("Update error: {}", e))?;

    get_snippet_by_id(id)?.ok_or_else(|| "Snippet not found".to_string())
}

/// Update only the code content of a code snippet (preserves language and label).
pub fn update_code_content(id: i64, code: String) -> Result<Snippet, String> {
    let conn = open_connection()?;
    conn.execute(
        "UPDATE snippets SET code = ?1 WHERE id = ?2 AND kind = 'code'",
        params![code, id],
    )
    .map_err(|e| format!("Update error: {}", e))?;

    get_snippet_by_id(id)?.ok_or_else(|| "Snippet not found".to_string())
}

/// Update only the text content of a text snippet (preserves label).
pub fn update_text_content(id: i64, text: String) -> Result<Snippet, String> {
    let conn = open_connection()?;
    conn.execute(
        "UPDATE snippets SET text_content = ?1 WHERE id = ?2 AND kind = 'text'",
        params![text, id],
    )
    .map_err(|e| format!("Update error: {}", e))?;

    get_snippet_by_id(id)?.ok_or_else(|| "Snippet not found".to_string())
}

/// Update a text snippet.
pub fn update_text(id: i64, text: String, label: String) -> Result<Snippet, String> {
    let conn = open_connection()?;
    conn.execute(
        "UPDATE snippets SET text_content = ?1, label = ?2 WHERE id = ?3 AND kind = 'text'",
        params![text, label, id],
    )
    .map_err(|e| format!("Update error: {}", e))?;

    get_snippet_by_id(id)?.ok_or_else(|| "Snippet not found".to_string())
}

/// Get a snippet by ID.
pub fn get_snippet_by_id(id: i64) -> Result<Option<Snippet>, String> {
    let conn = open_connection()?;

    let result = conn.query_row(
        "SELECT id, kind, label, position, r, g, b, a, code, language, text_content 
         FROM snippets WHERE id = ?1",
        params![id],
        |row| {
            let id: i64 = row.get(0)?;
            let kind_str: String = row.get(1)?;
            let label: String = row.get(2)?;
            let position: i64 = row.get(3)?;

            let content = match kind_str.as_str() {
                "color" => {
                    let r: i32 = row.get(4)?;
                    let g: i32 = row.get(5)?;
                    let b: i32 = row.get(6)?;
                    let a: f64 = row.get(7)?;
                    SnippetContent::Color(ColorData::new(r as u8, g as u8, b as u8, a as f32))
                }
                "code" => {
                    let code: String = row.get(8)?;
                    let language: String = row.get(9)?;
                    SnippetContent::Code(CodeData::new(code, language))
                }
                "text" => {
                    let text: String = row.get(10)?;
                    SnippetContent::Text(TextData::new(text))
                }
                _ => SnippetContent::Text(TextData::new(label.clone())),
            };

            Ok(Snippet {
                id,
                label,
                content,
                position,
            })
        },
    );

    match result {
        Ok(snippet) => Ok(Some(snippet)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Query error: {}", e)),
    }
}

/// Delete a snippet by ID.
pub fn delete_snippet(id: i64) -> Result<i64, String> {
    let conn = open_connection()?;
    conn.execute("DELETE FROM snippets WHERE id = ?1", params![id])
        .map_err(|e| format!("Delete error: {}", e))?;
    Ok(id)
}

/// Move a snippet to the top (highest position).
pub fn move_to_top(id: i64) -> Result<(), String> {
    let conn = open_connection()?;
    let next_pos = get_next_position(&conn)?;

    conn.execute(
        "UPDATE snippets SET position = ?1 WHERE id = ?2",
        params![next_pos, id],
    )
    .map_err(|e| format!("Update error: {}", e))?;

    Ok(())
}

/// Find a color snippet by its RGBA values. Returns the ID if found.
pub fn find_color_by_rgba(r: u8, g: u8, b: u8, a: f32) -> Result<Option<i64>, String> {
    let conn = open_connection()?;

    let result: SqlResult<i64> = conn.query_row(
        "SELECT id FROM snippets WHERE kind = 'color' AND r = ?1 AND g = ?2 AND b = ?3 AND ABS(a - ?4) < 0.01",
        params![r as i32, g as i32, b as i32, a],
        |row| row.get(0),
    );

    match result {
        Ok(id) => Ok(Some(id)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Query error: {}", e)),
    }
}

/// Add a color snippet, handling duplicates by moving existing to top.
pub fn add_or_move_color(r: u8, g: u8, b: u8, a: f32, label: String) -> Result<Snippet, String> {
    // Check for duplicate
    if let Some(existing_id) = find_color_by_rgba(r, g, b, a)? {
        // Move existing to top
        move_to_top(existing_id)?;
        // Return the existing snippet
        get_snippet_by_id(existing_id)?.ok_or_else(|| "Snippet not found".to_string())
    } else {
        // Insert new color (Snippet::color handles empty label with nanoid)
        let label = if label.is_empty() {
            nanoid::nanoid!(8)
        } else {
            label
        };
        insert_snippet(Snippet::color(r, g, b, a, label))
    }
}

/// Add a code snippet.
pub fn add_code_snippet(code: String, language: String, label: String) -> Result<Snippet, String> {
    insert_snippet(Snippet::code(code, language, label))
}

/// Add a text snippet.
pub fn add_text_snippet(text: String, label: String) -> Result<Snippet, String> {
    insert_snippet(Snippet::text(text, label))
}

// ============================================================================
// Palette Operations
// ============================================================================

/// A palette for organizing snippets.
#[derive(Debug, Clone, PartialEq)]
pub struct Palette {
    pub id: i64,
    pub name: String,
}

/// Load all palettes from the database.
pub fn load_palettes() -> Result<Vec<Palette>, String> {
    let conn = open_connection()?;

    let mut stmt = conn
        .prepare("SELECT id, name FROM palettes ORDER BY name ASC")
        .map_err(|e| format!("Query error: {}", e))?;

    let palettes = stmt
        .query_map([], |row| {
            Ok(Palette {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })
        .map_err(|e| format!("Query error: {}", e))?
        .collect::<SqlResult<Vec<_>>>()
        .map_err(|e| format!("Query error: {}", e))?;

    Ok(palettes)
}

/// Create a new palette.
pub fn create_palette(name: String) -> Result<Palette, String> {
    let conn = open_connection()?;

    conn.execute("INSERT INTO palettes (name) VALUES (?1)", params![name])
        .map_err(|e| format!("Insert error: {}", e))?;

    let id = conn.last_insert_rowid();
    Ok(Palette { id, name })
}

/// Rename a palette.
pub fn rename_palette(id: i64, new_name: String) -> Result<Palette, String> {
    let conn = open_connection()?;

    conn.execute(
        "UPDATE palettes SET name = ?1 WHERE id = ?2",
        params![new_name, id],
    )
    .map_err(|e| format!("Update error: {}", e))?;

    Ok(Palette { id, name: new_name })
}

/// Delete a palette (cascade deletes palette_snippets entries).
pub fn delete_palette(id: i64) -> Result<i64, String> {
    let conn = open_connection()?;

    // Delete palette_snippets entries first (in case FK cascade isn't working)
    conn.execute(
        "DELETE FROM palette_snippets WHERE palette_id = ?1",
        params![id],
    )
    .map_err(|e| format!("Delete error: {}", e))?;

    conn.execute("DELETE FROM palettes WHERE id = ?1", params![id])
        .map_err(|e| format!("Delete error: {}", e))?;

    Ok(id)
}

/// Add a snippet to a palette.
pub fn add_snippet_to_palette(palette_id: i64, snippet_id: i64) -> Result<(), String> {
    let conn = open_connection()?;

    conn.execute(
        "INSERT OR IGNORE INTO palette_snippets (palette_id, snippet_id) VALUES (?1, ?2)",
        params![palette_id, snippet_id],
    )
    .map_err(|e| format!("Insert error: {}", e))?;

    Ok(())
}

/// Remove a snippet from a palette.
pub fn remove_snippet_from_palette(palette_id: i64, snippet_id: i64) -> Result<(), String> {
    let conn = open_connection()?;

    conn.execute(
        "DELETE FROM palette_snippets WHERE palette_id = ?1 AND snippet_id = ?2",
        params![palette_id, snippet_id],
    )
    .map_err(|e| format!("Delete error: {}", e))?;

    Ok(())
}

/// Get all palette IDs for a snippet.
pub fn get_palettes_for_snippet(snippet_id: i64) -> Result<Vec<i64>, String> {
    let conn = open_connection()?;

    let mut stmt = conn
        .prepare("SELECT palette_id FROM palette_snippets WHERE snippet_id = ?1")
        .map_err(|e| format!("Query error: {}", e))?;

    let palette_ids = stmt
        .query_map(params![snippet_id], |row| row.get(0))
        .map_err(|e| format!("Query error: {}", e))?
        .collect::<SqlResult<Vec<i64>>>()
        .map_err(|e| format!("Query error: {}", e))?;

    Ok(palette_ids)
}

/// Get all snippet IDs in a palette.
#[allow(dead_code)]
pub fn get_snippets_in_palette(palette_id: i64) -> Result<Vec<i64>, String> {
    let conn = open_connection()?;

    let mut stmt = conn
        .prepare("SELECT snippet_id FROM palette_snippets WHERE palette_id = ?1")
        .map_err(|e| format!("Query error: {}", e))?;

    let snippet_ids = stmt
        .query_map(params![palette_id], |row| row.get(0))
        .map_err(|e| format!("Query error: {}", e))?
        .collect::<SqlResult<Vec<i64>>>()
        .map_err(|e| format!("Query error: {}", e))?;

    Ok(snippet_ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snippet_kind_db_strings() {
        assert_eq!(SnippetKind::Color.as_db_str(), "color");
        assert_eq!(SnippetKind::Code.as_db_str(), "code");
        assert_eq!(SnippetKind::Text.as_db_str(), "text");
    }
}
