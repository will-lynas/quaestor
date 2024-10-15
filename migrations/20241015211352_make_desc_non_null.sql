ALTER TABLE transactions ADD COLUMN temp_description TEXT NOT NULL DEFAULT '';
UPDATE transactions SET temp_description = COALESCE(description, '');
ALTER TABLE transactions DROP COLUMN description;
ALTER TABLE transactions RENAME COLUMN temp_description TO description;
