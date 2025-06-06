#!/bin/bash
createdb -U postgres leave_management_app
psql -U postgres -d leave_management_app -f hrm_db.sql

