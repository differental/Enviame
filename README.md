<div align="center">
  <h1>Enviame</h1>
  <h3>Priority Messenger</h3>
  <p>Demo: <a href="https://msg-beta.brianc.tech">https://msg-beta.brianc.tech</a></p>
  <p>Project Writeup: <a href="https://gist.github.com/differental/086dfca65d17e629befb58d032a7fbf3">Gist</a></p>
  <p>
    <picture>
      <a href="https://github.com/differental/enviame/actions/workflows/ci_prod.yml">
        <img src="https://img.shields.io/github/actions/workflow/status/differental/enviame/ci_prod.yml?label=Production&style=for-the-badge" />
      </a>
    </picture>
    <span>&nbsp;&nbsp;&nbsp;</span>
    <picture>
      <a href="https://github.com/differental/enviame/actions/workflows/ci_beta.yml">
        <img src="https://img.shields.io/github/actions/workflow/status/differental/enviame/ci_beta.yml?label=Beta&style=for-the-badge">
      </a>
    </picture>
  </p>
  <p>
    <picture>
      <a href="https://github.com/differental/enviame/tree/main">
        <img src="https://img.shields.io/github/last-commit/differental/enviame?style=for-the-badge" />
      </a>
    </picture>
    <span>&nbsp;&nbsp;&nbsp;</span>
    <picture>
      <a href="https://github.com/differental/enviame/blob/main/LICENSE">
        <img src="https://img.shields.io/github/license/differental/enviame?style=for-the-badge&color=499dd0" />
      </a>
    </picture>
    <span>&nbsp;&nbsp;&nbsp;</span>
    <picture>
      <a href="https://www.rust-lang.org/">
        <img src="https://img.shields.io/badge/Made%20with-rust-red?style=for-the-badge" />
      </a>
    </picture>
  </p>
</div>

## Introduction

***TL;DR** Enviame is a priority messenger, designed as a simple solution to limited message prioritisation with instant messaging applications and protocols.*

For frequent users of "Focus Mode" or equivalent features on various devices, there is often the issue of being unable to selectively allow important messages to go through while not being disturbed by every single notification.

A popular solution is to mute all text messages and allow calls as the "emergency option", but mobile signals are not a guarantee whilst WhatsApp has limited call filtering features.

That's where Enviame comes in. Enviame is designed to allow more nuanced control over prioritised communication from trusted friends and family.

Features that integrate Enviame with smartwatches and other delivery options are planned.

## Tech Stack

- **Backend**: Rust
    - `axum`: Web Framework
    - `tokio`: Asynchronous Runtime
    - `lettre`: SMTP Email Delivery
    - `sqlx`: Database Interaction
    - `askama`: Static HTML Templating
- **Database**: PostgreSQL
- **Frontend**: 
    - Static HTML
    - Vanilla JS
    - Bootstrap UI

## Configuration

### Roles

The `role` column of the `users` table is used to distinguish between different categories of users. It is stored as an integer, and it is 0 by default during registration.

When `role` is 1 and the user is verified (logged in via their token once), the frontend will display a golden tick instead of a blue tick, and "sender_type" of their messages will be "trusted" instead of "verified", this is also displayed in email notifications.

This column is added in `v1.1.0` and designed with extensibility in mind. When deploying your own instance, you can easily add special operations or restrictions for specific `role` values. The `role` values are also returned by the login API and processed by the frontend.

### `.env`

```ini
# Homepage url. Used when sending login links to users
HOMEPAGE_URL=https://example.com/

# Database URL, see scripts/schema.sql
DATABASE_URL=postgres://user:password@localhost/dbname

# Calendar ICS URL, optional
CALENDAR_URL=https://example.com/personal.ics

# Google reCaptcha keys
RECAPTCHA_SITE_KEY=recaptcha_site_key
RECAPTCHA_SECRET_KEY=recaptcha_secret

# Hash key for message ID veification
HASH_KEY=random_string_here

# Recipient address of all notification emails, and reply_to address of all user emails
NOTIFICATION_EMAIL=name@domain.com

# SMTP Credentials
SMTP_SERVER=smtp.xxx.com
SMTP_PORT=587
SMTP_USERNAME=name@domain.com
SMTP_PASSWORD=abcdefghijklmnop

# Address where emails are sent from
# Can be different from SMTP_USERNAME
SMTP_FROM=from@domain.com
SMTP_FROM_URGENT=from-urgent@domain.com
SMTP_FROM_IMMEDIATE=from-immediate@domain.com

# App Port
APP_PORT=3000

# Deploy environment, relevant in displaying beta warning and modifying db below
DEPLOY_ENV=dev
```


## Database Schema

`messages`:

```text
  column_name   |        data_type         | is_nullable |            column_default            
----------------+--------------------------+-------------+--------------------------------------
 id             | integer                  | NO          | nextval('messages_id_seq'::regclass)
 submitted_time | timestamp with time zone | NO          | CURRENT_TIMESTAMP
 user_uid       | integer                  | YES         | 
 name           | text                     | NO          | 
 email          | text                     | NO          | 
 message        | text                     | NO          | 
 priority       | text                     | NO          | 
 status         | text                     | NO          | 'pending'::text
 sender         | text                     | NO          | 
```

`users`:

```text
 column_name |        data_type         | is_nullable |           column_default           
-------------+--------------------------+-------------+------------------------------------
 uid         | integer                  | NO          | nextval('users_uid_seq'::regclass)
 added_time  | timestamp with time zone | NO          | CURRENT_TIMESTAMP
 name        | text                     | NO          | 
 email       | text                     | NO          | 
 token       | text                     | NO          | 
 verified    | boolean                  | NO          | 
 role        | integer                  | NO          | 
```
