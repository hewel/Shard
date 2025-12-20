//! SQLite database module for persisting colors.
//!
//! Each operation opens a fresh connection to avoid Send/Sync issues with async tasks.

use crate::color::Color;
use rusqlite::{params, Connection, Result as SqlResult};
use std::path::PathBuf;

/// Get the path to the database file.
pub fn get_database_path() -> Result<PathBuf, String> {
    if let Some(proj_dirs) = directories::ProjectDirs::from("", "", "Shard") {
        let data_dir = proj_dirs.data_dir();
        Ok(data_dir.join("colors.db"))
    } else {
        // Fallback to current directory
        Ok(PathBuf::from("shard_colors.db"))
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

    // Initialize schema
    conn.execute(
        "CREATE TABLE IF NOT EXISTS colors (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            r INTEGER NOT NULL,
            g INTEGER NOT NULL,
            b INTEGER NOT NULL,
            a REAL NOT NULL,
            label TEXT NOT NULL,
            position INTEGER NOT NULL,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )
    .map_err(|e| format!("Schema error: {}", e))?;

    Ok(conn)
}

/// Load all colors from the database, ordered by position (newest first).
pub fn load_colors() -> Result<Vec<Color>, String> {
    let conn = open_connection()?;

    let mut stmt = conn
        .prepare("SELECT id, r, g, b, a, label FROM colors ORDER BY position DESC")
        .map_err(|e| format!("Query error: {}", e))?;

    let colors = stmt
        .query_map([], |row| {
            Ok(Color {
                id: row.get(0)?,
                r: row.get::<_, i32>(1)? as u8,
                g: row.get::<_, i32>(2)? as u8,
                b: row.get::<_, i32>(3)? as u8,
                a: row.get(4)?,
                label: row.get(5)?,
            })
        })
        .map_err(|e| format!("Query error: {}", e))?
        .collect::<SqlResult<Vec<_>>>()
        .map_err(|e| format!("Query error: {}", e))?;

    Ok(colors)
}

/// Insert a new color and return the color with its new ID.
pub fn insert_color(color: Color) -> Result<Color, String> {
    let conn = open_connection()?;

    // Get the next position (highest + 1)
    let max_position: i64 = conn
        .query_row("SELECT COALESCE(MAX(position), 0) FROM colors", [], |row| {
            row.get(0)
        })
        .map_err(|e| format!("Query error: {}", e))?;

    conn.execute(
        "INSERT INTO colors (r, g, b, a, label, position) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            color.r as i32,
            color.g as i32,
            color.b as i32,
            color.a,
            color.label,
            max_position + 1
        ],
    )
    .map_err(|e| format!("Insert error: {}", e))?;

    let id = conn.last_insert_rowid();
    Ok(Color { id, ..color })
}

/// Update a color's label.
pub fn update_label(id: i64, label: String) -> Result<(i64, String), String> {
    let conn = open_connection()?;
    conn.execute(
        "UPDATE colors SET label = ?1 WHERE id = ?2",
        params![label, id],
    )
    .map_err(|e| format!("Update error: {}", e))?;
    Ok((id, label))
}

/// Update a color's RGBA values and label.
pub fn update_color(id: i64, r: u8, g: u8, b: u8, a: f32, label: String) -> Result<Color, String> {
    let conn = open_connection()?;
    conn.execute(
        "UPDATE colors SET r = ?1, g = ?2, b = ?3, a = ?4, label = ?5 WHERE id = ?6",
        params![r as i32, g as i32, b as i32, a, label, id],
    )
    .map_err(|e| format!("Update error: {}", e))?;

    Ok(Color {
        id,
        r,
        g,
        b,
        a,
        label,
    })
}

/// Delete a color by ID.
pub fn delete_color(id: i64) -> Result<i64, String> {
    let conn = open_connection()?;
    conn.execute("DELETE FROM colors WHERE id = ?1", params![id])
        .map_err(|e| format!("Delete error: {}", e))?;
    Ok(id)
}

/// Move a color to the top (highest position).
pub fn move_to_top(id: i64) -> Result<(), String> {
    let conn = open_connection()?;

    let max_position: i64 = conn
        .query_row("SELECT COALESCE(MAX(position), 0) FROM colors", [], |row| {
            row.get(0)
        })
        .map_err(|e| format!("Query error: {}", e))?;

    conn.execute(
        "UPDATE colors SET position = ?1 WHERE id = ?2",
        params![max_position + 1, id],
    )
    .map_err(|e| format!("Update error: {}", e))?;

    Ok(())
}

/// Find a color by its RGBA values. Returns the ID if found.
pub fn find_by_rgba(r: u8, g: u8, b: u8, a: f32) -> Result<Option<i64>, String> {
    let conn = open_connection()?;

    let result: SqlResult<i64> = conn.query_row(
        "SELECT id FROM colors WHERE r = ?1 AND g = ?2 AND b = ?3 AND ABS(a - ?4) < 0.01",
        params![r as i32, g as i32, b as i32, a],
        |row| row.get(0),
    );

    match result {
        Ok(id) => Ok(Some(id)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Query error: {}", e)),
    }
}

/// Add a color, handling duplicates by moving existing to top.
/// Returns the color (either newly inserted or existing moved to top).
pub fn add_or_move_color(mut color: Color) -> Result<Color, String> {
    // Check for duplicate
    if let Some(existing_id) = find_by_rgba(color.r, color.g, color.b, color.a)? {
        // Move existing to top
        move_to_top(existing_id)?;
        // Reload colors to get the one at top
        let colors = load_colors()?;
        colors
            .into_iter()
            .next()
            .ok_or_else(|| "Color not found".to_string())
    } else {
        // Insert new color
        if color.label.is_empty() {
            color.label = color.default_label();
        }
        insert_color(color)
    }
}
