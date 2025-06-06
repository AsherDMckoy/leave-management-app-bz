# HRM Leave Management App

## Table of Contents

1. [Overview](#overview)
2. [Features](#features)
3. [Architecture](#architecture)
4. [Database Schema](#database-schema)
5. [Backend Structure](#backend-structure)
6. [Frontend Structure](#frontend-structure)
7. [API Endpoints](#api-endpoints)
8. [Authentication & Security](#authentication--security)
9. [Notifications System](#notifications-system)
10. [How to Run](#how-to-run)
11. [Customization](#customization)
12. [Contribution](#contribution)
13. [License](#license)

## Overview

The HRM Leave Management App is a web-based system for managing employee leave requests, approvals, and team leave calendars. It supports role-based access for Admins, Team Leads, Officers, and Human Resources, and provides a modern, interactive UI with real-time notifications.

## Features

- **User Authentication:** Secure login/logout, password hashing (Argon2), and session management.
- **Leave Requests:** Submit, edit, delete, and view leave requests with support for multiple leave types.
- **Team Calendar:** Visual calendar for team leave schedules with fixed headers and scrollable member lists.
- **Notifications:** Real-time notifications for team leads and HR about new leave requests, with the ability to mark as read.
- **Role-Based Access:** Different views and permissions for Admin, Team Lead, Officer, and Human Resources.
- **Filtering & Search:** Filter leave requests by status and search comments.
- **Responsive UI:** Modern, user-friendly interface with HTMX for dynamic updates.

## Architecture

- **Backend:** Rust (Axum, SQLx, Askama)
- **Frontend:** HTML templates (Askama), CSS, JavaScript (HTMX)
- **Database:** PostgreSQL

### Folder Structure

```
src/
  handlers/      # Route handlers (leave, notification, profile, auth, etc.)
  models/        # Data models (user, leave, notification, team)
  templates.rs   # Askama template structs
  routes.rs      # Route definitions
  main.rs        # App entry point
templates/       # HTML templates (Askama)
assets/          # Static files (CSS, JS, images)
```

## Database Schema

### users

| Column      | Type    | Description           |
|-------------|---------|-----------------------|
| id          | int     | Primary key           |
| username    | string  | Unique user name      |
| password    | string  | Argon2 password hash  |
| email       | string  | Email address         |
| role        | string  | User role             |
| name        | string  | Full name             |
| team_id     | int     | Foreign key to teams  |

### leave_requests

| Column        | Type    | Description           |
|---------------|---------|-----------------------|
| id            | int     | Primary key           |
| user_id       | int     | Foreign key to users  |
| start_date    | date    | Leave start date      |
| end_date      | date    | Leave end date        |
| leave_type    | string  | Type of leave         |
| status        | string  | Pending/Approved/Rejected |
| comments      | string  | Optional comments     |
| days          | int     | Number of leave days  |

### notifications

| Column         | Type    | Description                |
|----------------|---------|----------------------------|
| id             | int     | Primary key                |
| recipient_id   | int     | User to notify             |
| leave_request_id| int    | Related leave request      |
| message        | string  | Notification message       |
| is_read        | bool    | Read status                |
| created_at     | datetime| Timestamp                  |

### teams

| Column      | Type    | Description           |
|-------------|---------|-----------------------|
| id          | int     | Primary key           |
| name        | string  | Team name             |
| team_lead_id| int     | User ID of team lead  |

## Backend Structure

### Models (`src/models/`)

- **user.rs:** User struct, roles, authentication logic.
- **leave.rs:** Leave request types, status, and data structures.
- **notification.rs:** Notification struct for user alerts.
- **team.rs:** Team and team member structs.

### Handlers (`src/handlers/`)

- **auth.rs:** Login, logout, and session management.
- **leave.rs:** All leave request logic (submit, update, delete, calendar, etc.).
- **notification.rs:** Notification creation, fetching, and marking as read.
- **profile.rs:** User profile and password change logic.

### Templates (`src/templates.rs` + `templates/`)

- Rust structs for Askama templates.
- HTML templates for all pages and partials.

### Routing (`src/routes.rs`)

- All HTTP routes and their handler bindings.

## Frontend Structure

### Templates

- **base.html:** Main layout, navigation, notification panel.
- **login.html:** Login form.
- **profile.html:** User profile and password change.
- **calendar.html:** Team leave calendar.
- **leaves_list.html:** User's leave requests with filters.
- **notifications.html:** Notification panel.
- **requests.html:** Admin/HR leave request review.

### Static Assets

- **CSS:** `assets/styles/style.css` (modern, responsive, color-coded leave types, notification styles)
- **JS:** `assets/js/app.js assets/js/calendar.js` (notification panel logic, calendar functionality, HTMX triggers)
- **Icons:** `assets/icons/`

## API Endpoints

### Authentication

- `POST /login` — User login.
- `GET /logout` — User logout.
- `POST /change-password` — Change user password.

### Leave Management

- `GET /leaveslist` — View all leave requests for the user.
- `POST /submit_leave` — Submit a new leave request.
- `PUT /leave/:id` — Update an existing leave request.
- `DELETE /leave/:id` — Delete a leave request.
- `GET /calendar` — View the team leave calendar.
- `GET /api/calendar` — Get team leave data as JSON.

### Notifications

- `GET /notifications` — Fetch unread notifications for the logged-in user.
- `PUT /notifications/:id/mark-read` — Mark a notification as read.

### Requests (Admin/HR/Team Lead)

- `GET /requests` — View all pending requests.
- `GET /requests/pending` — View pending requests.
- `GET /requests/approved` — View approved requests.
- `GET /requests/rejected` — View rejected requests.
- `POST /approve_request/:id` — Approve a leave request.
- `POST /reject_request/:id` — Reject a leave request.

### User Management (HR)

- `GET /new_user` — Add new user form.
- `POST /add_user` — Create a new user.

## Authentication & Security

- **Password Hashing:** All passwords are hashed using Argon2.
- **Session Management:** Uses axum-login for secure sessions.
- **Role-Based Access:** Only authorized users can access certain endpoints (e.g., only HR can add users).
- **CSRF Protection:** CSRF tokens for sensitive forms (e.g., password change, add user).
- **Error Handling:** All database and template errors are logged and return appropriate HTTP status codes.

## Notifications System

- **Creation:** Notifications are created for team leads or HR when a leave request is submitted, approved, or rejected.
- **Fetching:** Only unread notifications are fetched and displayed.
- **Mark as Read:** Notifications disappear from the panel when marked as read.
- **UI:** Notification panel slides in from the side, with a bell icon indicator.

## How to Run

### Recommended: Using Docker Compose

The easiest way to run the application and database is with Docker Compose. This will set up both the Rust web server and a PostgreSQL database, and automatically initialize the database using the provided SQL dump.

1. **Install Docker and Docker Compose:**
   - [Install Docker](https://docs.docker.com/get-docker/)
   - [Install Docker Compose](https://docs.docker.com/compose/install/)

2. **Start the application and database:**
   ```sh
   docker-compose up --build
   ```
   This will build the Rust application, start the web server, and launch a PostgreSQL database. The database will be initialized using the SQL dump located in the `db/` directory.

3. **Access the app:**
   Open your browser and go to `http://localhost:8000` (or your configured port).

### Manual Setup (Advanced)

If you prefer to run the app outside Docker:

1. **Install Rust and Cargo:**  
   [Install Rust](https://www.rust-lang.org/tools/install)

2. **Install PostgreSQL:**
   - [Install PostgreSQL](https://www.postgresql.org/download/)

3. **Set up the database using the SQL dump:**
   - Create a new PostgreSQL database (e.g., `hrmDashboardDB`).
   - Load the schema and initial data:
     ```sh
     psql -U <your_user> -d hrmDashboardDB -f db/hrm_db.sql
     ```

4. **Configure environment variables:**
   - Set your `DATABASE_URL` and any other required settings.

5. **Install dependencies and run the server:**
   ```sh
   cargo build
   cargo run
   ```

6. **Access the app:**
   Open your browser and go to `http://localhost:8000` (or your configured port).

## Prebuilt Logins

You can use the following prebuilt accounts to access the application immediately after setup:

### Admin Accounts
- **Username:** Ada64 &nbsp;&nbsp; **Password:** LaceLove
- **Username:** Hypatia &nbsp;&nbsp; **Password:** Alexandria

### Human Resources (HR) Accounts
- **Username:** Tobi &nbsp;&nbsp; **Password:** password
- **Username:** Bully2002 &nbsp;&nbsp; **Password:** SpideySenses

### Officer Accounts
- For basic officer accounts, please use the "Create Employee" feature available to HR users in the application interface.
