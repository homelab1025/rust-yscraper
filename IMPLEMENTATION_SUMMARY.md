# Comment Refreshing with Time Limits and Frequency - Implementation Summary

## 🎯 Feature Overview

Successfully implemented automatic comment refreshing with configurable time limits and frequency. The system now supports:

- **Configurable default values** for days limit and frequency hours
- **Request validation** to prevent invalid (zero/negative) values  
- **Automatic background refresh** based on scheduling metadata
- **Time-based expiry** - stops refreshing after X days from initial scrape

## 📝 Files Modified

### Core Implementation

#### `web_server/src/config.rs`
- ✅ Added `default_days_limit: u32` and `default_frequency_hours: u32` to `AppConfig`
- ✅ Updated `from_config()` to read from config with defaults (7 days, 24 hours)

#### `web_server/src/db/comments_repository.rs`  
- ✅ Updated `upsert_url_with_scheduling()` signature to return `Result<(), sqlx::Error>`
- ✅ Added new trait methods: `upsert_url_with_scheduling()`, `get_urls_due_for_refresh()`, `update_last_scraped()`
- ✅ Added `DbScheduledUrlRow` struct for scheduling queries

#### `web_server/src/db/postgresql.rs`
- ✅ Implemented scheduling methods with proper SQL
- ✅ Fixed database logic: only updates `url` on conflict, preserves scheduling metadata
- ✅ Removed incorrect scheduling validation from database layer

#### `web_server/src/background_scheduler.rs`
- ✅ Created comprehensive background scheduler
- ✅ Runs every minute checking for due URLs
- ✅ Updates `last_scraped` timestamps to prevent duplicate scheduling
- ✅ Includes comprehensive test coverage

#### `web_server/src/api/comments.rs`
- ✅ Added validation function for `days_limit` and `frequency_hours` > 0
- ✅ Updated `ScrapeRequest` with optional `days_limit` and `frequency_hours`
- ✅ Simplified `scrape_comments()` - always schedules initial scrape
- ✅ Updated `CommentsAppState` with default values
- ✅ Added validation tests

#### `web_server/src/api/app_state.rs`
- ✅ Added `config: AppConfig` field to `AppState`
- ✅ Updated state management to pass config through

#### `web_server/src/main.rs`
- ✅ Updated application initialization to pass config to state
- ✅ Integrated background scheduler startup

#### `conf/config.toml`
- ✅ Added configurable defaults:
  ```toml
  default_days_limit=7
  default_frequency_hours=24
  ```

### Database Schema

#### `db/changelog/002_add_scheduling.sql`
- ✅ Added scheduling metadata columns to `urls` table:
  - `last_scraped TIMESTAMPTZ` - when URL was last scraped
  - `frequency_hours INTEGER` - hours between refresh attempts  
  - `days_limit INTEGER` - days to continue refreshing from initial scrape
- ✅ Added indexes for efficient scheduling queries

#### `db/changelog/db.changelog-master.yaml`
- ✅ Added new migration to change set

## 🔄 Corrected Logic

### Before (Flawed):
- ❌ DB layer decided whether to schedule based on `days_limit`
- ❌ Always updated scheduling metadata on conflict
- ❌ `days_limit` was checked against insertion time

### After (Correct):
- ✅ DB layer only stores data, no scheduling decisions
- ✅ Only updates `url` field on conflict, preserves original scheduling
- ✅ Background scheduler handles time-based refresh logic
- ✅ `days_limit` correctly counted from initial scrape time

## 🧪 Testing

### Validation Tests
```rust
// ✅ Rejects zero days_limit
// ✅ Rejects zero frequency_hours  
// ✅ Accepts valid requests with defaults
// ✅ Accepts valid requests with custom values
```

### Integration Tests
- ✅ All existing tests pass (37/37)
- ✅ Background scheduler tests pass
- ✅ Mock implementations updated correctly

## 🎛️ Usage

### API Endpoint
```bash
# With defaults (7 days, 24 hours)
curl -X POST http://localhost:3000/scrape \
  -H "Content-Type: application/json" \
  -d '{"item_id": 12345}'

# With custom values
curl -X POST http://localhost:3000/scrape \
  -H "Content-Type: application/json" \
  -d '{"item_id": 12345, "days_limit": 14, "frequency_hours": 12}'
```

### Configuration
```toml
# conf/config.toml
default_days_limit=7      # Days to refresh from initial scrape
default_frequency_hours=24  # Hours between refreshes
```

## 🏗️ Behavior

### Initial Request
1. ✅ **Validation**: Rejects zero/negative values with 400 Bad Request
2. ✅ **Database**: Stores URL with scheduling metadata
3. ✅ **Scheduling**: Always schedules initial scrape task
4. ✅ **Background**: No impact on immediate execution

### Background Refresh
1. ✅ **Every minute**: Checks for URLs due for refresh
2. ✅ **Frequency**: Refreshes if `last_scraped + frequency_hours <= NOW()`
3. ✅ **Time Limit**: Only if `date_added + days_limit >= NOW()`
4. ✅ **Automatic**: Stops refreshing after time limit expires

### Conflict Handling
- ✅ **Existing URLs**: Updates URL but preserves original scheduling metadata
- ✅ **Duplicate Requests**: Returns `AlreadyScheduled` response

## 🔧 Technical Details

### Database Query for Due URLs
```sql
SELECT id, url, last_scraped, frequency_hours, days_limit
FROM urls
WHERE 
    last_scraped IS NOT NULL
    AND date_added >= (NOW() - INTERVAL '1 day' * days_limit)
    AND last_scraped <= (NOW() - INTERVAL '1 hour' * frequency_hours)
ORDER BY last_scraped ASC
```

### Key Design Decisions
- 🎯 **Separation of Concerns**: DB stores data, business logic handles scheduling
- 🛡️ **Validation at API Layer**: Early rejection of invalid requests
- 🔒 **Immutable Scheduling**: Original scheduling metadata preserved on updates
- ⏰ **Time-Based Expiry**: Natural expiration using database constraints

## ✅ All Tests Passing
```
running 37 tests
test result: ok. 37 passed; 0 failed; 0 ignored; 0 measured
```

## 🚀 Ready for Production

The implementation is complete and tested with:
- ✅ Configurable defaults
- ✅ Request validation  
- ✅ Background scheduling
- ✅ Time-based expiration
- ✅ Comprehensive test coverage
- ✅ Database migration applied

The system now correctly refreshes comments based on time limits and frequency as requested!