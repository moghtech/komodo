# Image Versioning

Komodo uses a major.minor.patch versioning scheme to Build versioning. By default, every RunBuild will auto increment the Build's version patch number, and push the image to docker hub with the version tag, as well as the `latest` tag. A tag containing the latest short commit hash at the time the repo was cloned will also be created. 

You can also turn off the auto incrementing feature, and manage the version yourself. In addition, you can configure a "version tag" on the build. This will postfix the version tag / commit hash tag with a custom label. For example, an image tag of `dev` will produce tags like `image_name:1.1.1-dev` and `image_name:h3c87c-dev`.