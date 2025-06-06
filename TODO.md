# HRM Dashboard Project TODO

## Project Setup
- [x] Initialize Rust project for backend
- [x] Initialize npm for frontend development
- [x] Enable hot reloading with cargo watch

## Backend
- [x] Set up Axum server
- [x] Create basic routes (`/` and `/auth/login`)
  - [x] `GET /api/leaves_requests`: Retrieve employee pending leave requests
  - [x] `POST /api/request_leave`: Submitt new leave request pending approval
- [x] Add database integration
  - [x] Set up PostgreSQL database
  - [x] Create Employee table
  - [x] Inssert Dummy data for teams, managers and users
  - [x] Insert new leave request into table 
  - [x] Read pending leave requests from table
  - [x] Implement database operations in Axum
- [x] Add authentication
  - [x] Implement user login and registration
  - [x] Protect API routes with authentication

## Frontend
- [x] Create initial HTML structure
  - [x] `index.html`: Main dashboard page
- [x] Add HTMX for dynamic interactions
  - [x] Add page to request new leaves
  - [x] Add calendar page
  - [x] Load pending leaves requests for manager and team_lead users
- [x] Create CSS for styling
  - [x] `styles.css`: Basic styling for the dashboard
- [x] Create JavaScript for additional functionality
  - [x] `app.js`: Handle HTMX events

## Development Tools
- [x] Install development dependencies
  - [x] `html-minifier-terser`
  - [x] `cssnano`
  - [x] `terser`

