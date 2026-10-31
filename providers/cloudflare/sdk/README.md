# sdk

Cloudflare provider SDK for Giong.

Resources accept `name`, `config`, and a required `CloudflareProvider` provider. Add the
resource directly to a Stack:

```ts
import {Stack} from "@giong/sdk";
import {CloudflareProvider, Worker, Container} from "@giong/cloudflare";

const personal = new CloudflareProvider("personal", {
    accountId: process.env.CLOUDFLARE_PERSONAL_ACCOUNT_ID!,
    apiToken: process.env.CLOUDFLARE_PERSONAL_TOKEN!,
});
const company = new CloudflareProvider("company", {
    accountId: process.env.CLOUDFLARE_COMPANY_ACCOUNT_ID!,
    apiToken: process.env.CLOUDFLARE_COMPANY_TOKEN!,
});
const stack = new Stack();

const worker = new Worker("backend", {
    main: "src/index.ts",
    compatibilityDate: "2026-10-31",
    containers: [
        {className: "ApiContainer", binding: "API"},
        {className: "JobsContainer", binding: "JOBS"},
    ],
}, personal);
stack.add(worker);
stack.add(new Container("backend-api", {
    workerName: worker.name,
    className: "ApiContainer",
    image: "registry.example.com/api:latest",
    maxInstances: 2,
}, personal));
stack.add(new Container("backend-jobs", {
    workerName: worker.name,
    className: "JobsContainer",
    image: "registry.example.com/jobs:latest",
}, personal));
stack.add(new Worker("company-backend", {
    main: "src/company.ts",
    compatibilityDate: "2026-10-31",
}, company));

await stack.submit();
```

`CloudflareProvider` requires a profile name and account ID. Its API token can
be supplied explicitly or read from `CLOUDFLARE_API_TOKEN` in the stack process.
Every resource requires a provider argument. Provider URNs use the profile name
(`cloudflare::personal`). Use a distinct name for each profile.

Stack sends each used provider once, separately from resource inputs. The
reconciler supplies provider inputs to each WASM invocation through the host
context. Snapshots and responses contain the provider reference rather than
credentials. Every submission supplies the credentials again. Update both
`@giong/sdk` and the reconciler to use this submission format.

Workers and container applications are separate resources. Export each declared
Durable Object class from the Worker source. Worker `containers` entries only
declare class names and bindings; they do not create, update, or delete container
applications. Class names and binding names should be unique within a Worker.

`Container` uses package `cloudflare/container`. It manages image, scaling,
rollout, and application ID using the namespace of `workerName` and `className`.
Defaults are `maxInstances: 1`, `instanceType: "lite"`, and `observability: true`.
Worker observability is configured independently. A Container and its Worker
must use the same account/provider.

Apply the Worker before its Containers. Delete Containers before deleting the
Worker or removing their class declarations. Place them in that order in the
stack. Image-only updates use the Container resource and do not upload the
Worker script.

To migrate embedded containers, keep Worker name, class names, and bindings,
and move image/scaling settings into separate Container resources with the same
application names. Applying a Container reuses an existing application only
when linked to the expected Worker namespace. Worker state no longer stores
application IDs or performs container cleanup.
