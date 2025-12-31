# Test Containers

Docker containers for use in testing the application are defined here.

## Deploying

To preserve the assets uploaded to the *development* instance of the namazu container, a volume mount is needed.

```shell
env NAMAZU_ASSETS_PATH=/zeniba/namazu/assets docker compose up --build -d
```
