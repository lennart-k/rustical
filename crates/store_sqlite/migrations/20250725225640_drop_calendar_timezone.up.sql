-- We don't want to save timezones as ics anymore
-- but instead just rely on the TZDB identifier
ALTER TABLE calendars DROP COLUMN timezone;
