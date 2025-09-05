use anyhow::Result;
use learning_system::database::Database;
use regex::Regex;
use sqlx::Row;
use std::env;
use uuid::Uuid;

#[derive(Debug)]
struct MigrationStats {
    total_cards: usize,
    cards_with_headers: usize,
    cards_updated: usize,
    errors: Vec<String>,
}

impl MigrationStats {
    fn new() -> Self {
        Self {
            total_cards: 0,
            cards_with_headers: 0,
            cards_updated: 0,
            errors: Vec::new(),
        }
    }

    fn print_summary(&self, dry_run: bool) {
        println!("\n=== Migration Summary ===");
        println!("Total cards examined: {}", self.total_cards);
        println!("Cards with markdown headers: {}", self.cards_with_headers);
        
        if dry_run {
            println!("Cards that WOULD BE updated: {}", self.cards_with_headers);
            println!("\n** DRY RUN MODE - No changes were made **");
        } else {
            println!("Cards successfully updated: {}", self.cards_updated);
            if !self.errors.is_empty() {
                println!("Errors encountered: {}", self.errors.len());
                for error in &self.errors {
                    println!("  - {}", error);
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct CardUpdate {
    id: Uuid,
    zettel_id: String,
    original_content: String,
    extracted_title: String,
    new_content: String,
}

impl CardUpdate {
    fn print_preview(&self) {
        println!("\nCard: {} ({})", self.zettel_id, self.id);
        println!("  Title: \"{}\"", self.extracted_title);
        println!("  Content before: \"{}...\"", 
            self.original_content.chars().take(50).collect::<String>()
        );
        println!("  Content after:  \"{}...\"", 
            self.new_content.chars().take(50).collect::<String>()
        );
    }
}

fn extract_markdown_header(content: &str) -> Option<(String, String)> {
    let lines: Vec<&str> = content.lines().collect();
    
    if lines.is_empty() {
        return None;
    }
    
    let first_line = lines[0].trim();
    
    // Regex to match markdown headers (# Header, ## Header, etc.)
    let header_regex = Regex::new(r"^(#{1,6})\s+(.+)$").unwrap();
    
    if let Some(captures) = header_regex.captures(first_line) {
        let title = captures.get(2)?.as_str().trim().to_string();
        
        if title.is_empty() {
            return None;
        }
        
        // Remove the header line from content
        let remaining_lines = if lines.len() > 1 {
            lines[1..].join("\n").trim_start().to_string()
        } else {
            String::new()
        };
        
        Some((title, remaining_lines))
    } else {
        None
    }
}

async fn find_cards_with_headers(db: &Database) -> Result<Vec<CardUpdate>> {
    let pool = &db.pool;
    
    // Query cards that don't have titles and have content
    let rows = sqlx::query(
        "SELECT id, zettel_id, content FROM cards WHERE (title IS NULL OR title = '') AND content != ''"
    )
    .fetch_all(pool)
    .await?;

    let mut updates = Vec::new();
    
    for row in rows {
        let id_str: String = row.get("id");
        let id = Uuid::parse_str(&id_str)?;
        let zettel_id: String = row.get("zettel_id");
        let content: String = row.get("content");
        
        if let Some((title, new_content)) = extract_markdown_header(&content) {
            updates.push(CardUpdate {
                id,
                zettel_id,
                original_content: content,
                extracted_title: title,
                new_content,
            });
        }
    }
    
    Ok(updates)
}

async fn apply_updates(db: &Database, updates: &[CardUpdate]) -> Result<MigrationStats> {
    let mut stats = MigrationStats::new();
    let pool = &db.pool;
    
    // Start a transaction for all updates
    let mut tx = pool.begin().await?;
    
    for update in updates {
        match sqlx::query(
            "UPDATE cards SET title = ?1, content = ?2 WHERE id = ?3"
        )
        .bind(&update.extracted_title)
        .bind(&update.new_content)
        .bind(update.id.to_string())
        .execute(&mut *tx)
        .await
        {
            Ok(result) => {
                if result.rows_affected() > 0 {
                    stats.cards_updated += 1;
                    println!("✓ Updated card {} with title: \"{}\"", 
                        update.zettel_id, update.extracted_title);
                } else {
                    stats.errors.push(format!("No rows affected for card {}", update.zettel_id));
                }
            }
            Err(e) => {
                stats.errors.push(format!("Failed to update card {}: {}", update.zettel_id, e));
            }
        }
    }
    
    if stats.errors.is_empty() {
        tx.commit().await?;
        println!("\n✓ All updates committed successfully!");
    } else {
        tx.rollback().await?;
        println!("\n✗ Transaction rolled back due to errors!");
    }
    
    Ok(stats)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();
    
    let args: Vec<String> = env::args().collect();
    let dry_run = args.contains(&"--dry-run".to_string());
    
    println!("=== Card Title Migration Tool ===");
    if dry_run {
        println!("** RUNNING IN DRY-RUN MODE **");
        println!("This will show what would be changed without making any updates.");
    } else {
        println!("** LIVE MODE - Changes will be made to the database **");
        println!("Make sure you have backed up your database before proceeding!");
    }
    
    // Connect to database
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:./learning_system.db".to_string());
    
    println!("\nConnecting to database: {}", database_url);
    let db = Database::new(&database_url).await?;
    
    // Find all cards that could be updated
    println!("\nScanning for cards with markdown headers...");
    let updates = find_cards_with_headers(&db).await?;
    
    let mut stats = MigrationStats::new();
    stats.total_cards = sqlx::query("SELECT COUNT(*) as count FROM cards")
        .fetch_one(&db.pool)
        .await?
        .get::<i64, _>("count") as usize;
    
    stats.cards_with_headers = updates.len();
    
    if updates.is_empty() {
        println!("\n✓ No cards found with markdown headers that need title extraction.");
        stats.print_summary(dry_run);
        return Ok(());
    }
    
    println!("\nFound {} cards with markdown headers:", updates.len());
    
    // Show preview of changes
    for (i, update) in updates.iter().enumerate() {
        if i < 5 || dry_run {  // Show first 5 in live mode, all in dry-run
            update.print_preview();
        } else if i == 5 {
            println!("\n... and {} more cards", updates.len() - 5);
            break;
        }
    }
    
    if dry_run {
        stats.print_summary(true);
        println!("\nTo perform the actual migration, run:");
        println!("cargo run --bin migrate_titles");
        return Ok(());
    }
    
    // Confirm before proceeding
    println!("\nProceed with updating {} cards? (y/N): ", updates.len());
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    if input.trim().to_lowercase() != "y" {
        println!("Migration cancelled.");
        return Ok(());
    }
    
    // Apply the updates
    println!("\nApplying updates...");
    stats = apply_updates(&db, &updates).await?;
    stats.total_cards = sqlx::query("SELECT COUNT(*) as count FROM cards")
        .fetch_one(&db.pool)
        .await?
        .get::<i64, _>("count") as usize;
    stats.cards_with_headers = updates.len();
    
    stats.print_summary(false);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_markdown_header() {
        // Test basic header extraction
        let content = "# This is a title\n\nThis is the content.";
        let result = extract_markdown_header(content);
        assert!(result.is_some());
        let (title, new_content) = result.unwrap();
        assert_eq!(title, "This is a title");
        assert_eq!(new_content, "This is the content.");
        
        // Test with different header levels
        let content = "## Section Header\nContent here";
        let result = extract_markdown_header(content);
        assert!(result.is_some());
        let (title, new_content) = result.unwrap();
        assert_eq!(title, "Section Header");
        assert_eq!(new_content, "Content here");
        
        // Test with no header
        let content = "Just regular content\nNo header here";
        let result = extract_markdown_header(content);
        assert!(result.is_none());
        
        // Test with empty header
        let content = "# \nContent";
        let result = extract_markdown_header(content);
        assert!(result.is_none());
        
        // Test with only header
        let content = "# Just a title";
        let result = extract_markdown_header(content);
        assert!(result.is_some());
        let (title, new_content) = result.unwrap();
        assert_eq!(title, "Just a title");
        assert_eq!(new_content, "");
    }
}