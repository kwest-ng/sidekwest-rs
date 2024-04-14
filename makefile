DATABASE_URL := sqlite://testing.sqlite?mode=rwc
USE_DB := --database-url=${DATABASE_URL}

regen-entities:
	sea-orm-cli migrate fresh ${USE_DB}
	sea-orm-cli generate entity -o entities/src/entities/ ${USE_DB}

