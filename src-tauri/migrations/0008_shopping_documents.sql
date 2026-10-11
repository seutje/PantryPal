-- Versioned generated documents support unresolved lines and i128 exact totals.
-- Original Phase 1 shopping rows remain intact and are carried into documents
-- on first edit. No destructive migration or implicit adjustment reset.
ALTER TABLE shopping_list ADD COLUMN revision INTEGER NOT NULL DEFAULT 0 CHECK(revision >= 0);
ALTER TABLE shopping_list ADD COLUMN document_json TEXT CHECK(document_json IS NULL OR json_valid(document_json));
