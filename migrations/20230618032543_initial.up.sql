CREATE TABLE IF NOT EXISTS "tick" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "start_time" TIMESTAMP NOT NULL,
    "current_tick" INTEGER NOT NULL,
    "current_check" INTEGER NOT NULL
);

INSERT INTO "tick" ("id", "start_time", "current_tick", "current_check") VALUES (1, DATETIME(), 0, 0);

CREATE TABLE IF NOT EXISTS "teams" (
	"id" INTEGER NOT NULL PRIMARY KEY,
    "arbitrary_bonus_points" INTEGER NOT NULL DEFAULT 0,
	"ip" INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS "services" (
	"name" VARCHAR(16) NOT NULL PRIMARY KEY,
	"port" INTEGER NOT NULL,

    "atk_score" INTEGER NOT NULL,
    "def_score" INTEGER NOT NULL,
    "up_score" INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS "flags" (
    "tick" INTEGER NOT NULL,
    "team_id" INTEGER NOT NULL,
    "service_name" VARCHAR(16) NOT NULL,
    "flag" VARCHAR(255) NOT NULL,
    "flag_id" TEXT,

    "in_progress" BOOLEAN NOT NULL DEFAULT TRUE,
    "claimed_by" INTEGER,
    "defended" BOOLEAN NOT NULL DEFAULT FALSE,

    "created" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY ("tick", "team_id", "service_name"),
    FOREIGN KEY ("team_id") REFERENCES "teams"("id"),
    FOREIGN KEY ("service_name") REFERENCES "services"("name")
);

CREATE TABLE IF NOT EXISTS "check_ups" (
    "id" INTEGER NOT NULL,
    "team_id" INTEGER NOT NULL,
    "service_name" TEXT NOT NULL,

    "in_progress" BOOLEAN NOT NULL DEFAULT TRUE,
    "up" BOOLEAN NOT NULL,
    "timestamp" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY ("id", "team_id", "service_name"),
    FOREIGN KEY ("team_id") REFERENCES "teams"("id"),
    FOREIGN KEY ("service_name") REFERENCES "services"("name")
);