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
11. [Deployment Options](#deployment-options)  
12. [Login Access](#prebuilt-logins)

---

## 🧭 Overview

The **HRM Leave Management App** is a role-based, web-based system designed for managing employee leave workflows within small-to-medium Belizean organizations. Built with **Rust + Axum**, it features a modern UI, secure access control, real-time notifications, and a PostgreSQL backend—all orchestrated via Docker or cloud deployment.

---

## 🌟 Features

- ✅ Secure login/logout with session management (Argon2, axum-login)  
- 📅 Submit/edit/delete leave requests with approval workflows  
- 📈 Team calendar with dynamic UI via HTMX  
- 🔔 Real-time notifications for admins and HR  
- 🧩 Role-based access: Admins, HR, Officers (future: Team Leads)  
- 🔍 Search and filter leave requests  
- 📱 Fully responsive UI and modern styling  

---

## 🏗️ Architecture

- **Backend:** Rust (Axum, SQLx, Askama)  
- **Frontend:** Askama templates, HTMX, JS, CSS  
- **Database:** PostgreSQL  
- **ORM:** SQLx (chosen over Diesel for async integration, simplicity, and community support)  
- **Security:** Serde (sanitization), Argon2 (password hashing), Tower Sessions  
- **Containerization:** Docker + Docker Compose  
- **Hosting (optional):** Render

---

## 📚 Database Schema

### users

| Column      | Type    | Description           |
|-------------|---------|-----------------------|
| id          | int     | Primary key           |
| username    | string  | Unique user name      |
| password_hash    | string  | Argon2 password hash  |
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

## 🧱 Backend Structure

- `handlers/`: Route controllers  
- `models/`: Database-bound structs  
- `templates.rs`: Askama template bindings  
- `routes.rs`: Route-to-handler map  
- `main.rs`: App entrypoint  
- `profile.rs`: User profile and password change logic

---

## 🎨 Frontend Structure

- `templates/`: Layout and dynamic HTML (HTMX + Askama)  
- `assets/styles/`: Custom CSS  
- `assets/js/`: JS for notifications and calendar  
- `assets/icons/`: Icons  

---

## 🔌 API Endpoints

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

---

## 🛡️ Authentication & Security

- Passwords hashed via Argon2  
- Session auth via `axum-login`  
- CSRF protection via tokens  
- Role-based route access  
- Graceful fallback on errors  

---

## 🔔 Notifications System

- Notifies HR/Admins on leave actions  
- “Mark as read” toggle  
- Fetches unread only  
- Slide-in UI panel with icon alerts  

---

## 🧪 How to Run

### Getting Started - Clone the Repository if not already provided with zip file
```bash
git clone https://github.com/AsherDMckoy/leave-management-app-bz.git
cd leave-management-app-bz
```

###OR 

### Unzip provided zip repo
```bash
unzip leave-management-app-bz-main.zip
cd leave-management-app-bz-main 
```

### 🔁 Recommended: Docker Compose

**1. Prerequisites**  
- [Install Docker](https://docs.docker.com/get-docker/)  
- [Install Docker Compose](https://docs.docker.com/compose/install/)

**2. Start application**  
```bash
docker-compose up --build
```

This builds the app and initializes a PostgreSQL DB using the SQL file in `/db/`.

**3. Access the app:**  
Go to [http://localhost:8000](http://localhost:8000)

---

### ⚙️ Manual Setup (Advanced)

**1. Install** [Rust](https://www.rust-lang.org/tools/install) & PostgreSQL  
**2. Create a DB**, then load `db/hrm_db.sql`  
```bash
psql -U <user> -d hrmDashboardDB -f db/hrm_db.sql
```

**3. Set `.env`**  
Set `DATABASE_URL` and other env vars.

**4. Run the app**  
```bash
cargo build
cargo run
```

---

## 🌐 Deployment Options

| Method             | Description |
|-------------------|-------------|
| **Docker Compose** | Local deployment with bundled PostgreSQL database. |
| **Manual Setup**   | Advanced manual setup using Rust and PostgreSQL outside Docker. |
| **Render (Cloud Hosting)** |  
Hosted live version of the application:  
🔗 **[https://leave-management-app-bz.onrender.com](https://leave-management-app-bz.onrender.com)**  
📅 **Available until July 6th, 2025**

If you prefer to run the backend locally and connect to the cloud database, use the following:

```bash
DATABASE_URL=postgresql://leave_management_db_0u7z_user:GHlmym3G0sewKD08sImMAArMozCrk9W8@dpg-d11639adbo4c739mh6b0-a.oregon-postgres.render.com/leave_management_db_0u7z
```

> ⚠️ If the Render link becomes unavailable, fallback to local Docker deployment or use the manual setup method.

---

## 🔐 Prebuilt Logins (username / password)

### Admins  
- **Ada64 / LaceLove**  
- **Hypatia / Alexandria**

### HR  
- **Tobi / password**  
- **Bully2002 / SpideySenses**

> Officer accounts can be created via HR panel in-app.
