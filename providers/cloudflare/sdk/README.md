# sdk

Cloudflare provider SDK for Giong.

## Worker custom domains

`WorkerDomain` attaches a hostname to an existing Worker, including the Worker
created by `Containers`:

```ts
import { Stack } from "@giong/sdk";
import { WorkerDomain } from "@giong/cloudflare";

const stack = new Stack();
stack.add("domain", new WorkerDomain({
  hostname: "app.example.com",
  zoneId: "<CLOUDFLARE_ZONE_ID>",
  service: "littledays", // Worker name, matching Containers.name
}));
await stack.submit();
```

The resource type is `cloudflare.worker-domain`. It uses the existing
`CLOUDFLARE_ACCOUNT_ID` and `CLOUDFLARE_API_TOKEN` secrets. The token needs
Workers Scripts Write permission, and the hostname must belong to the specified
Cloudflare zone. Cloudflare manages the custom domain's DNS and TLS certificate.

Resources are applied sequentially. Add `Containers` before `WorkerDomain` in the
same stack so the Worker exists before its domain is attached. Delete the domain
before deleting its Worker.

The provider reads domains by hostname and zone, updates the target Worker with
PUT, and deletes bindings by domain ID. When a previous state is supplied for a
hostname change, it attaches the new domain before removing the old binding.

API reference: https://developers.cloudflare.com/api/resources/workers/subresources/domains/
