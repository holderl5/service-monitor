# Overview
Cloudflare worker to monitor and alert for downtime on a list of services.  This worker is authored in rust and uses https://github.com/cloudflare/workers-rs for the runtime.  Configuration is via a JSON object that goes in your worker's environment.  The repository was created with the workers-rs hello-world template.

# Why
Read the WHY here: link pending

# Status
This is the initial version, there are a few TODOs left to make it complete.

# Local Testing
To make it easier to add/change the services, a shell script manipulates json files and places the data into the .dev.vars file for local testing with wrangler.

Edit config.json and resource.json
Run ./make_dev_vars.sh to overwrite .dev.vars
