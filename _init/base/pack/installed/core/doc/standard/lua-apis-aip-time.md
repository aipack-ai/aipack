## aip.time

Functions for retrieving current time, date, and timezone information in UTC or local time.

### Functions Summary

```lua
aip.time.now_iso_utc(): string
aip.time.now_iso_local(): string
aip.time.now_iso_utc_micro(): string
aip.time.now_iso_local_micro(): string
aip.time.now_utc_micro(): integer
aip.time.today_utc(): string
aip.time.today_local(): string
aip.time.today_iso_utc(): string
aip.time.today_iso_local(): string
aip.time.weekday_utc(): string
aip.time.weekday_local(): string
aip.time.local_tz_id(): string
```

### Usage Examples

```lua
aip.time.now_iso_utc(): string            -- RFC 3339 UTC (seconds precision)
-- e.g., "2025-08-23T14:35:12Z"

aip.time.now_iso_local(): string          -- RFC 3339 local time (seconds precision)
-- e.g., "2025-08-23T09:35:12-05:00"

aip.time.now_iso_utc_micro(): string      -- RFC 3339 UTC (microseconds)
-- e.g., "2025-08-23T14:35:12.123456Z"

aip.time.now_iso_local_micro(): string    -- RFC 3339 local time (microseconds)
-- e.g., "2025-08-23T09:35:12.123456-05:00"

aip.time.now_utc_micro(): integer         -- epoch microseconds (UTC)
-- e.g., 1766561712123456

aip.time.today_utc(): string              -- weekday + date (UTC)
-- e.g., "Saturday 2025-08-23"

aip.time.today_local(): string            -- weekday + date (local)
-- e.g., "Saturday 2025-08-23"

aip.time.today_iso_utc(): string          -- "YYYY-MM-DD" (UTC)
-- e.g., "2025-08-23"

aip.time.today_iso_local(): string        -- "YYYY-MM-DD" (local)
-- e.g., "2025-08-23"

aip.time.weekday_utc(): string            -- weekday name (UTC)
-- e.g., "Saturday"

aip.time.weekday_local(): string          -- weekday name (local)
-- e.g., "Saturday"

aip.time.local_tz_id(): string            -- IANA timezone id for local zone
-- e.g., "America/Los_Angeles"
```
