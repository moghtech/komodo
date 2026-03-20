# Configuration

## Choose the docker image

There are two options to configure the docker image to deploy. 

### Attaching a Komodo build
If the software you want to deploy is built by Komodo, you can attach the build directly to the deployment.

By default, Komodo will deploy the latest available version of the build, or you can specify a specific version using the version dropdown.

Also by default, Komodo will use the same docker account that is attached to the build in order to pull the image on the periphery server. If that account is not available on the server, you can specify another available account to use instead, this account just needs to have read access to the docker repository.

### Using a custom image
You can also manually specify an image name, like `mongo` or `ghcr.io/mbecker20/random_image:0.1.1`.

If the image repository is private, you can still select an available docker account to use to pull the image.

## Configuring the network

One feature of docker is that it allows for the creation of [virtual networks between containers](https://docs.docker.com/network/). Komodo allows you to specify a docker virtual network to connect the container to, or to use the host system networking to bypass the docker virtual network.

The default selection is `host`, which bypasses the docker virtual network layer.

If you do select select a network other than host, you can specify port bindings with the GUI. For example, if you are running mongo (which defaults to port 27017), you could use the mapping:

```
27018 : 27017
```

In this case, you would access mongo from outside of the container on port `27018`.

Note that this is not the only effect of using a network other than `host`. For example, containers running on different networks can not communicate, and ones on the same network can not reach other containers on `localhost` even when they are running on the same system. This behavior can be a bit confusing if you are not familiar with it, and it can be bypassed entirely by just using `host` network.

## Configuring restart behavior

Docker, like systemd, has a couple options for handling when a container exits. See [docker restart policies](https://docs.docker.com/config/containers/start-containers-automatically/). Komodo allows you to select the appropriate restart behavior from these options.

## Configuring environment variables

Komodo enables you to easily manage environment variables passed to the container. 
In the GUI, navigate to the environment tab of the configuration on the deployment page.

You pass environment variables just as you would with a ```.env``` file:

```
ENV_VAR_1=some_value
ENV_VAR_2=some_other_value
```

## Configuring volumes

A docker container's filesystem is segregated from that of the host. However, it is still possible for a container to access system files and directories, this is accomplished by using [bind mounts](https://docs.docker.com/storage/bind-mounts/).

Say your container needs to read a config file located on the system at ```/home/ubuntu/config.toml```. You can specify the bind mount to be:

```
/home/ubuntu/config.toml : /config/config.toml
```

The first path is the one on the system, the second is the path in the container. Your application would then read the file at ```/config/config.toml``` in order to load its contents.

These can be configured easily with the GUI in the 'volumes' card. You can configure as many bind mounts as you need.

## Extra args

Not all features of docker are mapped directly by Komodo, only the most common. You can still specify any custom flags for Komodo to include in the `docker run` command by utilizing 'extra args'. For example, you can enable log rotation using these two extra args:

```
--log-opt max-size=10M
```
```
--log-opt max-file=3
```

## Command

Sometimes you need to override the default command in the image, or specify some flags to be passed directly to the application. What is put here is inserted into the docker run command after the image. For example, to pass the `--quiet` flag to MongoDB, the docker run command would be:

```
docker run -d --name mongo-db mongo:6.0.3 --quiet
```

In order to achieve this with Komodo, just pass `--quiet` to 'command'.
