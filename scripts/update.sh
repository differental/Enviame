#!/bin/bash

# This is an example of the deployment script referenced in the old deployment workflow.
# The script runs on the target server, pulls the latest commit from a branch, builds the 
#   server binary and restarts the relevant systemd service.

# This is designed to be a simple script, and thus you will need to manually install rustup,
#   configure the git repository and configure the systemd service before using this.

# This has been replaced by the current CI workflow.


. "$HOME/.cargo/env"

WORKDIR="/path/to/enviame" # Must be a git repository with remote set up
BRANCH="prod" # or "main"

cd "$WORKDIR" || { echo "Failed to enter $WORKDIR"; exit 1; }

echo "Pulling latest changes from branch $BRANCH"
if ! git pull origin "$BRANCH" > /dev/null 2>> error.log; then
    echo "Git pull failed, trying force pull..."

    echo "Fetching latest changes"
    git fetch origin > /dev/null 2>> error.log || { echo "Git fetch failed"; exit 1; }

    echo "Resetting to remote branch state"
    git reset --hard origin/"$BRANCH" > /dev/null 2>> error.log || { echo "Git hard reset failed"; exit 1; }
fi

echo "Building with cargo build"
if ! cargo build --release > /dev/null 2>> error.log; then
    echo "Cargo build failed, trying cargo clean..."

    echo "Cleaning old builds"
    cargo clean > /dev/null 2>> error.log || { echo "Cargo clean failed"; exit 1; }

    echo "Retrying building"
    cargo build --release > /dev/null 2>> error.log || { echo "Cargo build failed"; exit 1; }
fi

echo "Restarting service"
sudo systemctl restart enviame > /dev/null 2>> error.log || { echo "Failed to restart service"; exit 1; }

echo "Deployment successful!"