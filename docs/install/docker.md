# Docker

For a home server or a NAS. The image, `ghcr.io/zytocc/upnext`, has the server and the web UI, without the app window. It's published for amd64 and arm64, and Docker pulls the right one.

```sh
git clone https://github.com/zytocc/upnext && cd upnext
docker compose -f contrib/docker/compose.yaml up -d
docker logs upnext      # "Open http://localhost:8787 and enter 482 913 to pair your browser"
```

Open `http://localhost:8787` and type the code. The first code is in the log when UpNext starts and lasts an hour, until something is paired. `docker exec upnext upnext pair` prints a new one. For a script's token: `docker exec upnext upnext token create --name ha --scope read`.

To update, `docker compose -f contrib/docker/compose.yaml pull && docker compose -f contrib/docker/compose.yaml up -d`. To uninstall, `docker compose -f contrib/docker/compose.yaml down`; the `/data` volume stays until you remove it.

## Data and security

The database, backups and an optional `.env` are in the `/data` volume. The image runs as uid 1000; for another user, use `--user <uid>:<gid>` (compose: `user:`). The compose file also runs it read-only with no Linux capabilities; `docker run` takes `--read-only --tmpfs /tmp --security-opt no-new-privileges --cap-drop ALL`.

## Local libraries

Steam and Heroic folders come in only when mounted read-only. Uncomment the Steam or Heroic volume and its `UPNEXT_STEAM_DIR` or `UPNEXT_HEROIC_DIR` line in the compose file. Without them, Steam works through the Web API, GOG through its account, and Galaxy through an upload. GOG signs in with a "Send to UpNext" bookmark, since no window can open in a container.

## Your network

The compose file publishes the port on `127.0.0.1` only. Drop the `127.0.0.1:` to reach it from other devices, and set "Address for other devices". A device that asks to be allowed is shown in the log with the command that allows it: `docker exec upnext upnext pair approve 4821` ([Other devices](../other-devices.md#pairing)). Other computers find UpNext on your network only with `network_mode: host`; otherwise type its address.

## The AI

An API key, or an OpenAI-compatible server such as Ollama on the host at `http://host.docker.internal:11434/v1` (the compose file adds that name), works as it is. The image has no Claude, Codex or Copilot CLI.

For your Claude subscription, set `UPNEXT_INSTALL_CLAUDE: "1"`, or run `docker exec upnext upnext install-claude`; it downloads about 250 MB into `/data` once. Run `claude setup-token` on a computer where you use Claude and give the container `CLAUDE_CODE_OAUTH_TOKEN` from an `env_file` (`chmod 600`). The compose file has both lines, commented out.

## Running it

`TZ` sets the hour the daily sync runs at. Stopping lets a running job record where it got to, and the next start resumes it. The health check is `upnext healthcheck`.
