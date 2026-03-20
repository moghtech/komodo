# Container Management

The lifetime of a docker container is more like a virtual machine. They can be created, started, stopped, and destroyed. Komodo will display the state of the container and provides an API to manage all your container's lifetimes.

This is achieved internally by running the appropriate docker command for the requested action (docker stop, docker start, etc).

### Stopping a Container

Sometimes you want to stop a running application but preserve its logs and configuration, either to be restarted later or to view the logs at a later time. It is more like *pausing* the application with its current config, as no configuration (like environment variable, volume mounts, etc.) will be changed when the container is started again. 

Note that in order to restart an application with updated configuration, it must be *redeployed*. stopping and starting a container again will keep all configuration as it was when the container was initially created.

### Container Redeploy

Redeploying is the action of destroying a container and recreating it. If you update deployment config, these changes will not take effect until the container is redeployed. Just note this will destroy the previous containers logs along with the container itself.