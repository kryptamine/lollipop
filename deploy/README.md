# Deploying the collector

`make deploy` builds the collector in Docker and configures the VPS over SSH. It is
safe to run again: this is also how a new binary, key or rework list reaches the host.
The reasons are in [ADR 0010](../docs/adr/0010-ansible-configures-the-vps.md).

## What it needs

- On this machine: Docker and `uv`.
- An Ubuntu 26.04 VPS reachable over SSH, as root or a user with sudo.
- `deploy/inventory.yml`, copied from `inventory.example.yml` and filled in. Git ignores
  it, because it holds every secret.

## Check it after a deploy

The play succeeds even if the collector then fails, as with a rejected key.

```sh
systemctl status lollipop-collector
journalctl -u lollipop-collector -f
systemctl list-timers 'lollipop*'
```

## Move to another VPS

1. On the old host, stop the collector, so two hosts never spend one rate limit, and
   take a last backup:
   `systemctl disable --now lollipop-collector && systemctl start lollipop-backup`
2. Point `ansible_host` in `deploy/inventory.yml` at the new host and run `make deploy`.

`lollipop-restore.service` runs before the collector whenever the database is missing.
It loads the newest backup, starts empty if the bucket holds none, and keeps the
collector down if the storage cannot be read.

## Check a backup by hand

```sh
systemctl start lollipop-backup
rclone lsl r2:lollipop/backups   # with the variables of /etc/lollipop/backup.env set
```
