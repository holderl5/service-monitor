Initial version of cloudflare worker that will monitor a list of services

Uses the hello-world template

For local testing:
Edit config.json and resource.json
Run make_dev_vars.sh

# Local development in docker

## Build Image
```
docker build  -t cf-rust .
```
## Run Image
```
docker run --rm -it -p 127.0.0.1:8787:8787 -v .:/home/developer/app -w /home/developer/app --name cf-rust cf-rust
```
With the --rm, the container will self delete on exit

I ran into issues running, this seemed to fix the problem, not sure how important it still is:
```
npx wrangler dev --ip 0.0.0.0
```
