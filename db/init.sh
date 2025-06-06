#!/bin/bash
set -e

# Set default values if not already set
: "${PGHOST:=postgres}"
: "${PGUSER:=postgres}"
: "${PGPASSWORD:=postgres}"

export PGPASSWORD

# Wait for PostgreSQL to be ready
until psql -h "$PGHOST" -U "$PGUSER" -c '\q' > /dev/null 2>&1; do
  >&2 echo "PostgreSQL is unavailable - sleeping"
  sleep 1
done

>&2 echo "PostgreSQL is up - executing commands"

# Create database if it doesn't exist
psql -h "$PGHOST" -U "$PGUSER" -tc "SELECT 1 FROM pg_database WHERE datname = 'hrmDashboardDB'" | grep -q 1 || \
  psql -h "$PGHOST" -U "$PGUSER" -c "CREATE DATABASE hrmDashboardDB;"

# Run your SQL script
psql -h "$PGHOST" -U "$PGUSER" -d hrmDashboardDB -f /docker-entrypoint-initdb.d/hrm_db.sql

