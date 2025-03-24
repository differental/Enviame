# Enviame

<p align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/differental/enviame/deploy_prod.yml?label=Production&style=for-the-badge" />
  <img src="https://img.shields.io/github/actions/workflow/status/differental/enviame/deploy_main.yml?label=Beta&style=for-the-badge" />
  <img src="https://img.shields.io/badge/Made%20with-rust-red?style=for-the-badge" />
</p>

Priority Messenger, designed to patch the issues with instant messaging where message prioritisation is limited.

## Introduction

For frequent users of "Focus Mode" or equivalent features on various devices, there is often the issue of being unable to selectively allow important messages to go through while not being disturbed by every single notification. 

A popular solution is to mute all text messages and allow calls as the "emergency option", but mobile signals are not a guarantee whilst WhatsApp has limited call filtering features.

That's where Enviame comes in. Meaning "(to) send me" in Spanish, the tool is designed to allow prioritised communication from trusted friends and family to go through while blocking all other messages.

Features that integrate Enviame with smartwatches and other delivery options are planned.

## Tech Stack

- Backend: Rust (w/ Cargo)
- Frontend: Static HTML for now
