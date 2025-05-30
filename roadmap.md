# Roadmap

In order to clarify the goals and invite community participation in the direction of the project, this document will serve as a roadmap for upcoming features / releases.

If you have an idea for Komodo, feel free to open an issue beginning with the `[Request]` tag. The community is also encouraged to open PRs fulfilling the goals of any planned release.

## Release plans

- **v1.12**: Support any git provider / docker registry (supports self-hosted providers like Gitea) ✅
- **v1.13**: Support "Compose" resource - Paste in a docker compose file and manage it like a Portainer "Stack" ✅
- **v1.14**: Manage docker networks, images, volumes in the UI ✅
- **v1.15**: Support generic OIDC providers (including self-hosted) ✅
- **v1.16**: "Action" resource: Run requests on the Komodo API using snippets of typescript. ✅
- **v1.17**: Procedure Schedules: Run procedures / Actions at scheduled times, like CRON job. Connect to host terminals and exec into containers ✅
- **v1.18**: Upgrade granular role based access control system ✅
- **Undecided**: Support "Swarm" resource - Manage docker swarms, attach Deployments / Stacks to "Swarm". 
- **Undecided**: Support "Cluster" resource - Manage Kubernetes cluster, can attach deployments to "Cluster"

**Note. The specific versions associated with these features are not final.**