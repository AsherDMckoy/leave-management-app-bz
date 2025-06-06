#!/bin/bash
createdb -U postgres leave_management_app
psql -U postgres -d leave_management_app -f db/leave_management_dump.sql

